use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_time_limit")]
    pub time_limit: u64, // ms
    #[serde(default = "default_stack_size")]
    pub stack_size: usize,
    #[serde(default = "default_output_size")]
    pub output_size: usize,
    #[serde(default = "default_time_check_interval")]
    pub time_check_interval: u64,
}

fn default_time_limit() -> u64 {
    2000
}

fn default_stack_size() -> usize {
    1024 * 1024 / 8 // 128k items (approx 1MB)
}

fn default_output_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_time_check_interval() -> u64 {
    1000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time_limit: default_time_limit(),
            stack_size: default_stack_size(),
            output_size: default_output_size(),
            time_check_interval: default_time_check_interval(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if Path::new("flap.toml").exists() {
            match fs::read_to_string("flap.toml") {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse flap.toml: {}", e);
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Failed to read flap.toml: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }
}
