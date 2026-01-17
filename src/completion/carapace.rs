use anyhow::Result;
use log::debug;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
pub struct CarapaceItem {
    #[serde(rename = "value")]
    pub value: String,
    #[serde(rename = "display")]
    pub display: String,
    #[serde(rename = "description")]
    pub description: Option<String>,
    #[serde(rename = "style")]
    pub style: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct CarapaceOutput {
    #[serde(rename = "values")]
    pub values: Vec<CarapaceItem>,
}

pub struct CarapaceProvider;

impl CarapaceProvider {
    pub fn fetch_suggestions(cmd_name: &str, args: &[String]) -> Result<Option<Vec<CarapaceItem>>> {
        let mut command = Command::new("carapace");
        command.arg(cmd_name).arg("export");

        debug!("cmd_name: {cmd_name}, args: {:?}", args);

        for arg in args {
            command.arg(arg);
        }

        let output = match command.output() {
            Ok(o) => o,
            Err(_) => return Ok(None),
        };

        if !output.status.success() {
            return Ok(None);
        }

        let output_str = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(e) => {
                debug!("Carapace output is not valid UTF-8: {}", e);
                return Ok(None);
            }
        };

        let output: CarapaceOutput = match serde_json::from_str(&output_str) {
            Ok(o) => o,
            Err(e) => {
                debug!("Failed to parse carapace output: {}", e);
                debug!("Carapace output was: {}", output_str);
                return Ok(None);
            }
        };

        Ok(Some(output.values))
    }
}
