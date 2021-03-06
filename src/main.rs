mod argument_resolver;
mod command_executor;
mod environment_updater;
mod execution_platform;
mod link_executor;
mod schema;

use clap::Parser;
use std::fs::File;

use execution_platform::construct_execution_platform;
use schema::Schema;

#[derive(Parser)]
struct CommandLineArg {
    #[clap(parse(from_os_str))]
    scheme_file_path: std::path::PathBuf,
    platform: String,
}

#[derive(Debug)]
struct ExecutionError(Option<i32>);

#[derive(Debug)]
struct InstallResult {
    success: usize,
    failure: usize,
    skip: usize,
}

impl InstallResult {
    fn new() -> Self {
        Self {
            success: 0,
            failure: 0,
            skip: 0,
        }
    }

    fn success(&mut self) {
        self.success += 1;
    }
    fn failure(&mut self) {
        self.failure += 1;
    }
    fn skip(&mut self) {
        self.skip += 1;
    }
}

impl std::fmt::Display for InstallResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} success, {} skip, {} failure",
            self.success, self.skip, self.failure
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CommandLineArg::parse();

    let file = File::open(args.scheme_file_path)?;
    let execution_platform = construct_execution_platform();
    let mut schema = serde_yaml::from_reader::<File, Schema>(file)?;

    schema.expand()?;

    let mut install_result = InstallResult::new();

    let apps = &schema.application;
    for app in apps {
        if let Some(recipe) = app.resolve_recipe(&args.platform) {
            println!("install `{}`", app.name());

            let mut failed = false;
            if recipe.skip_if.is_some() {
                match execution_platform.app_already_installed(recipe.skip_if.as_ref().unwrap()) {
                    Ok(installed) => {
                        if installed {
                            install_result.skip();
                            println!("`{}` is already installed. skip...", app.name());
                            continue;
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                        failed = true
                    }
                }
            }

            if !failed {
                for (i, operation) in ((&recipe.operations).iter()).enumerate() {
                    println!("STEP: {}", i + 1);
                    if let Err(e) = execution_platform.execute(operation) {
                        println!("{}", e);
                        failed = true;
                        break;
                    };
                    println!();
                }
            }

            if failed {
                install_result.failure();
                println!("fail to install `{}`", app.name());
            } else {
                install_result.success();
                println!("succeed to install `{}`", app.name());
            }
        }
    }

    println!("{}", install_result);

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
