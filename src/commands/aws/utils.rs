use std::{fs, path::PathBuf};

use aws_sdk_sts::primitives::DateTime as AwsDateTime;
use chrono::{DateTime, Local, TimeZone, Utc};

pub fn aws_datetime_to_local(aws_dt: &AwsDateTime) -> DateTime<Local> {
    let utc_datetime = Utc
        .timestamp_opt(aws_dt.secs(), aws_dt.subsec_nanos())
        .unwrap();
    utc_datetime.with_timezone(&Local)
}

/// Get the knife config directory path (~/.knife)
pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").map_err(|_| -> Box<dyn std::error::Error> {
        format!("HOME environment variable not set").into()
    })?;

    let config_dir = PathBuf::from(home).join(".knife");

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| -> Box<dyn std::error::Error> {
            format!("Failed to create config directory: {}", e).into()
        })?;
    }

    Ok(config_dir)
}
