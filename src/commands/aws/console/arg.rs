use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct AWSConsoleCommand {
    #[command(subcommand)]
    pub command: ConsoleSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConsoleSubCommand {
    /// redirect to the console login page
    Login(Login),
}

#[derive(Debug, Args)]
pub struct Login {
    // Profile name (if not provided, will show derrived the current logged profile)
    #[arg(long)]
    pub profile: Option<String>,
}
