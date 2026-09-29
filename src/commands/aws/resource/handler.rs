use super::{
    super::{
        arg::{AWSCommand, AWSSubCommand, OutputFormat},
        ec2::arg::{AWSEC2Command, EC2SubCommand, SearchArg, TerminateArg},
        elb::arg::{AWSElbCommand, ElbSubCommand, GetArg, GetListenersArg, GetRulesArg},
        route53::arg::{AWSRoute53Command, Route53SubCommand},
    },
    arg::*,
};
pub fn validate_ec2_filters(aws: &AWSCommand) -> Result<(), String> {
    let AWSSubCommand::Get(GetResource {
        resource: GetKind::Ec2(args),
    }) = &aws.command
    else {
        return Ok(());
    };

    if !args.state.is_empty() && (args.private_ip.is_some() || args.public_ip.is_some()) {
        return Err("--state cannot be combined with --private-ip or --public-ip".into());
    }
    if let Some(selector) = &args.selector {
        for entry in selector {
            let Some((key, value)) = entry.split_once('=') else {
                return Err(format!("selector must be KEY=VALUE: {entry}"));
            };
            if value.is_empty() {
                return Err(format!("selector must be KEY=VALUE: {entry}"));
            }
            if !matches!(key, "name" | "id") {
                return Err(format!(
                    "unsupported EC2 selector key: {key} (supported: name, id)"
                ));
            }
        }
    }
    Ok(())
}

impl AWSResourceCommand {
    pub fn normalize(self) -> Result<(AWSSubCommand, OutputFormat), String> {
        let (command, format) = match self.into_verb() {
            ResourceVerb::Get(get) => match get.resource {
                GetKind::Ec2(args) => {
                    let selector = args.selector.unwrap_or_default();
                    let (name, instance_id) =
                        selector.iter().fold((None, None), |(name, id), pair| {
                            let (key, value) = pair
                                .split_once('=')
                                .expect("selector validated before normalization");
                            match key {
                                "name" => (Some(value.to_owned()), id),
                                "id" => (name, Some(value.to_owned())),
                                _ => unreachable!("unsupported EC2 selectors are rejected"),
                            }
                        });
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
                        AWSSubCommand::EC2Compat(AWSEC2Command {
                            command: EC2SubCommand::Get(search.normalize()?),
                        }),
                        args.output,
                    )
                }
                GetKind::Elb(args) => (
                    AWSSubCommand::LegacyElb(AWSElbCommand {
                        command: ElbSubCommand::Get(GetArg {
                            name: args.name.unwrap_or_default(),
                            num: args.limit,
                            fuzzy: !args.exact,
                        }),
                    }),
                    args.output,
                ),
                GetKind::ElbListeners(args) => (
                    AWSSubCommand::LegacyElb(AWSElbCommand {
                        command: ElbSubCommand::GetListeners(GetListenersArg {
                            loadbalancer_arn: args.arn,
                        }),
                    }),
                    args.output,
                ),
                GetKind::ElbRules(args) => (
                    AWSSubCommand::LegacyElb(AWSElbCommand {
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
                    AWSSubCommand::LegacyASGCompat(crate::commands::aws::legacy::LegacyASG {
                        name: args.name,
                        command: crate::commands::aws::legacy::LegacyASGCommand::Get,
                    }),
                    args.output,
                ),
                GetKind::Route53(args) => (
                    AWSSubCommand::LegacyRoute53(AWSRoute53Command {
                        command: Route53SubCommand::Get {
                            domain: args.domain,
                        },
                    }),
                    args.output,
                ),
            },
            ResourceVerb::Describe(args) => match args.resource {
                DescribeKind::Ec2(args) => (
                    AWSSubCommand::EC2Compat(AWSEC2Command {
                        command: EC2SubCommand::Describe(
                            crate::commands::aws::ec2::arg::DescribeArg { id: args.id },
                        ),
                    }),
                    args.output,
                ),
            },
            ResourceVerb::Delete(args) => match args.resource {
                DeleteKind::Ec2(args) => (
                    AWSSubCommand::EC2Compat(AWSEC2Command {
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
                    AWSSubCommand::LegacyASGCompat(crate::commands::aws::legacy::LegacyASG {
                        name: args.name,
                        command: crate::commands::aws::legacy::LegacyASGCommand::Scale {
                            min: args.min_size,
                            max: args.max_size,
                            desired: args.desired_capacity,
                            yes: args.yes,
                        },
                    }),
                    OutputFormat::Json,
                ),
            },
            ResourceVerb::Attach(args) => match args.resource {
                AttachKind::Asg(args) => (
                    AWSSubCommand::LegacyASGCompat(crate::commands::aws::legacy::LegacyASG {
                        name: args.name,
                        command: crate::commands::aws::legacy::LegacyASGCommand::AttachInstances {
                            ids: args.ids,
                        },
                    }),
                    OutputFormat::Json,
                ),
            },
            ResourceVerb::Detach(args) => match args.resource {
                DetachKind::Asg(args) => (
                    AWSSubCommand::LegacyASGCompat(crate::commands::aws::legacy::LegacyASG {
                        name: args.name,
                        command: crate::commands::aws::legacy::LegacyASGCommand::DetachInstances {
                            ids: args.ids,
                            replace: args.replace,
                            yes: args.yes,
                        },
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
            let (command, format) = AWSResourceCommand::from_verb(verb).normalize()?;
            self.command = command;
            self.output_format = format;
        }
        Ok(())
    }
}
