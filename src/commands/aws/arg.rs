use crate::commands::aws::{
    asg::arg::AWSASGCommand,
    console::arg::AWSConsoleCommand,
    ec2::arg::AWSEC2Command,
    elb::arg::AWSElbCommand,
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
    /// Output format for AWS resource operations.
    #[arg(
        long = "output",
        short = 'o',
        value_enum,
        global = true,
        default_value = "json"
    )]
    pub output_format: OutputFormat,
    /// AWS region context for AWS service operations.
    #[arg(long, short = 'r', global = true)]
    pub region: Option<String>,
    /// AWS profile used for authentication.
    #[arg(long, short = 'p', global = true)]
    pub profile: Option<String>,
    /// Enable verbose logging.
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,
    /// Enable debug logging.
    #[arg(long, global = true)]
    pub debug: bool,
    /// AWS operation to execute.
    #[command(subcommand)]
    pub command: AWSSubCommand,

    #[arg(skip)]
    pub handler_command: Option<(AWSHandlerCommand, OutputFormat)>,
}

/// AWS top-level operations.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum AWSSubCommand {
    /// Read an AWS resource.
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

    /// Describe the currently authenticated user.
    #[command(name = "whoami")]
    Whoami(AWSWhoAmICommand),

    /// Start an AWS Systems Manager session.
    #[command(name = "ssm")]
    SSM(AWSSSMCommand),

    /// Log in to AWS with an SSO profile.
    #[command(name = "login")]
    Login(AWSLoginCommand),

    /// Log out of the current SSO session.
    #[command(name = "logout")]
    Logout(AWSLogoutCommand),

    /// Open the AWS web console.
    #[command(name = "console")]
    Console(AWSConsoleCommand),
}
/// Internal AWS command dispatched to a service handler.
#[derive(Debug)]
pub enum AWSHandlerCommand {
    Ec2(AWSEC2Command),
    Elb(AWSElbCommand),
    Route53(AWSRoute53Command),
    Asg(AWSASGCommand),
    Whoami(AWSWhoAmICommand),
    SSM(AWSSSMCommand),
    Login(AWSLoginCommand),
    Logout(AWSLogoutCommand),
    Console(AWSConsoleCommand),
}

impl From<AWSSubCommand> for AWSHandlerCommand {
    fn from(value: AWSSubCommand) -> Self {
        match value {
            AWSSubCommand::Get(_)
            | AWSSubCommand::Describe(_)
            | AWSSubCommand::Delete(_)
            | AWSSubCommand::Scale(_)
            | AWSSubCommand::Attach(_)
            | AWSSubCommand::Detach(_) => {
                unreachable!("resource commands are converted before dispatch")
            }
            AWSSubCommand::Whoami(v) => Self::Whoami(v),
            AWSSubCommand::SSM(v) => Self::SSM(v),
            AWSSubCommand::Login(v) => Self::Login(v),
            AWSSubCommand::Logout(v) => Self::Logout(v),
            AWSSubCommand::Console(v) => Self::Console(v),
        }
    }
}
#[derive(Debug)]
pub struct GlobalOptions {
    pub sdk_config: SdkConfig,
    pub verbose: bool,
    pub output_format: OutputFormat,
}
/// Output encoding used for AWS resource responses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Yaml,
    Table,
    Wide,
}
