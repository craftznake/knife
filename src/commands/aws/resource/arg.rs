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
    #[command(name = "ec2")]
    Ec2(Ec2Get),
    #[command(name = "elb")]
    Elb(ElbGet),
    #[command(name = "elb-listeners")]
    ElbListeners(ElbListenersGet),
    #[command(name = "elb-rules")]
    ElbRules(ElbRulesGet),
    #[command(name = "asg")]
    Asg(AsgGet),
    #[command(name = "route53")]
    Route53(Route53Get),
}

#[derive(Debug, Args)]
pub struct Ec2Get {
    #[arg(value_name = "IDENTIFIER", required_unless_present_any = ["selector", "state", "private_ip", "public_ip"])]
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
    #[arg(value_name = "NAME")]
    pub name: Option<String>,
    #[arg(long)]
    pub limit: Option<i8>,
    #[arg(long)]
    pub exact: bool,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct ElbListenersGet {
    #[arg(value_name = "LOAD-BALANCER-ARN")]
    pub arn: String,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct ElbRulesGet {
    #[arg(value_name = "LISTENER-ARN")]
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
    #[arg(value_name = "NAME")]
    pub name: String,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct Route53Get {
    #[arg(value_name = "DOMAIN")]
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
    #[command(name = "ec2")]
    Ec2(Ec2Describe),
}
#[derive(Debug, Args)]
pub struct Ec2Describe {
    #[arg(value_name = "ID")]
    pub id: String,
    #[arg(short = 'o', long = "output", value_enum, default_value = "json")]
    pub output: OutputFormat,
}
#[derive(Debug, Args)]
pub struct DeleteResource {
    #[command(subcommand)]
    pub resource: DeleteKind,
}
#[derive(Debug, Subcommand)]
pub enum DeleteKind {
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
    #[command(name = "asg")]
    Asg(AsgScale),
}
#[derive(Debug, Args)]
pub struct AsgScale {
    #[arg(value_name = "NAME")]
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
    #[command(name = "asg")]
    Asg(AsgAttach),
}
#[derive(Debug, Args)]
pub struct AsgAttach {
    #[arg(value_name = "NAME")]
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
    #[command(name = "asg")]
    Asg(AsgDetach),
}
#[derive(Debug, Args)]
pub struct AsgDetach {
    #[arg(value_name = "NAME")]
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
