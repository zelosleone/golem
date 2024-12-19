use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MethodPattern {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
    Connect,
}

impl TryFrom<i32> for MethodPattern {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MethodPattern::Get),
            1 => Ok(MethodPattern::Post),
            2 => Ok(MethodPattern::Put),
            3 => Ok(MethodPattern::Delete),
            4 => Ok(MethodPattern::Patch),
            5 => Ok(MethodPattern::Head),
            6 => Ok(MethodPattern::Options),
            7 => Ok(MethodPattern::Trace),
            8 => Ok(MethodPattern::Connect),
            _ => Err(format!("Invalid HTTP method code: {}", value)),
        }
    }
}
