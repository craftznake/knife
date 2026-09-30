use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct AWSSSMCommand {
    #[command(subcommand)]
    pub command: SSMSubCommand,
}

#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum SSMSubCommand {
    /// Start an SSM session.
    #[command(name = "start")]
    Start(StartArg),
}

#[derive(Debug, Args)]
pub struct StartArg {
    /// Instance ID to start a session with.
    #[arg(value_name = "INSTANCE-ID", add = crate::recents::completer("ec2"))]
    pub id: String,
}
