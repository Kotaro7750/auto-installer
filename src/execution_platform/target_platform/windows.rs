use core::ffi::c_void;
use std::path::Path;
use std::process::Command;
use std::slice::from_raw_parts;
use windows_sys::Win32::{
    Security::TOKEN_QUERY,
    Storage::FileSystem::STANDARD_RIGHTS_READ,
    System::{
        Environment::{CreateEnvironmentBlock, SetEnvironmentVariableW},
        Threading::{GetCurrentProcess, OpenProcessToken},
    },
};

use crate::argument_resolver::ArgumentResolver;
use crate::command_executor::CommandExecutor;
use crate::execution_platform::EnvironmentUpdater;
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
        let mut command = Command::new(self.resolve_argument(&command_config.command));

        if let Some(ref args) = command_config.args {
            command.args(args.iter().map(|arg| self.resolve_argument(arg)));
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

        let inner_command = format!(
            "New-Item -ItemType SymbolicLink -Path \"{}\" -Value \"{}\"",
            link.display(),
            original.display()
        );

        command.arg(inner_command);

        let status = command.status()?;

        if status.success() {
            Ok(())
        } else {
            Err(Box::new(ExecutionError(status.code())))
        }
    }
}

impl EnvironmentUpdater for WindowsExecutionPlatform {
    fn update_current_environment(&self) {
        const TOKEN_READ: u32 = STANDARD_RIGHTS_READ | TOKEN_QUERY;

        unsafe {
            let process_handle = GetCurrentProcess();

            let mut token_handle = 0;
            if OpenProcessToken(process_handle, TOKEN_READ, &mut token_handle) == 0 {
                panic!("OpenProcessToken");
            }

            let mut env = std::ptr::null_mut::<c_void>();
            if CreateEnvironmentBlock(&mut env, token_handle, 0) == 0 {
                panic!("create env");
            }

            let mut ptr = env as *const u16;
            let mut i: usize = 0;
            loop {
                let wb = *ptr.offset(i.try_into().unwrap());
                if wb == 0 {
                    let arr = from_raw_parts(ptr, i);
                    let str = String::from_utf16(arr).unwrap();
                    if let Some((name, value)) = parse_environment_string(str) {
                        set_environment_variable(name, value);
                    }

                    ptr = ptr.offset((i + 1).try_into().unwrap());
                    i = 0;
                    if *ptr.offset(i.try_into().unwrap()) == 0 {
                        break;
                    }
                }
                i += 1;
            }
        }
    }
}

impl ExecutionPlatform for WindowsExecutionPlatform {}

fn parse_environment_string(entry: String) -> Option<(String, String)> {
    let v: Vec<&str> = entry.split('=').collect();

    if v.len() == 2 {
        Some((v[0].to_string(), v[1].to_string()))
    } else {
        None
    }
}

fn set_environment_variable(name: String, value: String) {
    let mut name_u16_arr: Vec<u16> = name.encode_utf16().collect();
    let mut value_u16_arr: Vec<u16> = value.encode_utf16().collect();

    // ここで0を追加するのはVecをそのまま使ってしまうとNULL終端されていない文字列を渡してしまうことになるから
    name_u16_arr.push(0);
    value_u16_arr.push(0);

    unsafe {
        if SetEnvironmentVariableW(name_u16_arr.as_ptr(), value_u16_arr.as_ptr()) == 0 {
            panic!("SetEnvironmentVariableW");
        }
    }
}
