use crate::command_executor::CommandExecutor;
use crate::link_executor::LinkExecutor;
use crate::schema::Operation;

mod unix;

pub trait ExecutionPlatform: CommandExecutor + LinkExecutor {
    fn execute(&self, operation: &Operation) -> Result<(), Box<dyn std::error::Error>> {
        match operation {
            Operation::Command {
                command,
                as_root,
                args,
            } => {
                self.execute_command(command, as_root, args)?;
            }
            Operation::Link { original, link } => {
                self.execute_link(original, link)?;
            }
        }

        Ok(())
    }
}

pub fn construct_execution_platform() -> Box<dyn ExecutionPlatform> {
    match std::env::consts::OS {
        "linux" | "macos" => Box::new(unix::UnixExecutionPlatform),
        _ => unimplemented!(),
    }
}
