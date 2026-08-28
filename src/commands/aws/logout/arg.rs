use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct AWSLogoutCommand {
    /// Profile name (if not provided, will logout all profile)
    #[arg(long)]
    pub profile: Option<String>,
}
