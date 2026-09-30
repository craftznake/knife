use super::{
    super::{
        arg::{AWSCommand, AWSHandlerCommand, AWSSubCommand, OutputFormat},
        asg::arg::{
            ASGSubCommand, AWSASGCommand, AttachInstancesArg, DetachInstancesArg, ScaleArg,
        },
        ec2::arg::{AWSEC2Command, DescribeArg, EC2SubCommand, SearchArg, TerminateArg},
        elb::arg::{AWSElbCommand, ElbSubCommand, GetArg, GetListenersArg, GetRulesArg},
        route53::arg::{AWSRoute53Command, Route53SubCommand},
    },
    arg::*,
};
impl AWSResourceCommand {
    pub fn into_handler(
        self,
        output_format: OutputFormat,
    ) -> Result<(AWSHandlerCommand, OutputFormat), String> {
        let (command, format) = match self.into_verb() {
            ResourceVerb::Get(get) => match get.resource {
                GetKind::Ec2(args) => {
                    if !args.state.is_empty()
                        && (args.private_ip.is_some() || args.public_ip.is_some())
                    {
                        return Err(
                            "--state cannot be combined with --private-ip or --public-ip".into(),
                        );
                    }
                    let selector = args.selector.unwrap_or_default();
                    let (name, instance_id) =
                        selector.iter().try_fold((None, None), |(name, id), pair| {
                            let (key, value) = pair
                                .split_once('=')
                                .ok_or_else(|| format!("selector must be KEY=VALUE: {pair}"))?;
                            if value.is_empty() {
                                return Err(format!("selector must be KEY=VALUE: {pair}"));
                            }
                            match key {
                                "name" => Ok((Some(value.to_owned()), id)),
                                "id" => Ok((name, Some(value.to_owned()))),
                                _ => Err(format!(
                                    "unsupported EC2 selector key: {key} (supported: name, id)"
                                )),
                            }
                        })?;
                    let identifier = args.identifier;
                    let name = name.or_else(|| {
                        identifier
                            .as_ref()
                            .filter(|s| !s.starts_with("i-"))
                            .cloned()
                    });
                    let instance_id =
                        instance_id.or_else(|| identifier.filter(|s| s.starts_with("i-")));
                    let search = SearchArg {
                        fuzzy: false,
                        private_ip: args.private_ip,
                        public_ip: args.public_ip,
                        instance_id,
                        name,
                        state: args.state,
                    };
                    (
                        AWSHandlerCommand::Ec2(AWSEC2Command {
                            command: EC2SubCommand::Get(search.normalize()?),
                        }),
                        output_format,
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
                    output_format,
                ),
                GetKind::ElbListener(args) => (
                    AWSHandlerCommand::Elb(AWSElbCommand {
                        command: ElbSubCommand::GetListeners(GetListenersArg {
                            loadbalancer_arn: args.arn,
                        }),
                    }),
                    output_format,
                ),
                GetKind::ElbRule(args) => (
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
                    output_format,
                ),
                GetKind::Asg(args) => (
                    AWSHandlerCommand::Asg(AWSASGCommand {
                        name: args.name,
                        command: ASGSubCommand::Get,
                    }),
                    output_format,
                ),
                GetKind::Route53(args) => (
                    AWSHandlerCommand::Route53(AWSRoute53Command {
                        command: Route53SubCommand::Get {
                            domain: args.domain,
                        },
                    }),
                    output_format,
                ),
            },
            ResourceVerb::Describe(args) => match args.resource {
                DescribeKind::Ec2(args) => (
                    AWSHandlerCommand::Ec2(AWSEC2Command {
                        command: EC2SubCommand::Describe(DescribeArg { id: args.id }),
                    }),
                    output_format,
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
                    output_format,
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
                    output_format,
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
                    output_format,
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
                    output_format,
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
            let (command, format) =
                AWSResourceCommand::from_verb(verb).into_handler(self.output_format)?;
            self.handler_command = Some((command, format));
            self.output_format = format;
        }
        Ok(())
    }
}
