use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug)]
pub enum ConfigError {
    InvalidValue(String, String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidValue(value, field) => write!(f, "Invalid value {} given for {}", value, field),
        }
    }
}