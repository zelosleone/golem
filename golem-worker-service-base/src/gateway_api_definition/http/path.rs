use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct AllPathPatterns(String);

impl AllPathPatterns {
    pub fn parse(path: &str) -> Result<Self, String> {
        // Add validation logic here if needed
        Ok(AllPathPatterns(path.to_string()))
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Display for AllPathPatterns {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AllPathPatterns {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
