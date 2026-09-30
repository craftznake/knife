use clap::{Args, Subcommand};

/// Operate on an Auto Scaling group.
#[derive(Debug, Args)]
pub struct AWSASGCommand {
    /// Auto Scaling operation to execute.
    #[command(subcommand)]
    pub command: ASGSubCommand,
    /// Auto Scaling group name.
    #[arg(long = "name", required = true)]
    pub name: String,
}

/// Auto Scaling operations.
#[derive(Debug, Subcommand)]
pub enum ASGSubCommand {
    /// Get ASG configuration.
    Get,
    /// Scale an Auto Scaling group.
    Scale(ScaleArg),
    /// Detach instances from an Auto Scaling group.
    DetachInstances(DetachInstancesArg),
    /// Attach instances to an Auto Scaling group.
    AttachInstances(AttachInstancesArg),
}

/// Set the capacity of an Auto Scaling group.
#[derive(Debug, Args)]
pub struct ScaleArg {
    /// Minimum group size.
    #[arg(long = "min")]
    pub min_size: Option<i32>,

    /// Maximum group size.
    #[arg(long = "max")]
    pub max_size: Option<i32>,

    /// Desired group capacity.
    #[arg(long = "desired")]
    pub desired_capacity: Option<i32>,

    /// Skip confirmation prompt
    #[arg(long)]
    pub yes: bool,
}

/// Detach instances from an Auto Scaling group.
#[derive(Debug, Args)]
pub struct DetachInstancesArg {
    /// One or more instance IDs to detach.
    #[arg(long,  num_args = 1..)]
    pub ids: Vec<String>,

    /// Replace each detached instance to maintain group capacity.
    #[arg(long)]
    pub replace: bool,

    /// Skip confirmation prompt
    #[arg(long)]
    pub yes: bool,
}

/// Attach instances to an Auto Scaling group.
#[derive(Debug, Args)]
pub struct AttachInstancesArg {
    /// One or more instance IDs to attach.
    #[arg(long,  num_args = 1..)]
    pub ids: Vec<String>,
}

/// Get an Auto Scaling group configuration.
#[derive(Debug, Args)]
pub struct AsgGet {
    /// Auto Scaling group name.
    #[arg(value_name = "NAME", add = crate::recents::completer("asg"))]
    pub name: String,
}

/// Set the capacity of an Auto Scaling group.
#[derive(Debug, Args)]
pub struct AsgScale {
    /// Auto Scaling group name.
    #[arg(value_name = "NAME", add = crate::recents::completer("asg"))]
    pub name: String,
    /// Minimum group size.
    #[arg(long = "min")]
    pub min_size: Option<i32>,
    /// Maximum group size.
    #[arg(long = "max")]
    pub max_size: Option<i32>,
    /// Desired group capacity.
    #[arg(long = "desired")]
    pub desired_capacity: Option<i32>,
    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

/// Attach instances to an Auto Scaling group.
#[derive(Debug, Args)]
pub struct AsgAttach {
    /// Auto Scaling group name.
    #[arg(value_name = "NAME", add = crate::recents::completer("asg"))]
    pub name: String,
    /// One or more instance IDs to attach.
    #[arg(value_name="INSTANCE-ID",required=true,num_args=1..)]
    pub ids: Vec<String>,
}

/// Detach instances from an Auto Scaling group.
#[derive(Debug, Args)]
pub struct AsgDetach {
    /// Auto Scaling group name.
    #[arg(value_name = "NAME", add = crate::recents::completer("asg"))]
    pub name: String,
    /// One or more instance IDs to detach.
    #[arg(value_name="INSTANCE-ID",required=true,num_args=1..)]
    pub ids: Vec<String>,
    /// Replace each detached instance to maintain group capacity.
    #[arg(long)]
    pub replace: bool,
    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}
