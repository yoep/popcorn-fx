use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug)]
pub enum ConfigError {
    InvalidValue(String, String),
    UnknownProvider(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidValue(value, field) => write!(f, "invalid value {} given for {}", value, field),
            ConfigError::UnknownProvider(name) => write!(f, "provider with name \"{}\" is unknown", name),
        }
    }
}