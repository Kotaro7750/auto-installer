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
    // powershell.exeのStart-Processコマンドレットを使って「New-Itemコマンドレットを実行するpowershell.exe」を管理者権限で起動する
    fn create_link(&self, original: &Path, link: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new("powershell.exe");
        command.arg("-NoProfile");
        command.arg("-Command");

        // \"だと囲えない
        // 恐らく呼び出しスタックのどこかで勝手に解釈されてしまったのだと思う
        // そのため\という文字自体を渡してやることで一段解釈を先送りにする
        let inner_command = format!(
            "New-Item -ItemType SymbolicLink -Path \\\"{}\\\" -Value \\\"{}\\\"",
            link.display(),
            original.display()
        );

        command.arg(format!("Start-Process 'powershell.exe' -Wait -PassThru -Verb RunAs -ArgumentList '-NoProfile','-Command','{}'",inner_command));

        // XXX inner_commandが失敗しても失敗と判定されていない
        let status = command.status()?;

        if status.success() {
            Ok(())
        } else {
            Err(Box::new(ExecutionError(status.code())))
        }
    }
}

impl ExecutionPlatform for WindowsExecutionPlatform {}
