use crate::{args::KnifeArgs, commands::aws::arg::OutputFormat};
use clap::Parser;
use serde_json::json;

fn parse(args: &[&str]) -> KnifeArgs {
    KnifeArgs::try_parse_from(args).expect("arguments parse")
}

fn normalized(args: &[&str]) -> (String, OutputFormat) {
    let parsed = parse(args);
    let crate::args::Command::Aws(mut aws) = parsed.command else {
        panic!("expected AWS")
    };
    crate::commands::aws::resource::handler::validate_ec2_filters(&aws).unwrap();
    aws.normalize_resource().unwrap();
    let format = aws.output_format;
    let command = match aws.command {
        crate::commands::aws::arg::AWSSubCommand::LegacyASGCompat(legacy) => {
            // Exercise the adapter used by the real AWS handler dispatch.
            format!("{:?}", crate::commands::aws::legacy::normalize_asg(legacy))
        }
        command => format!("{command:?}"),
    };
    (command, format)
}

#[test]
fn canonical_resource_paths_parse_and_asg_normalizes_for_real_dispatch() {
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

    for (canonical, expected) in [
        (vec!["knife", "aws", "get", "asg", "g"], "Get"),
        (
            vec![
                "knife",
                "aws",
                "scale",
                "asg",
                "g",
                "--desired",
                "2",
                "--yes",
            ],
            "Scale(ScaleArg { min_size: None, max_size: None, desired_capacity: Some(2), yes: true })",
        ),
        (
            vec!["knife", "aws", "attach", "asg", "g", "i-1", "i-2"],
            "AttachInstances(AttachInstancesArg { ids: [\"i-1\", \"i-2\"] })",
        ),
        (
            vec![
                "knife",
                "aws",
                "detach",
                "asg",
                "g",
                "i-1",
                "--replace",
                "--yes",
            ],
            "DetachInstances(DetachInstancesArg { ids: [\"i-1\"], replace: true, yes: true })",
        ),
    ] {
        let (actual, _) = normalized(&canonical);
        assert!(
            actual.contains(expected),
            "actual={actual:?}, expected={expected:?}"
        );
    }
}

#[test]
fn legacy_resource_paths_parse_with_original_asg_name_placement() {
    for args in [
        vec!["knife", "aws", "ec2", "get", "--id", "i-12345678"],
        vec!["knife", "aws", "ec2", "get", "--name", "api"],
        vec!["knife", "aws", "ec2", "get", "--private-ip", "1.2.3.4"],
        vec!["knife", "aws", "ec2", "get", "--public-ip", "1.2.3.4"],
        vec!["knife", "aws", "ec2", "get", "--state", "running"],
        vec![
            "knife",
            "aws",
            "ec2",
            "terminate",
            "--id",
            "i-12345678",
            "--yes",
        ],
        vec![
            "knife", "aws", "elb", "get", "--name", "api", "--num", "3", "--fuzzy",
        ],
        vec!["knife", "aws", "elb", "get-listeners", "--arn", "arn:x"],
        vec![
            "knife",
            "aws",
            "elb",
            "get-rules",
            "--arn",
            "arn:x",
            "--tag",
            "env",
            "prod",
        ],
        vec!["knife", "aws", "asg", "--name", "group", "get"],
        vec![
            "knife",
            "aws",
            "asg",
            "--name",
            "group",
            "scale",
            "--desired",
            "2",
        ],
        vec![
            "knife",
            "aws",
            "asg",
            "--name",
            "group",
            "attach-instances",
            "--ids",
            "i-1",
            "i-2",
        ],
        vec![
            "knife",
            "aws",
            "asg",
            "--name",
            "group",
            "detach-instances",
            "--ids",
            "i-1",
            "--replace",
            "--yes",
        ],
        vec!["knife", "aws", "route53", "get", "example.com"],
        vec!["knife", "aws", "ssm", "start", "--id", "i-12345678"],
    ] {
        parse(&args);
    }
}

#[test]
fn old_and_new_commands_normalize_to_equivalent_handler_inputs() {
    for (old, new) in [
        (
            vec!["knife", "aws", "ec2", "get", "--id", "i-123"],
            vec!["knife", "aws", "get", "ec2", "--selector", "id=i-123"],
        ),
        (
            vec!["knife", "aws", "ec2", "get", "--name", "api"],
            vec!["knife", "aws", "get", "ec2", "--selector", "name=api"],
        ),
        (
            vec!["knife", "aws", "ec2", "get", "--name", "api"],
            vec!["knife", "aws", "get", "ec2", "--selector", "name=api"],
        ),
        (
            vec!["knife", "aws", "ec2", "get", "--name", "api"],
            vec!["knife", "aws", "get", "ec2", "api"],
        ),
        (
            vec!["knife", "aws", "ec2", "get", "--private-ip", "1.2.3.4"],
            vec!["knife", "aws", "get", "ec2", "--private-ip", "1.2.3.4"],
        ),
        (
            vec!["knife", "aws", "ec2", "get", "--public-ip", "1.2.3.4"],
            vec!["knife", "aws", "get", "ec2", "--public-ip", "1.2.3.4"],
        ),
        (
            vec!["knife", "aws", "ec2", "get", "--state", "running"],
            vec!["knife", "aws", "get", "ec2", "--state", "running"],
        ),
        (
            vec!["knife", "aws", "ec2", "terminate", "--id", "i-123", "--yes"],
            vec!["knife", "aws", "delete", "ec2", "i-123", "--yes"],
        ),
        (
            vec!["knife", "aws", "elb", "get", "--name", "api", "--num", "2"],
            vec!["knife", "aws", "get", "elb", "api", "--limit", "2"],
        ),
        (
            vec!["knife", "aws", "elb", "get", "--name", "api", "--fuzzy"],
            vec!["knife", "aws", "get", "elb", "api"],
        ),
        (
            vec!["knife", "aws", "elb", "get-listeners", "--arn", "arn:x"],
            vec!["knife", "aws", "get", "elb-listeners", "arn:x"],
        ),
        (
            vec![
                "knife",
                "aws",
                "elb",
                "get-rules",
                "--arn",
                "arn:x",
                "--tag",
                "k",
                "v",
            ],
            vec!["knife", "aws", "get", "elb-rules", "arn:x", "-l", "k=v"],
        ),
        (
            vec!["knife", "aws", "asg", "--name", "g", "get"],
            vec!["knife", "aws", "get", "asg", "g"],
        ),
        (
            vec![
                "knife",
                "aws",
                "asg",
                "--name",
                "g",
                "scale",
                "--desired",
                "2",
                "--yes",
            ],
            vec![
                "knife",
                "aws",
                "scale",
                "asg",
                "g",
                "--desired",
                "2",
                "--yes",
            ],
        ),
        (
            vec![
                "knife",
                "aws",
                "asg",
                "--name",
                "g",
                "attach-instances",
                "--ids",
                "i-1",
                "i-2",
            ],
            vec!["knife", "aws", "attach", "asg", "g", "i-1", "i-2"],
        ),
        (
            vec![
                "knife",
                "aws",
                "asg",
                "--name",
                "g",
                "detach-instances",
                "--ids",
                "i-1",
                "--replace",
                "--yes",
            ],
            vec![
                "knife",
                "aws",
                "detach",
                "asg",
                "g",
                "i-1",
                "--replace",
                "--yes",
            ],
        ),
        (
            vec!["knife", "aws", "route53", "get", "example.com"],
            vec!["knife", "aws", "get", "route53", "example.com"],
        ),
        (
            vec!["knife", "aws", "ssm", "start", "--id", "i-123"],
            vec!["knife", "aws", "ssm", "start", "i-123"],
        ),
    ] {
        let old_normalized = normalized(&old);
        let new_normalized = normalized(&new);
        if old.starts_with(&["knife", "aws", "ssm"]) {
            assert!(old_normalized.0.contains("Some(\"i-123\")"));
            assert!(new_normalized.0.contains("Some(\"i-123\")"));
            continue;
        }
        assert_eq!(old_normalized, new_normalized, "old={old:?}");
    }

    let root = normalized(&[
        "knife",
        "--profile",
        "dev",
        "--region",
        "us-west-2",
        "aws",
        "get",
        "ec2",
        "i-1",
    ]);
    let after_aws = normalized(&[
        "knife",
        "aws",
        "--profile",
        "dev",
        "--region",
        "us-west-2",
        "get",
        "ec2",
        "i-1",
    ]);
    assert_eq!(root, after_aws);
    assert_eq!(root.1, OutputFormat::Json);
    assert_eq!(
        normalized(&["knife", "aws", "get", "ec2", "i-1", "-o", "yaml"]).1,
        OutputFormat::Yaml
    );
    assert_eq!(
        normalized(&["knife", "aws", "get", "ec2", "i-1", "--output", "table"]).1,
        OutputFormat::Table
    );
    assert_eq!(
        normalized(&["knife", "aws", "get", "ec2", "i-1", "-o", "yaml"]).1,
        OutputFormat::Yaml
    );
    assert_eq!(
        normalized(&["knife", "aws", "get", "ec2", "i-1", "--output", "table"]).1,
        OutputFormat::Table
    );
}

#[test]
fn required_identifiers_and_filter_requirements_are_enforced() {
    assert!(KnifeArgs::try_parse_from(["knife", "aws", "get", "route53"]).is_err());
    assert!(KnifeArgs::try_parse_from(["knife", "aws", "delete", "ec2"]).is_err());
    assert!(KnifeArgs::try_parse_from(["knife", "aws", "get", "ec2"]).is_err());
    assert!(KnifeArgs::try_parse_from(["knife", "aws", "ssm", "start"]).is_err());
    assert!(KnifeArgs::try_parse_from(["knife", "aws", "asg", "get"]).is_err());

    let parsed = KnifeArgs::try_parse_from([
        "knife",
        "aws",
        "get",
        "ec2",
        "--state",
        "running",
        "--private-ip",
        "1.2.3.4",
    ])
    .unwrap();
    let crate::args::Command::Aws(aws) = parsed.command else {
        panic!("expected AWS")
    };
    let Err(error) = crate::commands::aws::resource::handler::validate_ec2_filters(&aws) else {
        panic!("expected incompatible-filter error")
    };
    assert!(error.contains("--state cannot be combined"));
    let unsupported =
        KnifeArgs::try_parse_from(["knife", "aws", "get", "ec2", "--selector", "tag=blue"])
            .unwrap();
    let crate::args::Command::Aws(aws) = unsupported.command else {
        panic!("expected AWS")
    };
    let Err(error) = crate::commands::aws::resource::handler::validate_ec2_filters(&aws) else {
        panic!("expected unsupported selector error")
    };
    assert!(error.contains("unsupported EC2 selector key"));
    let unsupported =
        KnifeArgs::try_parse_from(["knife", "aws", "get", "ec2", "--selector", "tag=blue"])
            .unwrap();
    let crate::args::Command::Aws(aws) = unsupported.command else {
        panic!("expected AWS")
    };
    assert!(crate::commands::aws::resource::handler::validate_ec2_filters(&aws).is_err());
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
