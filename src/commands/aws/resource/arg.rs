use super::super::arg::OutputFormat;
use super::super::ec2::arg::{validate_ip_address, validate_state};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum AWSResourceCommand {
    Get(GetResource),
    Describe(DescribeResource),
    Delete(DeleteResource),
    Scale(ScaleResource),
    Attach(AttachResource),
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
#[derive(Debug, Subcommand)]
pub enum ResourceVerb {
    Get(GetResource),
    Describe(DescribeResource),
    Delete(DeleteResource),
    Scale(ScaleResource),
    Attach(AttachResource),
    Detach(DetachResource),
}
#[derive(Debug, Args)]
pub struct GetResource {
    #[command(subcommand)]
    pub resource: GetKind,
}
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GetKind {
    /// EC2
    #[command(name = "ec2")]
    Ec2(Ec2Get),
    /// ELB
    #[command(name = "elb")]
    Elb(ElbGet),
    /// ELB listener
    #[command(name = "elb-listener")]
    ElbListener(ElbListenerGet),
    /// Elb Listener rule
    #[command(name = "elb-listener-rule")]
    ElbRule(ElbRulesGet),
    /// ASG
    #[command(name = "asg")]
    Asg(AsgGet),
    /// Route53
    #[command(name = "route53")]
    Route53(Route53Get),
}

#[derive(Debug, Args)]
pub struct Ec2Get {
    #[arg(value_name = "IDENTIFIER", required_unless_present_any = ["selector", "state", "private_ip", "public_ip"], add = crate::recents::completer("ec2"))]
    pub identifier: Option<String>,
    #[arg(long="selector", value_delimiter=',', value_parser=parse_selector, num_args=1..)]
    pub selector: Option<Vec<String>>,
    #[arg(long, value_parser=validate_state, num_args=1..)]
    pub state: Vec<String>,
    #[arg(long, value_parser=validate_ip_address)]
    pub private_ip: Option<String>,
    #[arg(long, value_parser=validate_ip_address)]
    pub public_ip: Option<String>,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct ElbGet {
    #[arg(value_name = "NAME", add = crate::recents::completer("elb"))]
    pub name: Option<String>,
    #[arg(long)]
    pub limit: Option<i8>,
    #[arg(long)]
    pub exact: bool,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct ElbListenerGet {
    #[arg(value_name = "LOAD-BALANCER-ARN", add = crate::recents::completer("elb-listeners"))]
    pub arn: String,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct ElbRulesGet {
    #[arg(value_name = "LISTENER-ARN", add = crate::recents::completer("elb-rules"))]
    pub arn: String,
    #[arg(short='l',long="selector",value_delimiter=',',value_parser=parse_selector, num_args=1..)]
    pub selector: Option<Vec<String>>,
    #[arg(long)]
    pub limit: Option<i8>,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct AsgGet {
    #[arg(value_name = "NAME", add = crate::recents::completer("asg"))]
    pub name: String,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct Route53Get {
    #[arg(value_name = "DOMAIN", add = crate::recents::completer("route53"))]
    pub domain: String,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct DescribeResource {
    #[command(subcommand)]
    pub resource: DescribeKind,
}
#[derive(Debug, Subcommand)]
pub enum DescribeKind {
    /// EC2
    #[command(name = "ec2")]
    Ec2(Ec2Describe),
}
#[derive(Debug, Args)]
pub struct Ec2Describe {
    /// Instance ID
    #[arg(value_name = "ID")]
    pub id: String,
    /// Output type
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct DeleteResource {
    /// EC2
    #[command(subcommand)]
    pub resource: DeleteKind,
}
#[derive(Debug, Subcommand)]
pub enum DeleteKind {
    /// EC2
    #[command(name = "ec2")]
    Ec2(Ec2Delete),
}
#[derive(Debug, Args)]
pub struct Ec2Delete {
    #[arg(value_name = "ID")]
    pub id: String,
    #[arg(long)]
    pub yes: bool,
}
#[derive(Debug, Args)]
pub struct ScaleResource {
    #[command(subcommand)]
    pub resource: ScaleKind,
}
#[derive(Debug, Subcommand)]
pub enum ScaleKind {
    /// ASG
    #[command(name = "asg")]
    Asg(AsgScale),
}
#[derive(Debug, Args)]
pub struct AsgScale {
    #[arg(value_name = "NAME", add = crate::recents::completer("asg"))]
    pub name: String,
    #[arg(long = "min")]
    pub min_size: Option<i32>,
    #[arg(long = "max")]
    pub max_size: Option<i32>,
    #[arg(long = "desired")]
    pub desired_capacity: Option<i32>,
    #[arg(long)]
    pub yes: bool,
}
#[derive(Debug, Args)]
pub struct AttachResource {
    #[command(subcommand)]
    pub resource: AttachKind,
}
#[derive(Debug, Subcommand)]
pub enum AttachKind {
    /// ASG
    #[command(name = "asg")]
    Asg(AsgAttach),
}
#[derive(Debug, Args)]
pub struct AsgAttach {
    #[arg(value_name = "NAME", add = crate::recents::completer("asg"))]
    pub name: String,
    #[arg(value_name="INSTANCE-ID",required=true,num_args=1..)]
    pub ids: Vec<String>,
}
#[derive(Debug, Args)]
pub struct DetachResource {
    #[command(subcommand)]
    pub resource: DetachKind,
}
#[derive(Debug, Subcommand)]
pub enum DetachKind {
    /// ASG
    #[command(name = "asg")]
    Asg(AsgDetach),
}
#[derive(Debug, Args)]
pub struct AsgDetach {
    #[arg(value_name = "NAME", add = crate::recents::completer("asg"))]
    pub name: String,
    #[arg(value_name="INSTANCE-ID",required=true,num_args=1..)]
    pub ids: Vec<String>,
    #[arg(long)]
    pub replace: bool,
    #[arg(long)]
    pub yes: bool,
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
