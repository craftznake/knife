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
#[group(required = true, multiple = false)]
pub struct StartArg {
    /// Instance ID to start a session with.
    #[arg(value_name = "INSTANCE-ID", required_unless_present = "legacy_id")]
    pub id: Option<String>,

    /// Legacy --id form; stdin sentinel is not supported.
    #[arg(long = "id", hide = true, required_unless_present = "id")]
    pub legacy_id: Option<String>,
}

impl StartArg {
    pub fn instance_id(&self) -> &str {
        self.id
            .as_deref()
            .or(self.legacy_id.as_deref())
            .unwrap_or_default()
    }
}
