use super::{
    super::{
        arg::{AWSCommand, AWSHandlerCommand, AWSSubCommand, OutputFormat},
        asg::arg::{
            ASGSubCommand, AWSASGCommand, AttachInstancesArg, DetachInstancesArg, ScaleArg,
        },
        ec2::arg::{AWSEC2Command, DescribeArg, EC2SubCommand, TerminateArg},
        elb::arg::{AWSElbCommand, ElbSubCommand, GetArg, GetListenersArg, GetRulesArg},
        route53::arg::{AWSRoute53Command, Route53SubCommand},
    },
    arg::*,
};
impl AWSResourceCommand {
    pub fn into_handler(self) -> Result<(AWSHandlerCommand, OutputFormat), String> {
        let (command, format) = match self.into_verb() {
            ResourceVerb::Get(get) => match get.resource {
                GetKind::Ec2(args) => {
                    let output = args.output;
                    let search = args.normalize()?;
                    (
                        AWSHandlerCommand::Ec2(AWSEC2Command {
                            command: EC2SubCommand::Get(search.normalize()?),
                        }),
                        output,
                    )
                }
                GetKind::Elb(args) => (
                    AWSHandlerCommand::Elb(AWSElbCommand {
                        command: ElbSubCommand::Get(GetArg {
                            name: args.name.unwrap_or_default(),
                            num: args.limit,
                            fuzzy: !args.exact,
                        }),
                    }),
                    args.output,
                ),
                GetKind::ElbListeners(args) => (
                    AWSHandlerCommand::Elb(AWSElbCommand {
                        command: ElbSubCommand::GetListeners(GetListenersArg {
                            loadbalancer_arn: args.arn,
                        }),
                    }),
                    args.output,
                ),
                GetKind::ElbRules(args) => (
                    AWSHandlerCommand::Elb(AWSElbCommand {
                        command: ElbSubCommand::GetRules(GetRulesArg {
                            listener_arn: args.arn,
                            num: args.limit,
                            tag: args.selector.map(|items| {
                                items
                                    .into_iter()
                                    .flat_map(|item| {
                                        let (key, value) = item
                                            .split_once('=')
                                            .expect("selector validated during parsing");
                                        [key.to_owned(), value.to_owned()]
                                    })
                                    .collect()
                            }),
                        }),
                    }),
                    args.output,
                ),
                GetKind::Asg(args) => (
                    AWSHandlerCommand::Asg(AWSASGCommand {
                        name: args.name,
                        command: ASGSubCommand::Get,
                    }),
                    args.output,
                ),
                GetKind::Route53(args) => (
                    AWSHandlerCommand::Route53(AWSRoute53Command {
                        command: Route53SubCommand::Get {
                            domain: args.domain,
                        },
                    }),
                    args.output,
                ),
            },
            ResourceVerb::Describe(args) => match args.resource {
                DescribeKind::Ec2(args) => (
                    AWSHandlerCommand::Ec2(AWSEC2Command {
                        command: EC2SubCommand::Describe(DescribeArg { id: args.id }),
                    }),
                    args.output,
                ),
            },
            ResourceVerb::Delete(args) => match args.resource {
                DeleteKind::Ec2(args) => (
                    AWSHandlerCommand::Ec2(AWSEC2Command {
                        command: EC2SubCommand::Terminate(TerminateArg {
                            instance_id: args.id,
                            yes: args.yes,
                        }),
                    }),
                    OutputFormat::Json,
                ),
            },
            ResourceVerb::Scale(args) => match args.resource {
                ScaleKind::Asg(args) => (
                    AWSHandlerCommand::Asg(AWSASGCommand {
                        name: args.name,
                        command: ASGSubCommand::Scale(ScaleArg {
                            min_size: args.min_size,
                            max_size: args.max_size,
                            desired_capacity: args.desired_capacity,
                            yes: args.yes,
                        }),
                    }),
                    OutputFormat::Json,
                ),
            },
            ResourceVerb::Attach(args) => match args.resource {
                AttachKind::Asg(args) => (
                    AWSHandlerCommand::Asg(AWSASGCommand {
                        name: args.name,
                        command: ASGSubCommand::AttachInstances(AttachInstancesArg {
                            ids: args.ids,
                        }),
                    }),
                    OutputFormat::Json,
                ),
            },
            ResourceVerb::Detach(args) => match args.resource {
                DetachKind::Asg(args) => (
                    AWSHandlerCommand::Asg(AWSASGCommand {
                        name: args.name,
                        command: ASGSubCommand::DetachInstances(DetachInstancesArg {
                            ids: args.ids,
                            replace: args.replace,
                            yes: args.yes,
                        }),
                    }),
                    OutputFormat::Json,
                ),
            },
        };
        Ok((command, format))
    }
}

impl AWSCommand {
    pub fn normalize_resource(&mut self) -> Result<(), String> {
        if matches!(
            self.command,
            AWSSubCommand::Get(_)
                | AWSSubCommand::Describe(_)
                | AWSSubCommand::Delete(_)
                | AWSSubCommand::Scale(_)
                | AWSSubCommand::Attach(_)
                | AWSSubCommand::Detach(_)
        ) {
            let command = match std::mem::replace(
                &mut self.command,
                AWSSubCommand::Login(crate::commands::aws::login::arg::AWSLoginCommand {
                    profile: None,
                }),
            ) {
                command @ (AWSSubCommand::Get(_)
                | AWSSubCommand::Describe(_)
                | AWSSubCommand::Delete(_)
                | AWSSubCommand::Scale(_)
                | AWSSubCommand::Attach(_)
                | AWSSubCommand::Detach(_)) => command,
                other => {
                    self.command = other;
                    return Ok(());
                }
            };
            let verb = match command {
                AWSSubCommand::Get(value) => ResourceVerb::Get(value),
                AWSSubCommand::Describe(value) => ResourceVerb::Describe(value),
                AWSSubCommand::Delete(value) => ResourceVerb::Delete(value),
                AWSSubCommand::Scale(value) => ResourceVerb::Scale(value),
                AWSSubCommand::Attach(value) => ResourceVerb::Attach(value),
                AWSSubCommand::Detach(value) => ResourceVerb::Detach(value),
                _ => unreachable!(),
            };
            let (command, format) = AWSResourceCommand::from_verb(verb).into_handler()?;
            self.handler_command = Some((command, format));
            self.output_format = format;
        }
        Ok(())
    }
}
