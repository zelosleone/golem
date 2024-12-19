use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    Literal(String),
    Parameter(String),
    CatchAll(String),
}

#[derive(Debug, Clone)]
pub struct PathPattern {
    segments: Vec<PathSegment>,
}

impl PathPattern {
    pub fn parameters(&self) -> Vec<String> {
        self.segments
            .iter()
            .filter_map(|segment| match segment {
                PathSegment::Parameter(name) | PathSegment::CatchAll(name) => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn is_catch_all(&self, param_name: &str) -> bool {
        self.segments.iter().any(|segment| {
            matches!(segment, PathSegment::CatchAll(name) if name == param_name)
        })
    }
}

#[derive(Debug, Clone)]
pub struct AllPathPatterns {
    patterns: Vec<PathPattern>,
}

impl AllPathPatterns {
    pub fn parse(path: &str) -> Result<Self, String> {
        let patterns = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if segment.starts_with('{') && segment.ends_with('}') {
                    let param_name = &segment[1..segment.len() - 1];
                    if param_name.ends_with("..") {
                        Ok(PathSegment::CatchAll(param_name[..param_name.len() - 2].to_string()))
                    } else {
                        Ok(PathSegment::Parameter(param_name.to_string()))
                    }
                } else {
                    Ok(PathSegment::Literal(segment.to_string()))
                }
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self {
            patterns: vec![PathPattern { segments: patterns }],
        })
    }

    pub fn parameters(&self) -> Vec<String> {
        self.patterns
            .iter()
            .flat_map(|pattern| pattern.parameters())
            .collect()
    }

    pub fn is_catch_all(&self, param_name: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.is_catch_all(param_name))
    }
}

impl FromStr for AllPathPatterns {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_path() {
        let patterns = AllPathPatterns::parse("/api/v1/users/{id}").unwrap();
        let params = patterns.parameters();
        assert_eq!(params, vec!["id"]);
    }

    #[test]
    fn test_parse_catch_all() {
        let patterns = AllPathPatterns::parse("/api/files/{path..}").unwrap();
        assert!(patterns.is_catch_all("path"));
    }

    #[test]
    fn test_parse_multiple_parameters() {
        let patterns = AllPathPatterns::parse("/api/users/{userId}/posts/{postId}").unwrap();
        let params = patterns.parameters();
        assert_eq!(params, vec!["userId", "postId"]);
    }
}
