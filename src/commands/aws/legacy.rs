use super::{
    asg::arg::{ASGSubCommand, AWSASGCommand, AttachInstancesArg, DetachInstancesArg, ScaleArg},
    ec2::arg::{AWSEC2Command, EC2SubCommand, SearchArg, TerminateArg},
};
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct LegacyASG {
    #[arg(long = "name", required = true)]
    pub name: String,
    #[command(subcommand)]
    pub command: LegacyASGCommand,
}
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum LegacyASGCommand {
    Get,
    Scale {
        #[arg(long)]
        min: Option<i32>,
        #[arg(long)]
        max: Option<i32>,
        #[arg(long)]
        desired: Option<i32>,
        #[arg(long)]
        yes: bool,
    },
    AttachInstances {
        #[arg(long = "ids", required = true, num_args = 1..)]
        ids: Vec<String>,
    },
    DetachInstances {
        #[arg(long = "ids", required = true, num_args = 1..)]
        ids: Vec<String>,
        #[arg(long)]
        replace: bool,
        #[arg(long)]
        yes: bool,
    },
}
pub fn normalize_asg(value: LegacyASG) -> AWSASGCommand {
    let command = match value.command {
        LegacyASGCommand::Get => ASGSubCommand::Get,
        LegacyASGCommand::Scale {
            min,
            max,
            desired,
            yes,
        } => ASGSubCommand::Scale(ScaleArg {
            min_size: min,
            max_size: max,
            desired_capacity: desired,
            yes,
        }),
        LegacyASGCommand::AttachInstances { ids } => {
            ASGSubCommand::AttachInstances(AttachInstancesArg { ids })
        }
        LegacyASGCommand::DetachInstances { ids, replace, yes } => {
            ASGSubCommand::DetachInstances(DetachInstancesArg { ids, replace, yes })
        }
    };
    AWSASGCommand {
        command,
        name: value.name,
    }
}

#[derive(Debug, Args)]
pub struct LegacyEC2 {
    #[command(subcommand)]
    pub command: LegacyEC2Command,
}
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum LegacyEC2Command {
    Get(SearchArg),
    Terminate(TerminateArg),
}
#[allow(dead_code)]
pub fn normalize_ec2(value: LegacyEC2) -> AWSEC2Command {
    AWSEC2Command {
        command: match value.command {
            LegacyEC2Command::Get(args) => EC2SubCommand::Get(args),
            LegacyEC2Command::Terminate(args) => EC2SubCommand::Terminate(args),
        },
    }
}
