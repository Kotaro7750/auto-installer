use crate::ExecutionError;
use std::process::Command;

pub trait CommandExecutor {
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
}
