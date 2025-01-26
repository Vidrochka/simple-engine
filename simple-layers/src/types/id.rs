use std::fmt::Display;

use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct LayerId(Uuid);

impl Display for LayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl LayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn null() -> Self {
        Self(Uuid::nil())
    }
}
