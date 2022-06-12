use std::path::PathBuf;
use std::process::Command;

use crate::command_executor::CommandExecutor;
use crate::execution_platform::ExecutionPlatform;
use crate::link_executor::LinkExecutor;
use crate::schema::CommandConfig;

pub struct UnixExecutionPlatform;

impl CommandExecutor for UnixExecutionPlatform {
    fn construct_command(&self, comman_config: &CommandConfig) -> Command {
        let mut command: Command;

        if let Some(true) = comman_config.as_root {
            command = Command::new("sudo");
            command.arg(comman_config.command.clone());
        } else {
            command = Command::new(comman_config.command.clone());
        }

        if let Some(ref args) = comman_config.args {
            command.args(args);
        }

        command
    }
}

impl LinkExecutor for UnixExecutionPlatform {
    fn create_link(
        &self,
        original: &PathBuf,
        link: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::os::unix::fs::symlink(original, link)?;
        Ok(())
    }
}

impl ExecutionPlatform for UnixExecutionPlatform {}
