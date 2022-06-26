use std::process::Stdio;

use crate::command_executor::CommandExecutor;
use crate::environment_updater::EnvironmentUpdater;
use crate::link_executor::LinkExecutor;
use crate::schema::CommandConfig;
use crate::schema::Operation;

mod target_platform;

pub trait ExecutionPlatform: CommandExecutor + LinkExecutor + EnvironmentUpdater {
    fn execute(&self, operation: &Operation) -> Result<(), Box<dyn std::error::Error>> {
        match operation {
            Operation::Command(command_config) => {
                self.execute_command(command_config)?;
            }
            Operation::Link { original, link } => {
                self.execute_link(original, link)?;
            }
        }

        self.update_current_environment();

        Ok(())
    }

    fn app_already_installed(
        &self,
        checker_command_config: &CommandConfig,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut checker_command = self.construct_command(checker_command_config);
        checker_command.stdout(Stdio::null());
        checker_command.stderr(Stdio::null());

        let status = checker_command.status()?;
        Ok(status.success())
    }
}

pub fn construct_execution_platform() -> Box<dyn ExecutionPlatform> {
    Box::new(target_platform::new())
}
