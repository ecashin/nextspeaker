use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct AppMode {
    pub value: crate::Mode,
}

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq, Serialize, Store)]
#[store(storage = "local")]
pub struct Candidates {
    pub value: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq, Serialize, Store)]
#[store(storage = "local")]
pub struct History {
    pub value: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize, Store)]
#[store(storage = "local")]
pub struct HistoryHalflife {
    pub numerator: i64,
    pub denominator: i64,
}

impl Default for HistoryHalflife {
    fn default() -> Self {
        Self {
            numerator: 10,
            denominator: 1,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct Selected {
    pub value: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct SimulationResults {
    pub value: Option<Vec<(String, u64)>>,
}
