use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
enum SchemaError {
    PlatformConfigNotFound,
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::PlatformConfigNotFound => write!(f, "platform config not found"),
        }
    }
}
impl std::error::Error for SchemaError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub platform_config: HashMap<String, PlatformConfig>,
    pub application: Vec<Application>,
}

impl Schema {
    pub fn expand(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for app in self.application.iter_mut() {
            for (platform, recipe) in app.recipe.iter_mut() {
                if let PlatformSpecificRecipe::ConcreteRecipe(ref mut concrete_recipe) = recipe {
                    let mut expandee = concrete_recipe.find_expandee();

                    // 後ろから展開することでインデックスがずれるのを防ぐ
                    // 前からだと複数のOperationに展開した際にインデックスがずれてしまう
                    expandee.sort_by(|a, b| b.0.cmp(&a.0));

                    if !expandee.is_empty() {
                        let platform_config =
                            match Self::resolve_platform_config(&self.platform_config, platform) {
                                Some(pc) => pc,
                                None => return Err(Box::new(SchemaError::PlatformConfigNotFound)),
                            };

                        for (i, package_name) in expandee.iter() {
                            let operations =
                                platform_config.construct_package_install_operations(package_name);

                            concrete_recipe.operations.splice(i..&(i + 1), operations);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn resolve_platform_config<'a>(
        platform_config: &'a HashMap<String, PlatformConfig>,
        platform: &String,
    ) -> Option<&'a ConcretePlatformConfig> {
        if let Some(pc) = platform_config.get(platform) {
            match pc {
                PlatformConfig::SameWith { same_with } => {
                    Self::resolve_platform_config(platform_config, same_with)
                }
                PlatformConfig::ConcretePlatformConfig(cf) => Some(cf),
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlatformConfig {
    SameWith { same_with: String },
    ConcretePlatformConfig(ConcretePlatformConfig),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConcretePlatformConfig {
    pub package_install: Vec<Operation>,
}

impl ConcretePlatformConfig {
    fn construct_package_install_operations(
        &self,
        package_name: impl AsRef<str>,
    ) -> Vec<Operation> {
        let mut operations = self.package_install.clone();

        for operation in operations.iter_mut() {
            // パッケージ名はコマンドの引数のみに表れる
            if let Operation::Command(ref mut command_config) = operation {
                if let Some(args) = &mut command_config.args {
                    let mut new_args = Vec::<Argument>::new();

                    for arg in args {
                        let mut new_arg = arg.clone();
                        if let Argument::String(arg_string) = &new_arg {
                            if arg_string == "${package}" {
                                new_arg = Argument::String(package_name.as_ref().to_string());
                            }
                        }

                        new_args.push(new_arg);
                    }

                    command_config.args = Some(new_args);
                }
            }
        }

        operations
    }
}

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

impl ConcreteRecipe {
    fn find_expandee(&self) -> Vec<(usize, String)> {
        let mut expandee = Vec::<(usize, String)>::new();
        for (i, operation) in self.operations.iter().enumerate() {
            if let Operation::PackageInstall { package_name } = operation {
                expandee.push((i, package_name.clone()));
            }
        }

        expandee
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathStr(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Argument {
    Path { path: PathStr },
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Operation {
    Command(CommandConfig),
    Link { original: PathStr, link: PathStr },
    PackageInstall { package_name: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandConfig {
    pub command: Argument,
    pub as_root: Option<bool>,
    pub args: Option<Vec<Argument>>,
}
