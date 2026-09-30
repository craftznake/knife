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
        vec!["knife", "aws", "-o", "table", "get", "ec2", "i-1"],
    ] {
        parse(&argv);
    }
    assert_eq!(
        output_format(&["knife", "aws", "get", "ec2", "i-1", "-o", "yaml"]),
        OutputFormat::Yaml
    );
    assert_eq!(
        output_format(&["knife", "aws", "-o", "yaml", "get", "ec2", "i-1"]),
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
    let crate::args::Command::Aws(mut aws) = parsed.command else {
        panic!("expected AWS")
    };
    assert!(aws.normalize_resource().is_err());

    let crate::args::Command::Aws(mut aws) =
        parse(&["knife", "aws", "get", "ec2", "--selector", "tag=blue"]).command
    else {
        panic!("expected AWS")
    };
    assert!(aws.normalize_resource().is_err());

    let crate::args::Command::Aws(mut aws) =
        parse(&["knife", "aws", "get", "ec2", "--selector", "name=api"]).command
    else {
        panic!("expected AWS")
    };
    aws.normalize_resource().unwrap();
}

#[test]
fn output_renderers_cover_formats_and_nested_records() {
    use crate::commands::aws::resource::output::render;
    let value = json!({"nested": {"id": "i-1"}, "_url": "https://console", "Name": "api"});
    use crate::commands::aws::resource::output::ResourceKind;
    let rendered_json = render(&value, OutputFormat::Json, ResourceKind::Ec2).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&rendered_json).unwrap(),
        value
    );
    assert!(
        render(&value, OutputFormat::Yaml, ResourceKind::Ec2)
            .unwrap()
            .contains("nested:")
    );
    let table = render(&value, OutputFormat::Table, ResourceKind::Ec2).unwrap();
    assert!(table.contains("api") && !table.contains("_url"));
    let wide = render(&value, OutputFormat::Wide, ResourceKind::Ec2).unwrap();
    assert!(wide.contains("NAME") && wide.contains("https://console"));
}

#[test]
fn curated_tables_format_each_aws_resource_and_preserve_json_yaml() {
    use crate::commands::aws::resource::output::{ResourceKind, render};

    let asg = json!({
        "Name": "stg-api-gateway",
        "ARN": "arn:aws:autoscaling:us-east-1:123:autoScalingGroup:abc:autoScalingGroupName/stg-api-gateway",
        "Capacity": {"Current": 2, "Desired": 2, "MinSize": 2, "MaxSize": 15},
        "HealthCheckType": "EC2",
        "Instances": ["i-1", "i-2"],
        "LaunchTemplate": {"Name": "api-launch", "Version": "7"},
        "TargetGroups": ["arn:aws:elasticloadbalancing:us-east-1:123:targetgroup/api/abc"],
        "VPCZoneIdentifier": "subnet-a,subnet-b",
        "AvailabilityZones": ["us-east-1a", "us-east-1b"]
    });
    let table = render(&asg, OutputFormat::Table, ResourceKind::Asg).unwrap();
    assert_eq!(table.lines().count(), 3, "header, rule, and one record");
    assert!(table.contains("NAME") && table.contains("CAPACITY"));
    assert!(table.contains('│'));
    assert_eq!(table.matches("stg-api-gateway").count(), 1);
    assert!(table.contains("2/2 (2/15)"));
    assert!(table.contains("api-launch@7"));
    assert!(table.contains("EC2"));
    assert!(!table.contains("arn:aws:"));
    assert!(!table.contains('{'));
    let wide = render(&asg, OutputFormat::Wide, ResourceKind::Asg).unwrap();
    assert!(wide.contains("arn:aws:autoscaling"));
    assert!(wide.contains("subnet-a,subnet-b"));
    assert!(wide.contains("us-east-1a,us-east-1b"));

    let ec2 = json!({"InstanceId":"i-1","Name":"api","State":"running","InstanceType":"t3.small","PrivateIpAddress":"10.0.0.1","PublicIpAddress":"203.0.113.1","AvailabilityZone":"us-east-1a","Tags":{"Name":"api"}});
    let table = render(&ec2, OutputFormat::Table, ResourceKind::Ec2).unwrap();
    assert!(table.lines().next().unwrap().contains("ID"));
    assert!(table.contains("i-1") && table.contains("api") && table.contains("running"));
    let wide = render(&ec2, OutputFormat::Wide, ResourceKind::Ec2).unwrap();
    assert!(
        wide.contains("203.0.113.1") && wide.contains("us-east-1a") && wide.contains("Name: api")
    );

    let elb = json!({"Name":"api","Arn":"arn:aws:elasticloadbalancing:us-east-1:123:loadbalancer/app/api/abc","Scheme":"internet-facing","DNSName":"api.example.com","Listeners":["https","http"]});
    let table = render(&elb, OutputFormat::Table, ResourceKind::Elb).unwrap();
    assert!(table.lines().next().unwrap().contains("SCHEME"));
    assert!(table.contains("internet-facing") && table.contains("api.example.com"));
    assert!(
        render(&elb, OutputFormat::Wide, ResourceKind::Elb)
            .unwrap()
            .contains("arn:aws:")
    );

    let listener = json!({"Protocol":"HTTPS","Port":443,"TargetGroupArn":"arn:aws:elasticloadbalancing:us-east-1:123:targetgroup/api/abc","Arn":"arn:aws:elasticloadbalancing:us-east-1:123:listener/app/api/abc/xyz"});
    let table = render(&listener, OutputFormat::Table, ResourceKind::ElbListeners).unwrap();
    assert!(table.lines().next().unwrap().contains("PROTOCOL"));
    assert!(table.contains("HTTPS") && table.contains("443") && table.contains("abc"));
    assert!(
        render(&listener, OutputFormat::Wide, ResourceKind::ElbListeners)
            .unwrap()
            .contains("arn:aws:")
    );

    let rule = json!({"Priority":"10","Actions":[{"Type":"forward","Config":{"arn:aws:elasticloadbalancing:us-east-1:123:targetgroup/api/abc":"1"}}],"Conditions":{"host-header":"api.example.com"},"Arn":"arn:aws:elasticloadbalancing:us-east-1:123:listener-rule/app/api/abc/xyz"});
    let table = render(&rule, OutputFormat::Table, ResourceKind::ElbRules).unwrap();
    assert!(table.lines().next().unwrap().contains("PRIORITY"));
    assert!(table.contains("forward -> abc"));
    assert!(!table.contains("arn:aws:"));

    let route53 = json!({"hosted_zone":{},"records":[{"Name":"api.example.com.","Type":"A","TTL":60,"Values":["192.0.2.1","192.0.2.2"]}]});
    let table = render(&route53, OutputFormat::Table, ResourceKind::Route53).unwrap();
    assert!(table.lines().next().unwrap().contains("VALUE"));
    assert!(table.contains("api.example.com."));
    assert!(table.contains("192.0.2.1") && table.contains("192.0.2.2"));
    assert!(!table.contains("…(+"));

    let route53_live_shape = json!({"records":[{"Name":"api.grab.com.","Type":"A","TTL":0,"Values":["prd-api-gateway-public-alb-1780608871111.us-east-1.elb.amazonaws.com","additional-record-value"]}]});
    let live_table = render(
        &route53_live_shape,
        OutputFormat::Table,
        ResourceKind::Route53,
    )
    .unwrap();
    assert!(live_table.contains("api.grab.com."));
    assert!(
        live_table.contains("prd-api-gateway-public-alb-1780608871111.us-east-1.elb.amazonaws.com")
    );
    assert!(live_table.contains("additional-record-value"));
    assert!(!live_table.contains("…(+"));

    for format in [OutputFormat::Json, OutputFormat::Yaml] {
        let rendered = render(&asg, format, ResourceKind::Asg).unwrap();
        let decoded = if format == OutputFormat::Json {
            serde_json::from_str::<serde_json::Value>(&rendered).unwrap()
        } else {
            serde_yaml::from_str::<serde_json::Value>(&rendered).unwrap()
        };
        assert_eq!(decoded, asg);
    }
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
