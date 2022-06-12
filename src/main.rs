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
        args: Option<Vec<String>>,
    },
    Link {
        original: String,
        link: String,
    },
}

#[derive(Debug)]
struct ExecutionError(Option<i32>);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CommandLineArg::parse();

    let file = File::open(args.scheme_file_path)?;
    let apps = serde_yaml::from_reader::<File, Vec<Application>>(file)?;

    for app in apps {
        if let Some(recipe) = app.recipe.get(&args.platform) {
            println!("install `{}`", app.name);

            let mut failed = false;
            for (i, operation) in ((&recipe.operations).iter()).enumerate() {
                println!("STEP: {}", i + 1);
                if let Err(_) = operation.execute() {
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

impl Operation {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Command { command, args } => {
                Self::execute_command(command, args)?;
            }
            Self::Link { original, link } => {
                Self::execute_link(original, link)?;
            }
        }

        Ok(())
    }

    fn execute_command(
        command: &String,
        args: &Option<Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new(command);
        let mut command = command.stdout(std::process::Stdio::inherit());

        if let Some(args) = args {
            command = command.args(args);
        }

        println!("execute `{:?}`", command);

        let exit_status = command.status()?;

        if exit_status.success() {
            Ok(())
        } else {
            Err(Box::new(ExecutionError(exit_status.code())))
        }
    }

    fn execute_link(original: &String, link: &String) -> Result<(), Box<dyn std::error::Error>> {
        let original_path = PathBuf::from(original);
        // 実際にはファイルの存在だけではなくmetadataの取得に必要なパーミッションがないときにもエラーを出す
        // これがなかったらどうせ現在のユーザーが読み取れないのでエラーにしてもよいはず
        // cf. https://doc.rust-lang.org/std/fs/fn.metadata.html#errors
        std::fs::metadata(&original_path)?;

        let link_path = PathBuf::from(link);
        // 今から張るリンクは存在してはならないが存在しているとリンクを張る段階でエラーが出るはず

        println!("create symlink original: `{}` link: `{}`", original, link);

        match std::env::consts::OS {
            "linux" | "macos" => Self::create_link_unix(&original_path, &link_path),
            "windows" => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    fn create_link_unix(original: &PathBuf, link: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        std::os::unix::fs::symlink(original, link)?;
        Ok(())
    }
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
