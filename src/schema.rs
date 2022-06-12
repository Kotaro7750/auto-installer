use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    pub recipe: HashMap<String, PlatformSpecificRecipe>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformSpecificRecipe {
    pub skip_if: Option<CommandConfig>,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Operation {
    Command(CommandConfig),
    Link { original: String, link: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandConfig {
    pub command: String,
    pub as_root: Option<bool>,
    pub args: Option<Vec<String>>,
}
