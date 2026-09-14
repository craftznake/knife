use clap::Args;

#[derive(Debug, Args)]
pub struct AWSLoginCommand {
    /// Profile name (if not provided, will show interactive selection)
    #[arg(long)]
    pub profile: Option<String>,
}
