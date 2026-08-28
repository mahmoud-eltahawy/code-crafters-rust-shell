use std::fmt::Display;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::str::FromStr;
use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::parser::{parse_builtin_command_name, parse_command};
mod parser;

fn init_executables() -> io::Result<Vec<PathBuf>> {
    let paths = std::env::var("PATH").unwrap();
    let paths = paths.split(':').map(|x| x.parse::<PathBuf>().unwrap());
    let mut executables = Vec::new();
    for path in paths {
        let enteries = std::fs::read_dir(path)?;
        for entry in enteries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let exec = metadata.is_file() && (metadata.permissions().mode() & 0o111) != 0;
            if exec {
                executables.push(entry.path());
            }
        }
    }
    Ok(executables)
}

fn main() -> io::Result<()> {
    let executables = init_executables()?;

    let mut pwd = PathBuf::from_str("./").unwrap().canonicalize().unwrap();

    let mut command_buf = String::new();
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let _ = stdin.read_line(&mut command_buf)?;
        command_buf.pop();

        let exec_non_builtins = |command: String, args: Vec<String>| {
            let exec = executables.iter().find(|x| {
                x.file_name().is_some_and(|x| {
                    x.len() <= command.len() && *x.to_str().unwrap() == command[..x.len()]
                })
            });
            match exec {
                Some(_) => {
                    let output = Command::new(command).args(args).output().unwrap().stdout;
                    let output = String::from_utf8(output).unwrap();

                    print!("{output}");
                }
                None => {
                    println!("{command}: not found");
                }
            };
        };

        let (_, command) = parse_command(&command_buf).unwrap();
        match command {
            ShellCommand::Exit => {
                break;
            }
            ShellCommand::Echo(txt) => {
                let txt = txt
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("");
                println!("{txt}",);
            }
            ShellCommand::Type(ref command) => {
                let (_, c) = parse_builtin_command_name(command).unwrap();
                let c = c.to_string().to_lowercase();
                if !c.is_empty() {
                    println!("{c} is a shell builtin");
                }
            }
            ShellCommand::Nothing => (),
            ShellCommand::Foreign { command, args } => exec_non_builtins(command, args),
            ShellCommand::Pwd => {
                let mut pwd = pwd.display().to_string();
                if pwd.ends_with('/') {
                    pwd.pop();
                };
                println!("{}", pwd);
            }
            ShellCommand::Cd(path_buf) => {
                if path_buf.is_absolute() && path_buf.exists() {
                    pwd = path_buf;
                } else {
                    let mut pwd2 = pwd.clone();
                    pwd2.push(path_buf.clone());
                    if let Ok(pwd2) = pwd2.canonicalize() {
                        pwd = pwd2;
                    } else {
                        println!("cd: {}: No such file or directory", path_buf.display())
                    };
                }
            }
        };

        command_buf.clear();
    }
    Ok(())
}

#[derive(Debug)]
enum ShellCommand {
    Nothing,
    Exit,
    Echo(Vec<Word>),
    Type(String),
    Cd(PathBuf),
    Pwd,
    Foreign { command: String, args: Vec<String> },
}

#[derive(Debug)]
enum Word {
    Quated(String),
    NonQuated(String),
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            Word::Quated(x) => x,
            Word::NonQuated(x) => x,
        };
        write!(f, "{x}")
    }
}
