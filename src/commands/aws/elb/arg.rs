use clap::{Args, Subcommand};

use crate::commands::utils;
/// Operate on Elastic Load Balancing resources.
#[derive(Debug, Args)]
pub struct AWSElbCommand {
    /// Load balancer operation to execute.
    #[command(subcommand)]
    pub command: ElbSubCommand,
}

/// Load balancer operations.
#[derive(Debug, Subcommand)]
pub enum ElbSubCommand {
    /// Find load balancers by name.
    Get(GetArg),
    /// Get listeners for a load balancer.
    GetListeners(GetListenersArg),
    /// Get rules for a load balancer listener.
    GetRules(GetRulesArg),
}

/// Get load balancers by name.
#[derive(Debug, Args)]
pub struct GetArg {
    /// Load balancer name to search for.
    #[arg(long)]
    pub name: String,
    /// Maximum number of results.
    #[arg(long)]
    pub num: Option<i8>,
    /// Enable fuzzy matching for the load balancer name.
    #[arg(long = "fuzzy", default_value_t = true)]
    pub fuzzy: bool,
}

/// Get rules for a load balancer listener.
#[derive(Debug, Args)]
pub struct GetRulesArg {
    /// ARN of the listener whose rules should be returned.
    #[arg(long = "arn", required = true)]
    #[clap(value_parser = utils::token_or_stdin_parser)]
    pub listener_arn: String,

    /// Maximum number of results.
    #[arg(long)]
    pub num: Option<i8>,

    /// Filter listener rules by a tag KEY and VALUE pair.
    #[arg(long, value_delimiter = ' ', num_args = 2, value_names = ["KEY", "VALUE"])]
    pub tag: Option<Vec<String>>,
}

/// Get listeners for a load balancer.
#[derive(Debug, Args)]
pub struct GetListenersArg {
    /// ARN of the load balancer whose listeners should be returned.
    #[arg(long = "arn", required = true)]
    #[clap(value_parser = utils::token_or_stdin_parser)]
    pub loadbalancer_arn: String,
}

/// Get a load balancer by name.
#[derive(Debug, Args)]
pub struct ElbGet {
    /// Load balancer name to search for.
    #[arg(value_name = "NAME", add = crate::recents::completer("elb"))]
    pub name: Option<String>,
    /// Maximum number of results.
    #[arg(long)]
    pub limit: Option<i8>,
    /// Match the name exactly.
    #[arg(long)]
    pub exact: bool,
}

/// Get listeners for a load balancer.
#[derive(Debug, Args)]
pub struct ElbListenerGet {
    /// Load balancer ARN.
    #[arg(value_name = "LOAD-BALANCER-ARN", add = crate::recents::completer("elb-listeners"))]
    pub arn: String,
}

/// Get rules for a listener.
#[derive(Debug, Args)]
pub struct ElbRulesGet {
    /// Listener ARN.
    #[arg(value_name = "LISTENER-ARN", add = crate::recents::completer("elb-rules"))]
    pub arn: String,
    /// Filter rules with comma-separated KEY=VALUE selectors.
    #[arg(short='l',long="selector",value_delimiter=',',value_parser=parse_selector, num_args=1..)]
    pub selector: Option<Vec<String>>,
    /// Maximum number of results.
    #[arg(long)]
    pub limit: Option<i8>,
}

fn parse_selector(value: &str) -> Result<String, String> {
    let (key, val) = value
        .split_once('=')
        .ok_or_else(|| format!("selector must be KEY=VALUE: {value}"))?;
    if key.is_empty() || val.is_empty() {
        Err(format!("selector must be KEY=VALUE: {value}"))
    } else {
        Ok(value.to_owned())
    }
}
