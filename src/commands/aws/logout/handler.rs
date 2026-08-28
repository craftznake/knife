use std::{fs, path::PathBuf, process::Command};

use crate::commands::Output;
use crate::commands::aws::logout::arg::AWSLogoutCommand;

impl AWSLogoutCommand {
    pub async fn execute(self, verbose: bool) -> Result<Output, Box<dyn std::error::Error>> {
        let output = Output::new(verbose);

        // Perform SSO logout
        match sso_logout(self.profile.as_deref()).await {
            Ok(_) => {
                output.stderr("SSO logout completed successfully");
                Ok(output)
            }
            Err(e) => {
                output.stderr(&format!("SSO logout failed: {}", e));
                Err(e)
            }
        }
    }
}

async fn sso_logout(profile: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // Execute aws sso login command
    let mut cmd = Command::new("aws");
    cmd.arg("sso").arg("logout");
    if let Some(p) = profile {
        cmd.arg("--profile").arg(p);
    };
    match cmd.status() {
        Ok(exit_status) => {
            if !exit_status.success() {
                return Err(format!("SSO logout failed with code: {}", exit_status).into());
            }

            // Save the profile and region for future knife commands
            if let Err(e) = clean_last_session() {
                return Err(format!("Warning: Failed to save session preferences: {}", e).into());
            }

            Ok(())
        }
        Err(e) => Err(format!("Failed to login SSO: {}", e).into()),
    }
}

fn clean_last_session() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir().map_err(|e| -> Box<dyn std::error::Error> {
        format!("failed to get config directory: {}", e).into()
    })?;
    let shell_config_dir = config_dir.join("shell");
    if shell_config_dir.is_dir() {
        for shell_config in fs::read_dir(shell_config_dir)? {
            let shell_config = shell_config?;
            let path = shell_config.path();
            if path.is_file() {
                fs::write(path, "")?;
            };
        }
    };
    Ok(())
}

/// Get the knife config directory path (~/.knife)
fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").map_err(|_| -> Box<dyn std::error::Error> {
        format!("HOME environment variable not set").into()
    })?;

    let config_dir = PathBuf::from(home).join(".knife");

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| -> Box<dyn std::error::Error> {
            format!("Failed to create config directory: {}", e).into()
        })?;
    }

    Ok(config_dir)
}
