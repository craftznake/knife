use clap::{Args, Subcommand};

/// Operate on Route 53 resources.
#[derive(Debug, Args)]
pub struct AWSRoute53Command {
    /// Route 53 operation to execute.
    #[command(subcommand)]
    pub command: Route53SubCommand,
}

/// Route 53 operations.
#[derive(Debug, Subcommand)]
pub enum Route53SubCommand {
    /// Get DNS records for a domain.
    Get {
        /// Domain name to look up.
        #[arg(value_name = "DOMAIN")]
        domain: String,
    },
}

/// Get records for a Route 53 domain.
#[derive(Debug, Args)]
pub struct Route53Get {
    /// Domain name to look up.
    #[arg(value_name = "DOMAIN", add = crate::recents::completer("route53"))]
    pub domain: String,
}
