use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
struct CommandLineArg {
    #[clap(parse(from_os_str))]
    scheme_file_path: std::path::PathBuf,
    platform: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Application {
    name: String,
    recipe: HashMap<String, PlatformSpecificRecipe>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlatformSpecificRecipe {
    operations: Vec<Operation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Operation {
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

#[derive(Debug)]
struct ExecutionError(Option<i32>);

trait ExecutionPlatform {
    fn execute(&self, operation: &Operation) -> Result<(), Box<dyn std::error::Error>> {
        match operation {
            Operation::Command {
                command,
                as_root,
                args,
            } => {
                self.execute_command(command, as_root, args)?;
            }
            Operation::Link { original, link } => {
                self.execute_link(original, link)?;
            }
        }

        Ok(())
    }

    fn construct_command(
        &self,
        command_str: &String,
        args: &Option<Vec<String>>,
        as_root: &Option<bool>,
    ) -> Command;

    fn execute_command(
        &self,
        command: &String,
        as_root: &Option<bool>,
        args: &Option<Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = self.construct_command(command, args, as_root);

        println!("execute `{:?}`", command);

        let exit_status = command.status()?;

        if exit_status.success() {
            Ok(())
        } else {
            Err(Box::new(ExecutionError(exit_status.code())))
        }
    }

    fn execute_link(
        &self,
        original: &String,
        link: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let original_path = PathBuf::from(original);
        // 実際にはファイルの存在だけではなくmetadataの取得に必要なパーミッションがないときにもエラーを出す
        // これがなかったらどうせ現在のユーザーが読み取れないのでエラーにしてもよいはず
        // cf. https://doc.rust-lang.org/std/fs/fn.metadata.html#errors
        std::fs::metadata(&original_path)?;

        let link_path = PathBuf::from(link);
        // 今から張るリンクは存在してはならないが存在しているとリンクを張る段階でエラーが出るはず

        println!("create symlink original: `{}` link: `{}`", original, link);

        self.create_link(&original_path, &link_path)
    }

    fn create_link(
        &self,
        original: &PathBuf,
        link: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

struct UnixExecutionPlatform;
impl ExecutionPlatform for UnixExecutionPlatform {
    fn create_link(
        &self,
        original: &PathBuf,
        link: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::os::unix::fs::symlink(original, link)?;
        Ok(())
    }

    fn construct_command(
        &self,
        command_str: &String,
        args: &Option<Vec<String>>,
        as_root: &Option<bool>,
    ) -> Command {
        let mut command: Command;

        if let Some(true) = as_root {
            command = Command::new("sudo");
            command.arg(command_str);
        } else {
            command = Command::new(command_str);
        }

        if let Some(args) = args {
            command.args(args);
        }

        command
    }
}

fn construct_execution_platform() -> Box<dyn ExecutionPlatform> {
    match std::env::consts::OS {
        "linux" | "macos" => Box::new(UnixExecutionPlatform),
        _ => unimplemented!(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CommandLineArg::parse();

    let file = File::open(args.scheme_file_path)?;
    let execution_platform = construct_execution_platform();
    let apps = serde_yaml::from_reader::<File, Vec<Application>>(file)?;

    for app in apps {
        if let Some(recipe) = app.recipe.get(&args.platform) {
            println!("install `{}`", app.name);

            let mut failed = false;
            for (i, operation) in ((&recipe.operations).iter()).enumerate() {
                println!("STEP: {}", i + 1);
                if let Err(_) = execution_platform.execute(operation) {
                    failed = true;
                    break;
                };
                println!("");
            }

            if failed {
                println!("fail to install `{}`", app.name);
            } else {
                println!("succeed to install `{}`", app.name);
            }
        }
    }

    Ok(())
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            Some(exit_code) => write!(f, "failed to execute: code `{}`", exit_code),
            None => write!(f, "failed to execute"),
        }
    }
}
impl std::error::Error for ExecutionError {}
