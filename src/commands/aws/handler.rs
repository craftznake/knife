use aws_config::{BehaviorVersion, Region, SdkConfig};

use crate::commands::{
    AWSCommand, CommandHandler, Output,
    aws::{
        arg::{AWSHandlerCommand, GlobalOptions},
        login::handler::{load_last_profile, load_last_region},
    },
};

pub struct AWSHandler {
    cmd: AWSCommand,
}

fn recent_target(
    command: &AWSCommand,
) -> (Option<String>, Option<&'static str>, Option<String>, bool) {
    use crate::commands::aws::{arg::AWSSubCommand, resource::arg::GetKind};
    let profile = command.profile.clone();
    let target = match &command.command {
        AWSSubCommand::Get(get) => match &get.resource {
            GetKind::Ec2(args) => args.identifier.as_deref().map(|id| ("ec2", id)),
            GetKind::Elb(args) => args.name.as_deref().map(|name| ("elb", name)),
            GetKind::ElbListeners(args) => Some(("elb-listeners", args.arn.as_str())),
            GetKind::ElbRules(args) => Some(("elb-rules", args.arn.as_str())),
            GetKind::Asg(args) => Some(("asg", args.name.as_str())),
            GetKind::Route53(args) => Some(("route53", args.domain.as_str())),
        },
        AWSSubCommand::Scale(args) => match &args.resource {
            crate::commands::aws::resource::arg::ScaleKind::Asg(args) => {
                Some(("asg", args.name.as_str()))
            }
        },
        AWSSubCommand::Attach(args) => match &args.resource {
            crate::commands::aws::resource::arg::AttachKind::Asg(args) => {
                Some(("asg", args.name.as_str()))
            }
        },
        AWSSubCommand::Detach(args) => match &args.resource {
            crate::commands::aws::resource::arg::DetachKind::Asg(args) => {
                Some(("asg", args.name.as_str()))
            }
        },
        _ => None,
    };
    let is_read = matches!(
        &command.command,
        AWSSubCommand::Get(_)
            | AWSSubCommand::Scale(_)
            | AWSSubCommand::Attach(_)
            | AWSSubCommand::Detach(_)
    );
    match target {
        Some((resource, id)) => (profile, Some(resource), Some(id.to_string()), is_read),
        None => (profile, None, None, false),
    }
}

async fn account_id(config: &SdkConfig) -> Option<String> {
    aws_sdk_sts::Client::new(config)
        .get_caller_identity()
        .send()
        .await
        .ok()
        .and_then(|identity| identity.account().map(str::to_owned))
}

impl CommandHandler for AWSHandler {
    async fn execute(self) -> Result<Output, Box<dyn std::error::Error>> {
        let mut command_args = self.cmd;
        let (_requested_profile, resource, identifier, is_read) = recent_target(&command_args);
        crate::commands::aws::resource::handler::validate_ec2_filters(&command_args)?;
        command_args.normalize_resource()?;
        let handler_command = command_args.handler_command;
        let AWSCommand {
            command,
            region,
            profile,
            verbose,
            debug,
            output_format,
            ..
        } = command_args;

        // Some commands don't need to load AWS config, just execute them directly
        let output_format = handler_command
            .as_ref()
            .map_or(output_format, |(_, format)| *format);

        let command = handler_command
            .map(|(command, _)| command)
            .unwrap_or_else(|| command.into());

        if let AWSHandlerCommand::Login(sso_cmd) = command {
            // SSO login doesn't require credentials
            let _ = sso_cmd.execute(verbose).await;
            let output = Output::new(verbose);
            output.stderr("SSO login completed");
            return Ok(output);
        }

        let selected_profile = profile.clone();
        let sdk_config = Self::load_sdk_config(region, selected_profile.clone(), verbose).await?;
        let opts = GlobalOptions {
            verbose: verbose || debug,
            sdk_config: sdk_config.clone(),
            output_format,
        };

        // Note: SSO command is handled above before credential loading
        let output = match command {
            AWSHandlerCommand::Elb(elb_cmd) => elb_cmd.execute(opts).await?,
            AWSHandlerCommand::Whoami(whoami_cmd) => whoami_cmd.execute(opts).await?,
            AWSHandlerCommand::Route53(route53_cmd) => route53_cmd.execute(opts).await?,
            AWSHandlerCommand::Ec2(ec2_cmd) => ec2_cmd.execute(opts).await?,
            AWSHandlerCommand::Asg(asg_cmd) => asg_cmd.execute(opts).await?,
            AWSHandlerCommand::SSM(ssm_cmd) => ssm_cmd.execute(opts).await?,
            AWSHandlerCommand::Console(console_cmd) => console_cmd.execute(opts).await?,
            AWSHandlerCommand::Logout(logout_cmd) => logout_cmd.execute(opts).await?,
            AWSHandlerCommand::Login(_) => {
                unreachable!("SSO command should have been handled earlier")
            }
        };
        if let (Some(resource), Some(identifier)) = (resource, identifier) {
            let active_profile = _requested_profile
                .or_else(|| profile.clone())
                .or_else(|| std::env::var("AWS_PROFILE").ok())
                .or_else(load_last_profile)
                .unwrap_or_else(|| "default".to_string());
            if is_read && let Some(account_id) = account_id(&sdk_config).await {
                crate::recents::record(&active_profile, &account_id, resource, &identifier);
            }
        }
        Ok(output)
    }
}

impl AWSHandler {
    pub fn new(cmd: AWSCommand) -> Self {
        AWSHandler { cmd }
    }

    async fn load_sdk_config(
        region: Option<String>,
        profile: Option<String>,
        verbose: bool,
    ) -> Result<SdkConfig, Box<dyn std::error::Error>> {
        // Start with AWS defaults (respects AWS_PROFILE, AWS_REGION, etc.)
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest());
        // Profile selection priority:
        // 1. Explicit --profile flag
        // 2. AWS_PROFILE environment variable
        // 3. Last used profile (from knife config)
        // 4. Default profile
        let selected_profile = if let Some(profile_name) = &profile {
            if verbose {
                eprintln!("Using profile from --profile flag: {}", profile_name);
            }
            Some(profile_name.clone())
        } else if let Ok(env_profile) = std::env::var("AWS_PROFILE") {
            if verbose {
                eprintln!("Using profile from AWS_PROFILE env: {}", env_profile);
            }
            Some(env_profile)
        } else if let Some(last_profile) = load_last_profile() {
            if verbose {
                eprintln!(
                    "Using last used profile from knife config: {}",
                    last_profile
                );
            }
            Some(last_profile)
        } else {
            if verbose {
                eprintln!("No profile specified, using default AWS profile");
            }
            None
        };

        if let Some(profile_name) = selected_profile {
            config_loader = config_loader.profile_name(profile_name);
        }

        // Region selection priority:
        // 1. Explicit --region flag
        // 2. AWS_REGION environment variable
        // 3. Last used region (from knife config)
        // 4. Profile's configured region (AWS SDK handles this)
        let selected_region = if let Some(region_str) = region {
            if verbose {
                eprintln!("Using region from --region flag: {}", region_str);
            }
            Some(region_str)
        } else if let Ok(env_region) = std::env::var("AWS_REGION") {
            if verbose {
                eprintln!("Using region from AWS_REGION env: {}", env_region);
            }
            Some(env_region)
        } else if let Some(last_region) = load_last_region() {
            if verbose {
                eprintln!("Using last used region from knife config: {}", last_region);
            }
            Some(last_region)
        } else {
            if verbose {
                eprintln!("Using region from profile config");
            }
            None
        };

        if let Some(region_str) = selected_region {
            config_loader = config_loader.region(Region::new(region_str));
        }

        // Try to load the config - AWS SDK will validate profile exists
        let sdk_config = config_loader.load().await;

        // Verify credentials are available
        if sdk_config.credentials_provider().is_none() {
            return Err("No AWS credentials found\n\nPlease ensure:\n  1. Profile exists\n  2. AWS_PROFILE environment variable is set, or\n  3. Use --profile flag to specify a profile".into());
        }

        // Verify region is set
        if sdk_config.region().is_none() {
            return Err("Region not found\n\nPlease ensure:\n  1. Region is set in current profile\n  2. AWS_REGION environment variable is set, or\n  3. Use --region flag to specify a region".into());
        }
        Ok(sdk_config)
    }
}
