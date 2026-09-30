use super::super::arg::OutputFormat;
use comfy_table::{Cell, ContentArrangement, ContentLineStyle, LineStyle, Table, TableStyle};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceKind {
    Asg,
    Ec2,
    Elb,
    ElbListeners,
    ElbRules,
    Route53,
}

pub fn render(
    value: &Value,
    format: OutputFormat,
    kind: ResourceKind,
) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(value)?),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(value)?),
        OutputFormat::Table | OutputFormat::Wide => Ok(render_table(value, format, kind)),
    }
}

fn render_table(value: &Value, format: OutputFormat, kind: ResourceKind) -> String {
    let wrapped_route53 =
        kind == ResourceKind::Route53 && value.get("records").is_some_and(Value::is_array);
    let value_rows = if wrapped_route53 {
        &value["records"]
    } else {
        value
    };
    let rows: Vec<&Value> = match value_rows {
        Value::Array(rows) => rows.iter().collect(),
        Value::Object(_) => vec![value_rows],
        _ => return value_rows.to_string(),
    };
    if rows.is_empty() {
        return String::new();
    }

    let wide = format == OutputFormat::Wide;
    let columns: &[&str] = match kind {
        #[rustfmt::skip]
        ResourceKind::Asg if wide => &[ "NAME", "DESIRED", "MIN", "MAX", "HEALTH", "INSTANCES", "LAUNCH-TEMPLATE", "TARGET-GROUPS", "ARN", "SUBNETS", "AZS" ],
        #[rustfmt::skip]
        ResourceKind::Asg => &[ "NAME", "CAPACITY", "HEALTH", "INSTANCES", "LAUNCH-TEMPLATE", "TARGET-GROUPS" ],
        #[rustfmt::skip]
        ResourceKind::Ec2 if wide => &[ "ID", "NAME", "STATE", "TYPE", "PRIVATE-IP", "PUBLIC-IP", "AZ", "TAGS", "_URL" ],
        ResourceKind::Ec2 => &["ID", "NAME", "STATE", "TYPE", "PRIVATE-IP"],
        ResourceKind::Elb if wide => &["NAME", "SCHEME", "DNS", "LISTENERS", "ARN"],
        ResourceKind::Elb => &["NAME", "SCHEME", "DNS", "LISTENERS"],
        ResourceKind::ElbListeners if wide => &["PROTOCOL", "PORT", "TARGET-GROUP", "ARN"],
        ResourceKind::ElbListeners => &["PROTOCOL", "PORT", "TARGET-GROUP"],
        ResourceKind::ElbRules if wide => &["PRIORITY", "ACTION", "CONDITIONS", "ARN"],
        ResourceKind::ElbRules => &["PRIORITY", "ACTION", "CONDITIONS"],
        ResourceKind::Route53 => &["NAME", "TYPE", "TTL", "WEIGHT", "VALUE"],
    };
    let mut table = Table::new();
    table.load_style(
        TableStyle::new()
            .header_separator(LineStyle::new('├', '─', '┼', '┤'))
            .header_lines(ContentLineStyle::new('│', '│', '│'))
            .content_lines(ContentLineStyle::new('│', '│', '│'))
            .top_border(LineStyle::new('┌', '─', '┬', '┐'))
            .bottom_border(LineStyle::new('└', '─', '┴', '┘')),
    );
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(columns.iter().map(Cell::new));
    for row in rows {
        table.add_row(
            columns
                .iter()
                .map(|column| Cell::new(cell(row, kind, column, wide))),
        );
    }
    table.to_string()
}

fn cell(row: &Value, kind: ResourceKind, column: &str, wide: bool) -> String {
    let value = match (kind, column) {
        (ResourceKind::Asg, "NAME") => row.get("Name"),
        (ResourceKind::Asg, "CAPACITY") => {
            return format!(
                "{}/{} ({}/{})",
                nested(row, "Capacity", "Current").unwrap_or("-".into()),
                nested(row, "Capacity", "Desired").unwrap_or("-".into()),
                nested(row, "Capacity", "MinSize").unwrap_or("-".into()),
                nested(row, "Capacity", "MaxSize").unwrap_or("-".into()),
            );
        }
        (ResourceKind::Asg, "DESIRED") => {
            return nested(row, "Capacity", "Desired").unwrap_or_else(|| "-".into());
        }
        (ResourceKind::Asg, "MIN") => {
            return nested(row, "Capacity", "MinSize").unwrap_or_else(|| "-".into());
        }
        (ResourceKind::Asg, "MAX") => {
            return nested(row, "Capacity", "MaxSize").unwrap_or_else(|| "-".into());
        }
        (ResourceKind::Asg, "HEALTH") => row.get("HealthCheckType"),
        (ResourceKind::Asg, "INSTANCES") => return count_or_names(row.get("Instances"), wide),
        (ResourceKind::Asg, "LAUNCH-TEMPLATE") => return launch_template(row),
        (ResourceKind::Asg, "TARGET-GROUPS") => {
            return arn_list(row.get("TargetGroups"), wide, true);
        }
        (ResourceKind::Asg, "ARN") => row.get("ARN"),
        (ResourceKind::Asg, "SUBNETS") => return expanded_list(row.get("VPCZoneIdentifier")),
        (ResourceKind::Asg, "AZS") => return count_or_names(row.get("AvailabilityZones"), wide),

        (ResourceKind::Ec2, "ID") => row.get("InstanceId"),
        (ResourceKind::Ec2, "NAME") => row.get("Name").or_else(|| row.get("name")),

        (ResourceKind::Ec2, "STATE") => row.get("State"),
        (ResourceKind::Ec2, "TYPE") => row.get("InstanceType"),
        (ResourceKind::Ec2, "PRIVATE-IP") => row.get("PrivateIpAddress"),
        (ResourceKind::Ec2, "PUBLIC-IP") => row.get("PublicIpAddress"),
        (ResourceKind::Ec2, "AZ") => row.get("AvailabilityZone"),
        (ResourceKind::Ec2, "TAGS") => row.get("Tags"),
        (ResourceKind::Ec2, "_URL") => row.get("_url"),

        (ResourceKind::Elb, "NAME") => row.get("Name"),
        (ResourceKind::Elb, "SCHEME") => row.get("Scheme"),
        (ResourceKind::Elb, "DNS") => row.get("DNSName"),
        (ResourceKind::Elb, "LISTENERS") => return count_or_names(row.get("Listeners"), wide),
        (ResourceKind::Elb, "ARN") => row.get("Arn"),

        (ResourceKind::ElbListeners, "PROTOCOL") => row.get("Protocol"),
        (ResourceKind::ElbListeners, "PORT") => row.get("Port"),
        (ResourceKind::ElbListeners, "TARGET-GROUP") => {
            let direct_target = row.get("TargetGroupArn");
            let target = direct_target
                .or_else(|| row.get("DefaultActions"))
                .or_else(|| row.get("TargetGroups"));
            if let Some(arn) = direct_target.and_then(Value::as_str) {
                return if wide {
                    arn.to_string()
                } else {
                    arn_tail(arn).to_string()
                };
            }
            return if let Some(target) = target.and_then(Value::as_str) {
                let target = if wide { target } else { arn_tail(target) };
                target.to_string()
            } else if let Some(actions) = target.and_then(Value::as_array) {
                let arns = actions
                    .iter()
                    .filter_map(|action| action.get("TargetGroupArn").and_then(Value::as_str))
                    .collect::<Vec<_>>();
                if arns.is_empty() {
                    "-".into()
                } else {
                    let names = arns
                        .iter()
                        .map(|arn| if wide { *arn } else { arn_tail(arn) })
                        .collect::<Vec<_>>();
                    names.join(",")
                }
            } else {
                arn_list(target, wide, false)
            };
        }
        (ResourceKind::ElbListeners, "ARN") => row.get("Arn"),

        (ResourceKind::ElbRules, "PRIORITY") => row.get("Priority"),
        (ResourceKind::ElbRules, "ACTION") => return actions(row.get("Actions")),
        (ResourceKind::ElbRules, "CONDITIONS") => return compact(row.get("Conditions")),
        (ResourceKind::ElbRules, "ARN") => row.get("Arn"),

        (ResourceKind::Route53, "NAME") => row.get("Name"),
        (ResourceKind::Route53, "TYPE") => row.get("Type"),
        (ResourceKind::Route53, "TTL") => row.get("TTL"),
        (ResourceKind::Route53, "WEIGHT") => row.get("Weight"),
        (ResourceKind::Route53, "VALUE") => return route53_value(row),
        _ => None,
    };
    value
        .map(|v| display(v, wide))
        .unwrap_or_else(|| "-".into())
}

fn nested(row: &Value, object: &str, key: &str) -> Option<String> {
    row.get(object)?.get(key).map(scalar)
}

fn scalar(value: &Value) -> String {
    match value {
        Value::Null => "-".into(),
        Value::String(text) if text.is_empty() || text == "N/A" => "-".into(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn display(value: &Value, wide: bool) -> String {
    match value {
        Value::Array(values) => format!("{}", values.len()),
        Value::Object(object) => object
            .iter()
            .map(|(key, value)| format!("{key}: {}", scalar(value)))
            .collect::<Vec<_>>()
            .join(", "),
        Value::String(text) if text.starts_with("arn:") && !wide => arn_tail(text).to_string(),
        other => scalar(other),
    }
}

fn arn_tail(arn: &str) -> &str {
    arn.rsplit('/').next().unwrap_or(arn)
}

fn count_or_names(value: Option<&Value>, wide: bool) -> String {
    let Some(value) = value else {
        return "-".into();
    };
    match value {
        Value::Array(_items) if wide => expanded_list(Some(value)),
        Value::Array(items) => items.len().to_string(),
        Value::Null => "-".into(),
        Value::String(text) if text.is_empty() || text == "N/A" => "-".into(),
        Value::String(text) => {
            let items = text
                .split(',')
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>();
            if wide {
                items.join(",")
            } else {
                items.len().to_string()
            }
        }
        other => scalar(other),
    }
}

fn expanded_list(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "-".into();
    };
    match value {
        Value::Array(items) => items.iter().map(scalar).collect::<Vec<_>>().join(","),
        Value::String(text) if text.is_empty() || text == "N/A" => "-".into(),
        Value::String(text) => text.split(',').collect::<Vec<_>>().join(","),
        other => scalar(other),
    }
}

fn arn_list(value: Option<&Value>, wide: bool, count_in_table: bool) -> String {
    let Some(value) = value else {
        return "-".into();
    };
    let values: Vec<String> = match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.as_str().map(str::to_string).or_else(|| {
                    item.get("TargetGroupArn")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
            })
            .collect(),
        Value::String(text) if !text.is_empty() && text != "N/A" => vec![text.clone()],
        _ => return "-".into(),
    };
    if values.is_empty() {
        return "-".into();
    }
    if count_in_table && !wide {
        return values.len().to_string();
    }
    let names = values
        .iter()
        .map(|value| {
            if wide {
                value.as_str()
            } else {
                arn_tail(value)
            }
        })
        .collect::<Vec<_>>();
    names.join(",")
}

fn launch_template(row: &Value) -> String {
    let template = row.get("LaunchTemplate");
    let name = template.and_then(|v| v.get("Name")).map(scalar);
    let version = template.and_then(|v| v.get("Version")).map(scalar);
    match (name, version) {
        (Some(name), Some(version)) if name != "-" && version != "-" => format!("{name}@{version}"),
        (Some(name), _) if name != "-" => name,
        _ => row
            .get("LaunchConfiguration")
            .map(scalar)
            .unwrap_or_else(|| "-".into()),
    }
}

fn actions(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "-".into();
    };
    let actions = value
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(std::slice::from_ref(value));
    actions
        .iter()
        .map(|action| {
            let action_type = action
                .get("Type")
                .and_then(Value::as_str)
                .unwrap_or("action");
            let target = action
                .get("Config")
                .and_then(|config| {
                    config
                        .get("TargetGroupArn")
                        .and_then(Value::as_str)
                        .or_else(|| {
                            config
                                .as_object()
                                .and_then(|object| object.keys().next().map(String::as_str))
                        })
                })
                .map(arn_tail);
            let target = target.or_else(|| {
                action
                    .get("Config")
                    .and_then(Value::as_object)
                    .and_then(|config| config.keys().next().map(String::as_str))
                    .map(arn_tail)
            });
            if action_type == "forward" {
                format!("forward -> {}", target.unwrap_or("-"))
            } else {
                action
                    .get("Config")
                    .map(scalar)
                    .unwrap_or_else(|| action_type.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn route53_value(row: &Value) -> String {
    if let Some(values) = row.get("Values").and_then(Value::as_array) {
        if values.is_empty() {
            return "-".into();
        }
        return values
            .iter()
            .map(|value| display(value, false))
            .collect::<Vec<_>>()
            .join("\n");
    }
    row.get("Target")
        .map(|v| compact(Some(v)))
        .unwrap_or_else(|| "-".into())
}

fn compact(value: Option<&Value>) -> String {
    value
        .map(|v| display(v, false))
        .unwrap_or_else(|| "-".into())
}
