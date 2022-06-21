use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use crate::argument_resolver::ArgumentResolver;
use crate::command_executor::CommandExecutor;
use crate::execution_platform::ExecutionPlatform;
use crate::link_executor::LinkExecutor;
use crate::schema::CommandConfig;
use crate::ExecutionError;

pub struct WindowsExecutionPlatform;

pub fn new() -> WindowsExecutionPlatform {
    WindowsExecutionPlatform
}

impl ArgumentResolver for WindowsExecutionPlatform {}

impl CommandExecutor for WindowsExecutionPlatform {
    fn construct_command(&self, command_config: &CommandConfig) -> Command {
        let mut command = Command::new("powershell.exe");
        command.args(["-Command", "Start-Process"]);

        command.arg(self.resolve_argument(&command_config.command));

        if let Some(ref args) = command_config.args {
            command.arg("-ArgumentList");
            command.args(args.iter().map(|arg| self.resolve_argument(arg)));
        }

        command.arg("-Wait");

        if let Some(true) = command_config.as_root {
            command.arg("-Verb");
            command.arg("RunAs");
        }

        command
    }
}

impl LinkExecutor for WindowsExecutionPlatform {
    fn create_link(&self, original: &Path, link: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new("powershell.exe");
        command.arg("-Command");

        let mut inner_command  = OsString::from("Start-Process powershell.exe -ArgumentList '-Command','New-Item -ItemType SymbolicLink -Path ");
        inner_command.push(format!("\"{}\"", original.display()));
        inner_command.push(" -Value ");
        inner_command.push(format!("\"{}\"", link.display()));
        inner_command.push("'");

        command.arg(inner_command);
        command.args(["-Wait", "-Verb", "RusAs"]);

        println!("execute `{:?}`", command);

        let status = command.status()?;

        if status.success() {
            Ok(())
        } else {
            Err(Box::new(ExecutionError(status.code())))
        }
    }
}

impl ExecutionPlatform for WindowsExecutionPlatform {}
