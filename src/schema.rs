use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    pub recipe: HashMap<String, PlatformSpecificRecipe>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformSpecificRecipe {
    pub operations: Vec<Operation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Operation {
    Command {
        command: String,
        as_root: Option<bool>,
        args: Option<Vec<String>>,
    },
    Link {
        original: String,
        link: String,
    },
}
