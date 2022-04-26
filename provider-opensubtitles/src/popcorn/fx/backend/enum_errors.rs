use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum EnumError {
    NotFound { value: String, enum_type: String }
}

impl Display for EnumError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EnumError::NotFound {value, enum_type} =>
                write!(f, "Enum value {value} not found for {enum_type}"),
        }
    }
}