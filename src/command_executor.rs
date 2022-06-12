use crate::schema::CommandConfig;
use crate::ExecutionError;
use std::process::Command;

pub trait CommandExecutor {
    fn construct_command(&self, command_config: &CommandConfig) -> Command;

    fn execute_command(
        &self,
        command_config: &CommandConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = self.construct_command(command_config);

        println!("execute `{:?}`", command);

        let exit_status = command.status()?;

        if exit_status.success() {
            Ok(())
        } else {
            Err(Box::new(ExecutionError(exit_status.code())))
        }
    }
}
