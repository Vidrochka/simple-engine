use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct LayerId(String);

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerId {
    pub fn new() -> LayerId {
        LayerId(Uuid::new_v4().to_string())
    }
}

impl From<&str> for LayerId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<Uuid> for LayerId {
    fn from(value: Uuid) -> Self {
        Self(value.to_string())
    }
}

impl std::ops::Deref for LayerId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}