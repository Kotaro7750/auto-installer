use std::path::PathBuf;
use std::process::Command;

use crate::command_executor::CommandExecutor;
use crate::execution_platform::ExecutionPlatform;
use crate::link_executor::LinkExecutor;

pub struct UnixExecutionPlatform;

impl CommandExecutor for UnixExecutionPlatform {
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
