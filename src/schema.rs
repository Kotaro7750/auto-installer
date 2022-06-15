use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    name: String,
    recipe: HashMap<String, PlatformSpecificRecipe>,
}

impl Application {
    pub fn resolve_recipe(&self, platform: &String) -> Option<&ConcreteRecipe> {
        if let Some(platform_recipe) = self.recipe.get(platform) {
            match platform_recipe {
                PlatformSpecificRecipe::SameWith { same_with } => self.resolve_recipe(same_with),
                PlatformSpecificRecipe::ConcreteRecipe(concrete_recipe) => Some(concrete_recipe),
            }
        } else {
            None
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlatformSpecificRecipe {
    SameWith { same_with: String },
    ConcreteRecipe(ConcreteRecipe),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConcreteRecipe {
    pub skip_if: Option<CommandConfig>,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathStr(pub String);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Path { path: PathStr },
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Operation {
    Command(CommandConfig),
    Link { original: PathStr, link: PathStr },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandConfig {
    pub command: Argument,
    pub as_root: Option<bool>,
    pub args: Option<Vec<Argument>>,
}
