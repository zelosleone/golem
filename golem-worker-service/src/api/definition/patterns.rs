use std::fmt;

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub key_name: String,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PathPattern {
    Literal(String),
    Var(VarInfo),
    CatchAllVar(VarInfo),
}

#[derive(Debug, Clone)]
pub struct AllPathPatterns {
    pub path_patterns: Vec<PathPattern>,
}

impl AllPathPatterns {
    pub fn parse(path: &str) -> Result<Self, String> {
        let mut patterns = Vec::new();
        let parts = path.split('/').filter(|s| !s.is_empty());

        for part in parts {
            if part.starts_with('{') && part.ends_with('}') {
                let var_name = part[1..part.len()-1].to_string();
                if var_name.starts_with('*') {
                    patterns.push(PathPattern::CatchAllVar(VarInfo {
                        key_name: var_name[1..].to_string(),
                        pattern: None,
                    }));
                } else {
                    patterns.push(PathPattern::Var(VarInfo {
                        key_name: var_name,
                        pattern: None,
                    }));
                }
            } else {
                patterns.push(PathPattern::Literal(part.to_string()));
            }
        }

        Ok(AllPathPatterns { path_patterns: patterns })
    }
}

impl fmt::Display for PathPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathPattern::Literal(s) => write!(f, "{}", s),
            PathPattern::Var(info) => write!(f, "{{{}}}", info.key_name),
            PathPattern::CatchAllVar(info) => write!(f, "{{*{}}}", info.key_name),
        }
    }
}
