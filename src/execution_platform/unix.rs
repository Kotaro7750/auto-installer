use std::path::Path;
use std::process::Command;

use crate::argument_resolver::ArgumentResolver;
use crate::command_executor::CommandExecutor;
use crate::execution_platform::ExecutionPlatform;
use crate::link_executor::LinkExecutor;
use crate::schema::CommandConfig;

pub struct UnixExecutionPlatform;

impl ArgumentResolver for UnixExecutionPlatform {}

impl CommandExecutor for UnixExecutionPlatform {
    fn construct_command(&self, command_config: &CommandConfig) -> Command {
        let mut command: Command;

        if let Some(true) = command_config.as_root {
            command = Command::new("sudo");
            command.arg(self.resolve_argument(&command_config.command));
        } else {
            command = Command::new(self.resolve_argument(&command_config.command));
        }

        if let Some(ref args) = command_config.args {
            command.args(args.iter().map(|arg| self.resolve_argument(arg)));
        }

        command
    }
}

impl LinkExecutor for UnixExecutionPlatform {
    fn create_link(
        &self,
        original: &Path,
        link: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::os::unix::fs::symlink(original, link)?;
        Ok(())
    }
}

impl ExecutionPlatform for UnixExecutionPlatform {}
