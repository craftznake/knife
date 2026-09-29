use super::super::arg::OutputFormat;
use serde_json::Value;
use std::collections::BTreeSet;

pub fn render(value: &Value, format: OutputFormat) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(value)?),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(value)?),
        OutputFormat::Table | OutputFormat::Wide => {
            let rows = match value {
                Value::Array(rows) => rows.as_slice(),
                Value::Object(_) => std::slice::from_ref(value),
                _ => return Ok(value.to_string()),
            };
            if rows.is_empty() {
                return Ok(String::new());
            }
            let mut columns = BTreeSet::new();
            for row in rows {
                if let Value::Object(object) = row {
                    columns.extend(object.keys().cloned());
                }
            }
            let mut columns: Vec<_> = columns.into_iter().collect();
            if format == OutputFormat::Table {
                columns.retain(|key| !key.starts_with('_'));
            }
            let mut lines = vec![columns.join("\t")];
            for row in rows {
                lines.push(
                    columns
                        .iter()
                        .map(|key| match &row[key] {
                            Value::Null => String::new(),
                            Value::String(value) => value.clone(),
                            value => value.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join("\t"),
                );
            }
            Ok(lines.join("\n"))
        }
    }
}
