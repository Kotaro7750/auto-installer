use std::path::Path;
use std::process::Command;

use crate::argument_resolver::ArgumentResolver;
use crate::command_executor::CommandExecutor;
use crate::execution_platform::ExecutionPlatform;
use crate::link_executor::LinkExecutor;
use crate::schema::CommandConfig;

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
        command.args(["-Command", "Start-Process", "mklink.exe", "-ArgumentList"]);

        if original.is_dir() {
            command.arg("/d");
        }

        command.args([link, original]);
        command.args(["-Wait", "-Verb", "RunAs"]);

        Ok(())
    }
}

impl ExecutionPlatform for WindowsExecutionPlatform {}
