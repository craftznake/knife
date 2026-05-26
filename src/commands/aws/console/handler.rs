use std::process::{self};

use dialoguer::FuzzySelect;

use crate::commands::Output;
use crate::commands::aws::arg::GlobalOptions;
use crate::commands::aws::console::arg::{AWSConsoleCommand, ConsoleSubCommand, Login};
use crate::commands::aws::sso::handler::{get_sso_profiles, load_last_profile};

impl AWSConsoleCommand {
    pub async fn execute(self, opts: GlobalOptions) -> Result<Output, Box<dyn std::error::Error>> {
        match self.command {
            ConsoleSubCommand::Login(args) => args.execute(opts.verbose).await,
        }
    }
}

impl Login {
    async fn execute(self, verbose: bool) -> Result<Output, Box<dyn std::error::Error>> {
        let output = Output::new(verbose);
        let profile: Option<String> = if let Some(profile) = self.profile {
            Some(profile)
        } else if let Some(profile) = load_last_profile() {
            Some(profile)
        } else {
            None
        };

        let mut profiles = get_sso_profiles()?;

        let sso_profile = match profile {
            Some(name) => {
                let sso_profile = profiles
                    .iter()
                    .find(|sso_profile| sso_profile.name == name)
                    .ok_or(format!("failed to get sso_profile {name}"))?;
                sso_profile
            }
            None => {
                if profiles.is_empty() {
                    output.stderr("No AWS SSO profiles found in ~/.aws/config");
                    return Err(format!("Error: No AWS SSO profiles found in ~/.aws/config").into());
                }

                // Set up Ctrl-C handler to restore terminal
                // Known issue that haven't solved at upstream.
                // dialoguer try to hide cursor while start the selection, but didn't
                // add any interrupt intercenption.
                // https://github.com/console-rs/dialoguer/issues/77
                if let Err(e) = ctrlc::set_handler(move || {
                    // Restore terminal state
                    let term = console::Term::stdout();
                    let _ = term.show_cursor();
                }) {
                    output.stderr(&format!("failed to setup ctrl_c handler. {e:?}"));
                };

                profiles.sort_by_key(|p| p.name.clone());
                // Interactive selection
                let profile_names: Vec<&str> = profiles.iter().map(|f| f.name.as_str()).collect();
                let selection = FuzzySelect::new()
                    .with_prompt("Select AWS SSO profile")
                    .items(&profile_names)
                    .default(0)
                    .interact_opt();

                match selection {
                    Ok(Some(idx)) => &profiles[idx],

                    Ok(None) | Err(_) => {
                        // Restore cursor visibility on exit
                        output.stderr("\nSelection cancelled");
                        process::exit(1);
                    }
                }
            }
        };

        let _ = open::that(format!(
            "{}#/console?account_id={}&role_name={}",
            sso_profile.start_url, sso_profile.account_id, sso_profile.role_name
        ));
        Ok(output)
    }
}
