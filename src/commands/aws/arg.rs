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
    #[arg(long, short = 'r', global = true)]
    pub region: Option<String>,
    #[arg(long, short = 'p', global = true)]
    pub profile: Option<String>,
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,
    #[arg(long, global = true)]
    pub debug: bool,
    #[command(subcommand)]
    pub command: AWSSubCommand,

    #[arg(skip=OutputFormat::Json)]
    pub output_format: OutputFormat,
    #[arg(skip)]
    pub handler_command: Option<(AWSHandlerCommand, OutputFormat)>,
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

    /// Whoami describe the current logged in user.
    #[command(name = "whoami")]
    Whoami(AWSWhoAmICommand),

    /// Session Manager
    #[command(name = "ssm")]
    SSM(AWSSSMCommand),

    /// Login AWS using sso profile
    #[command(name = "login")]
    Login(AWSLoginCommand),

    /// Logout current SSO session
    #[command(name = "logout")]
    Logout(AWSLogoutCommand),

    /// Web Console
    #[command(name = "console")]
    Console(AWSConsoleCommand),
}
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Yaml,
    Table,
    Wide,
}
