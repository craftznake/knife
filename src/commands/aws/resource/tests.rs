use crate::{args::KnifeArgs, commands::aws::arg::OutputFormat};
use clap::Parser;
use serde_json::json;

fn parse(args: &[&str]) -> KnifeArgs {
    KnifeArgs::try_parse_from(args).expect("arguments parse")
}

fn output_format(args: &[&str]) -> OutputFormat {
    let parsed = parse(args);
    let crate::args::Command::Aws(mut aws) = parsed.command else {
        panic!("expected AWS")
    };
    aws.normalize_resource().unwrap();
    aws.output_format
}

#[test]
fn canonical_resource_paths_parse() {
    for args in [
        vec!["knife", "aws", "get", "ec2", "i-12345678", "-o", "json"],
        vec![
            "knife", "aws", "get", "elb", "api", "--limit", "8", "--exact", "--output", "yaml",
        ],
        vec![
            "knife",
            "aws",
            "get",
            "elb-listeners",
            "arn:x",
            "-o",
            "table",
        ],
        vec![
            "knife",
            "aws",
            "get",
            "elb-rules",
            "arn:x",
            "-l",
            "env=prod",
            "--limit",
            "4",
            "-o",
            "wide",
        ],
        vec!["knife", "aws", "get", "asg", "group", "-o", "json"],
        vec![
            "knife",
            "aws",
            "get",
            "route53",
            "example.com",
            "-o",
            "yaml",
        ],
        vec!["knife", "aws", "describe", "ec2", "i-12345678"],
        vec!["knife", "aws", "delete", "ec2", "i-12345678", "--yes"],
        vec!["knife", "aws", "scale", "asg", "group", "--desired", "2"],
        vec!["knife", "aws", "attach", "asg", "group", "i-1", "i-2"],
        vec![
            "knife",
            "aws",
            "detach",
            "asg",
            "group",
            "i-1",
            "--replace",
            "--yes",
        ],
        vec!["knife", "aws", "ssm", "start", "i-12345678"],
    ] {
        parse(&args);
    }
}

#[test]
fn legacy_resource_paths_are_rejected() {
    for args in [
        vec!["knife", "aws", "ec2", "get", "--id", "x"],
        vec!["knife", "aws", "elb", "get", "--name", "n"],
        vec!["knife", "aws", "asg", "--name", "g", "get"],
        vec!["knife", "aws", "route53", "get", "d"],
        vec!["knife", "aws", "ssm", "start", "--id", "x"],
    ] {
        assert!(
            KnifeArgs::try_parse_from(&args).is_err(),
            "accepted {args:?}"
        );
    }
}

#[test]
fn context_flags_and_output_formats_parse() {
    for argv in [
        vec![
            "knife",
            "--profile",
            "dev",
            "--region",
            "us-west-2",
            "aws",
            "get",
            "ec2",
            "i-1",
        ],
        vec![
            "knife",
            "aws",
            "--profile",
            "dev",
            "--region",
            "us-west-2",
            "get",
            "ec2",
            "i-1",
        ],
        vec!["knife", "aws", "get", "ec2", "i-1", "-o", "yaml"],
        vec!["knife", "aws", "get", "ec2", "i-1", "--output", "table"],
    ] {
        parse(&argv);
    }
    assert_eq!(
        output_format(&["knife", "aws", "get", "ec2", "i-1", "-o", "yaml"]),
        OutputFormat::Yaml
    );
}

#[test]
fn required_identifiers_and_filter_requirements_are_enforced() {
    for argv in [
        vec!["knife", "aws", "get", "route53"],
        vec!["knife", "aws", "delete", "ec2"],
        vec!["knife", "aws", "get", "ec2"],
        vec!["knife", "aws", "ssm", "start"],
        vec!["knife", "aws", "get", "asg"],
    ] {
        assert!(
            KnifeArgs::try_parse_from(&argv).is_err(),
            "accepted {argv:?}"
        );
    }
    let parsed = parse(&[
        "knife",
        "aws",
        "get",
        "ec2",
        "--state",
        "running",
        "--private-ip",
        "1.2.3.4",
    ]);
    let crate::args::Command::Aws(aws) = parsed.command else {
        panic!("expected AWS")
    };
    let crate::commands::aws::resource::arg::GetKind::Ec2(args) = (match &aws.command {
        crate::commands::aws::arg::AWSSubCommand::Get(get) => &get.resource,
        _ => panic!("expected EC2 get"),
    }) else {
        panic!("expected EC2 get")
    };
    let Err(error) = crate::commands::aws::ec2::handler::validate_filters(args) else {
        panic!("expected incompatible-filter error")
    };
    assert!(error.contains("--state cannot be combined"));
    let parsed = parse(&["knife", "aws", "get", "ec2", "--selector", "tag=blue"]);
    let crate::args::Command::Aws(aws) = parsed.command else {
        panic!("expected AWS")
    };
    let crate::commands::aws::resource::arg::GetKind::Ec2(args) = (match &aws.command {
        crate::commands::aws::arg::AWSSubCommand::Get(get) => &get.resource,
        _ => panic!("expected EC2 get"),
    }) else {
        panic!("expected EC2 get")
    };
    let Err(error) = crate::commands::aws::ec2::handler::validate_filters(args) else {
        panic!("expected unsupported selector error")
    };
    assert!(error.contains("unsupported EC2 selector key"));
}

#[test]
fn output_renderers_cover_formats_and_nested_records() {
    use crate::commands::aws::resource::output::render;
    let value = json!({"nested": {"id": "i-1"}, "_url": "https://console", "name": "api"});
    let rendered_json = render(&value, OutputFormat::Json).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&rendered_json).unwrap(),
        value
    );
    assert!(
        render(&value, OutputFormat::Yaml)
            .unwrap()
            .contains("nested:")
    );
    let table = render(&value, OutputFormat::Table).unwrap();
    assert!(table.contains("name") && !table.contains("_url"));
    let wide = render(&value, OutputFormat::Wide).unwrap();
    assert!(wide.contains("_url") && wide.contains("name"));
}

#[test]
fn jwt_destination_collision_is_scoped() {
    assert!(
        KnifeArgs::try_parse_from([
            "knife", "jwt", "encode", "{}", "--secret", "s", "-o", "file"
        ])
        .is_err()
    );
    for flag in ["--out-file", "--out"] {
        assert!(
            KnifeArgs::try_parse_from([
                "knife", "jwt", "encode", "{}", "--secret", "s", flag, "file"
            ])
            .is_ok()
        );
    }
    let parsed = parse(&[
        "knife",
        "aws",
        "get",
        "ec2",
        "--private-ip",
        "1.2.3.4",
        "-o",
        "yaml",
    ]);
    let crate::args::Command::Aws(mut aws) = parsed.command else {
        panic!("expected AWS")
    };
    aws.normalize_resource().unwrap();
    assert_eq!(aws.output_format, OutputFormat::Yaml);
}
