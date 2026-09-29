use clap::{Args, Subcommand};

use crate::commands::utils;
#[derive(Debug, Args)]
pub struct AWSElbCommand {
    #[command(subcommand)]
    pub command: ElbSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum ElbSubCommand {
    /// Get command - Get load balancer using name
    #[command(name = "get", hide = true)]
    Get(GetArg),
    /// get-listener - Get load balancer's listener using lb arn
    #[command(name = "get-listeners", hide = true)]
    GetListeners(GetListenersArg),
    /// get-rules - Get listener's rules using listener_arn
    #[command(name = "get-rules", hide = true)]
    GetRules(GetRulesArg),
}

#[derive(Debug, Args)]
pub struct GetArg {
    /// Name of ALBs, knife will perform full text search on ALB names
    #[arg(long)]
    pub name: String,
    /// Number of records should be returned.
    #[arg(long)]
    pub num: Option<i8>,
    /// perform fuzzy search using the full name
    #[arg(long = "fuzzy", default_value_t = true)]
    pub fuzzy: bool,
}

#[derive(Debug, Args)]
pub struct GetRulesArg {
    /// ARN of Listener which contains the rules
    #[arg(long = "arn", required = true)]
    #[clap(value_parser = utils::token_or_stdin_parser)]
    pub listener_arn: String,

    /// Number of records should be returned.
    #[arg(long)]
    pub num: Option<i8>,

    /// Space separated key value pair of tag that will be used to filter rule
    #[arg(long, value_delimiter = ' ', num_args = 2, value_names = ["KEY", "VALUE"])]
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct GetListenersArg {
    /// Load balancer ARN which contains this listener
    #[arg(long = "arn", required = true)]
    #[clap(value_parser = utils::token_or_stdin_parser)]
    pub loadbalancer_arn: String,
}
