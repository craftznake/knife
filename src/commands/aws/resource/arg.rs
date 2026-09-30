pub use super::super::{
    asg::arg::{AsgAttach, AsgDetach, AsgGet, AsgScale},
    ec2::arg::{Ec2Delete, Ec2Describe, Ec2Get},
    elb::arg::{ElbGet, ElbListenerGet, ElbRulesGet},
    route53::arg::Route53Get,
};
use clap::{Args, Subcommand};

/// AWS resource operations.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum AWSResourceCommand {
    /// Read an AWS resource.
    Get(GetResource),
    /// Describe an AWS resource.
    Describe(DescribeResource),
    /// Delete an AWS resource.
    Delete(DeleteResource),
    /// Scale an AWS resource.
    Scale(ScaleResource),
    /// Attach resources to an AWS resource.
    Attach(AttachResource),
    /// Detach resources from an AWS resource.
    Detach(DetachResource),
}
impl AWSResourceCommand {
    pub fn from_verb(value: ResourceVerb) -> Self {
        match value {
            ResourceVerb::Get(v) => Self::Get(v),
            ResourceVerb::Describe(v) => Self::Describe(v),
            ResourceVerb::Delete(v) => Self::Delete(v),
            ResourceVerb::Scale(v) => Self::Scale(v),
            ResourceVerb::Attach(v) => Self::Attach(v),
            ResourceVerb::Detach(v) => Self::Detach(v),
        }
    }
    pub fn into_verb(self) -> ResourceVerb {
        match self {
            Self::Get(v) => ResourceVerb::Get(v),
            Self::Describe(v) => ResourceVerb::Describe(v),
            Self::Delete(v) => ResourceVerb::Delete(v),
            Self::Scale(v) => ResourceVerb::Scale(v),
            Self::Attach(v) => ResourceVerb::Attach(v),
            Self::Detach(v) => ResourceVerb::Detach(v),
        }
    }
}
/// An AWS resource operation verb.
#[derive(Debug, Subcommand)]
pub enum ResourceVerb {
    /// Read an AWS resource.
    Get(GetResource),
    /// Describe an AWS resource.
    Describe(DescribeResource),
    /// Delete an AWS resource.
    Delete(DeleteResource),
    /// Scale an AWS resource.
    Scale(ScaleResource),
    /// Attach resources to an AWS resource.
    Attach(AttachResource),
    /// Detach resources from an AWS resource.
    Detach(DetachResource),
}
/// Read an AWS resource.
#[derive(Debug, Args)]
pub struct GetResource {
    /// Resource type to read.
    #[command(subcommand)]
    pub resource: GetKind,
}
/// Select an AWS resource to read.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GetKind {
    /// Find EC2 instances.
    #[command(name = "ec2")]
    Ec2(Ec2Get),
    /// Find load balancers.
    #[command(name = "elb")]
    Elb(ElbGet),
    /// List load balancer listeners.
    #[command(name = "elb-listeners")]
    ElbListener(ElbListenerGet),
    /// List listener rules.
    #[command(name = "elb-rules")]
    ElbRule(ElbRulesGet),
    /// Get an Auto Scaling group.
    #[command(name = "asg")]
    Asg(AsgGet),
    /// Get Route 53 records.
    #[command(name = "route53")]
    Route53(Route53Get),
}

/// Describe an AWS resource.
#[derive(Debug, Args)]
pub struct DescribeResource {
    /// Resource type to describe.
    #[command(subcommand)]
    pub resource: DescribeKind,
}
/// Select an AWS resource type to describe.
#[derive(Debug, Subcommand)]
pub enum DescribeKind {
    /// Describe an EC2 instance.
    #[command(name = "ec2")]
    Ec2(Ec2Describe),
}
/// Delete an AWS resource.
#[derive(Debug, Args)]
pub struct DeleteResource {
    /// Resource type to delete.
    #[command(subcommand)]
    pub resource: DeleteKind,
}
/// Select an AWS resource type to delete.
#[derive(Debug, Subcommand)]
pub enum DeleteKind {
    /// Delete an EC2 instance.
    #[command(name = "ec2")]
    Ec2(Ec2Delete),
}
/// Scale an AWS resource.
#[derive(Debug, Args)]
pub struct ScaleResource {
    /// Resource type to scale.
    #[command(subcommand)]
    pub resource: ScaleKind,
}
/// Select an AWS resource type to scale.
#[derive(Debug, Subcommand)]
pub enum ScaleKind {
    /// Scale an Auto Scaling group.
    #[command(name = "asg")]
    Asg(AsgScale),
}
/// Attach resources to an AWS resource.
#[derive(Debug, Args)]
pub struct AttachResource {
    /// Resource type to attach to.
    #[command(subcommand)]
    pub resource: AttachKind,
}
/// Select an AWS resource type to attach to.
#[derive(Debug, Subcommand)]
pub enum AttachKind {
    /// Attach instances to an Auto Scaling group.
    #[command(name = "asg")]
    Asg(AsgAttach),
}
/// Detach resources from an AWS resource.
#[derive(Debug, Args)]
pub struct DetachResource {
    /// Resource type to detach from.
    #[command(subcommand)]
    pub resource: DetachKind,
}
/// Select an AWS resource type to detach from.
#[derive(Debug, Subcommand)]
pub enum DetachKind {
    /// Detach instances from an Auto Scaling group.
    #[command(name = "asg")]
    Asg(AsgDetach),
}
