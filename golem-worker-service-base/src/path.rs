use std::fmt::Display;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct Path(pub Vec<PathComponent>);

impl Path {
    pub fn from_key(input: &str) -> Path {
        let mut path = Path::default();
        path.update_key(input);
        path
    }

    pub fn from_index(index: usize) -> Path {
        let mut path = Path::default();
        path.update_index(index);
        path
    }

    pub fn update_key(&mut self, input: &str) {
        self.0.push(PathComponent::key_name(input));
    }

    pub fn update_index(&mut self, index: usize) {
        self.0.push(PathComponent::index(index));
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut path = "".to_string();

        for p in self.0.clone() {
            match p {
                PathComponent::Index(index) => path.push_str(format!("[{}]", index.0).as_str()),
                PathComponent::KeyName(keyname) => {
                    path.push_str(format!(".{}", keyname.0).as_str())
                }
            }
        }

        write!(f, "{}", path.trim_start_matches('.'))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct Index(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct KeyName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum PathComponent {
    Index(Index),
    KeyName(KeyName),
}

impl PathComponent {
    fn key_name(input: &str) -> PathComponent {
        PathComponent::KeyName(KeyName(input.to_string()))
    }

    fn index(index: usize) -> PathComponent {
        PathComponent::Index(Index(index))
    }
}