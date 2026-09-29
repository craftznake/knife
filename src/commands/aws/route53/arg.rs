use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct AWSRoute53Command {
    #[command(subcommand)]
    pub command: Route53SubCommand,
}

#[derive(Debug, Subcommand)]
pub enum Route53SubCommand {
    /// Get domain record configuration
    #[command(name = "get", hide = true)]
    Get {
        #[arg(value_name = "DOMAIN")]
        domain: String,
    },
}
