use crate::commands::aws::{
    console::arg::AWSConsoleCommand,
    ec2::arg::AWSEC2Command,
    elb::arg::AWSElbCommand,
    legacy::LegacyASG,
    login::arg::AWSLoginCommand,
    logout::arg::AWSLogoutCommand,
    resource::arg::{
        AttachResource, DeleteResource, DescribeResource, DetachResource, GetResource,
        ScaleResource,
    },
    route53::arg::AWSRoute53Command,
    ssm::arg::AWSSSMCommand,
    whoami::arg::AWSWhoAmICommand,
};
use aws_config::SdkConfig;
use clap::{Args, Subcommand, ValueEnum};
#[derive(Debug, Args)]
pub struct AWSCommand {
    #[arg(long, short = 'r', global = true, allow_hyphen_values = true)]
    pub region: Option<String>,
    #[arg(skip)]
    pub root_region: Option<String>,
    #[arg(long, short = 'p', global = true, allow_hyphen_values = true)]
    pub profile: Option<String>,
    #[arg(skip)]
    pub root_profile: Option<String>,
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,
    #[arg(long, global = true)]
    pub debug: bool,
    #[arg(skip=OutputFormat::Json)]
    pub output_format: OutputFormat,
    #[command(subcommand)]
    pub command: AWSSubCommand,
}
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum AWSSubCommand {
    /// Read AWS resources.
    #[command(name = "get")]
    Get(GetResource),
    /// Describe an AWS resource.
    #[command(name = "describe")]
    Describe(DescribeResource),
    /// Delete an AWS resource.
    #[command(name = "delete")]
    Delete(DeleteResource),
    /// Scale an Auto Scaling group.
    #[command(name = "scale")]
    Scale(ScaleResource),

    /// Attach instances to an Auto Scaling group.
    #[command(name = "attach")]
    Attach(AttachResource),
    /// Detach instances from an Auto Scaling group.
    #[command(name = "detach")]
    Detach(DetachResource),
    #[command(name = "elb", hide = true)]
    LegacyElb(AWSElbCommand),
    #[command(name = "whoami")]
    Whoami(AWSWhoAmICommand),
    #[command(name = "route53", hide = true)]
    LegacyRoute53(AWSRoute53Command),
    #[command(name = "ec2", hide = true)]
    EC2Compat(AWSEC2Command),
    #[command(name = "asg", hide = true)]
    LegacyASGCompat(LegacyASG),
    #[command(name = "ssm")]
    SSM(AWSSSMCommand),
    #[command(name = "login")]
    Login(AWSLoginCommand),
    #[command(name = "logout")]
    Logout(AWSLogoutCommand),
    #[command(name = "console")]
    Console(AWSConsoleCommand),
}
#[derive(Debug)]
pub struct GlobalOptions {
    pub sdk_config: SdkConfig,
    pub verbose: bool,
    pub output_format: OutputFormat,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Yaml,
    Table,
    Wide,
}
