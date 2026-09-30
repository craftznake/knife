use clap_complete::{ArgValueCompleter, engine::CompletionCandidate};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const RETAIN_PER_RESOURCE: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Recent {
    pub profile: String,
    pub account_id: String,
    pub resource: String,
    pub identifier: String,
    pub used_at: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct RecentFile {
    entries: Vec<Recent>,
}

pub fn disabled() -> bool {
    std::env::var("KNIFE_DISABLE_RECENTS").is_ok_and(|value| value == "1")
}

fn cache_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/knife/recents.json"))
}

fn load(path: &Path) -> Vec<Recent> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str::<RecentFile>(&contents)
        .map(|file| file.entries)
        .unwrap_or_default()
}

pub fn record(profile: &str, account_id: &str, resource: &str, identifier: &str) {
    if disabled() || profile.is_empty() || account_id.is_empty() || identifier.is_empty() {
        return;
    }
    if let Some(path) = cache_path() {
        record_at(&path, profile, account_id, resource, identifier);
    }
}

fn record_at(path: &Path, profile: &str, account_id: &str, resource: &str, identifier: &str) {
    let mut entries = load(path);
    entries.retain(|entry| {
        !(entry.profile == profile
            && entry.account_id == account_id
            && entry.resource == resource
            && entry.identifier == identifier)
    });
    entries.push(Recent {
        profile: profile.to_owned(),
        account_id: account_id.to_owned(),
        resource: resource.to_owned(),
        identifier: identifier.to_owned(),
        used_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    });
    let mut by_resource: HashMap<String, Vec<Recent>> = HashMap::new();
    for entry in entries {
        by_resource
            .entry(entry.resource.clone())
            .or_default()
            .push(entry);
    }
    let entries = by_resource
        .values_mut()
        .flat_map(|values| {
            values.sort_by_key(|entry| std::cmp::Reverse(entry.used_at));
            values.iter().take(RETAIN_PER_RESOURCE).cloned()
        })
        .collect();
    let Ok(contents) = serde_json::to_vec(&RecentFile { entries }) else {
        return;
    };
    let Some(parent) = path.parent() else { return };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(&contents)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
}

pub fn candidates(resource: &str, active_profile: Option<&str>) -> Vec<(String, Option<String>)> {
    if disabled() {
        return Vec::new();
    }
    let Some(path) = cache_path() else {
        return Vec::new();
    };
    candidates_from(load(&path), resource, active_profile)
}

fn candidates_from(
    mut entries: Vec<Recent>,
    resource: &str,
    active_profile: Option<&str>,
) -> Vec<(String, Option<String>)> {
    entries.retain(|entry| entry.resource == resource);
    entries.sort_by(|a, b| {
        (active_profile != Some(a.profile.as_str()))
            .cmp(&(active_profile != Some(b.profile.as_str())))
            .then_with(|| b.used_at.cmp(&a.used_at))
    });
    let mut seen = HashSet::new();
    entries
        .into_iter()
        .filter(|entry| seen.insert(entry.identifier.clone()))
        .map(|entry| {
            let help = (active_profile != Some(entry.profile.as_str()))
                .then(|| format!("{} / {}", entry.profile, entry.account_id));
            (entry.identifier, help)
        })
        .collect()
}

pub fn completer(resource: &'static str) -> ArgValueCompleter {
    ArgValueCompleter::new(move |current: &std::ffi::OsStr| {
        let current = current.to_string_lossy();
        let profile = std::env::var("AWS_PROFILE").ok();
        candidates(resource, profile.as_deref())
            .into_iter()
            .filter(|(value, _)| value.starts_with(current.as_ref()))
            .map(|(value, help)| CompletionCandidate::new(value).help(help.map(Into::into)))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("knife-recents-{}-{name}.json", std::process::id()))
    }

    #[test]
    fn cache_roundtrips_retains_fifty_and_contains_no_secret_fields() {
        let path = temp_path("roundtrip");
        let _ = fs::remove_file(&path);
        for index in 0..55 {
            record_at(&path, "dev", "123", "ec2", &format!("i-{index}"));
        }
        let entries = load(&path);
        assert_eq!(entries.len(), 50);
        let raw = fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("secret"));
        assert!(!raw.contains("token"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let _ = fs::remove_file(path);
    }

    #[test]
    fn malformed_cache_is_ignored_and_recreated() {
        let path = temp_path("malformed");
        fs::write(&path, "not valid json").unwrap();
        assert!(load(&path).is_empty());
        record_at(&path, "dev", "123", "route53", "example.com");
        assert_eq!(load(&path).len(), 1);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn active_profile_sorts_first_and_other_profiles_are_labeled() {
        let entries = vec![
            Recent {
                profile: "other".into(),
                account_id: "222".into(),
                resource: "ec2".into(),
                identifier: "i-other".into(),
                used_at: 20,
            },
            Recent {
                profile: "active".into(),
                account_id: "111".into(),
                resource: "ec2".into(),
                identifier: "i-active".into(),
                used_at: 1,
            },
        ];
        assert_eq!(
            candidates_from(entries, "ec2", Some("active")),
            vec![
                ("i-active".into(), None),
                ("i-other".into(), Some("other / 222".into()))
            ]
        );
    }

    #[test]
    fn completion_callback_filters_prefix_and_returns_entries() {
        let path = temp_path("completion");
        let _ = fs::remove_file(&path);
        record_at(&path, "active", "111", "ec2", "i-active");
        let entries = load(&path);
        let candidates = candidates_from(entries, "ec2", Some("active"));
        let callback_output: Vec<_> = candidates
            .into_iter()
            .filter(|(candidate, _)| candidate.starts_with("i-a"))
            .map(|(candidate, help)| CompletionCandidate::new(candidate).help(help.map(Into::into)))
            .collect();
        assert_eq!(callback_output.len(), 1);
        assert_eq!(callback_output[0].get_value(), "i-active");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn candidates_respect_opt_out_guard() {
        let path = temp_path("opt-out");
        let _ = fs::remove_file(&path);
        record_at(&path, "active", "111", "ec2", "i-active");
        let original_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", path.parent().unwrap()) };
        let cache_file = path.parent().unwrap().join(".cache/knife/recents.json");
        let _ = fs::remove_file(&cache_file);
        record_at(&cache_file, "active", "111", "ec2", "i-active");
        assert_eq!(candidates("ec2", Some("active")).len(), 1);
        unsafe {
            match original_home {
                Some(home) => std::env::set_var("HOME", home),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = fs::remove_file(cache_file);
        let _ = fs::remove_file(path);
    }
}
