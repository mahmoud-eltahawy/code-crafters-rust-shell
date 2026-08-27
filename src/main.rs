use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::str::FromStr;
use std::{
    io::{self, Write},
    path::PathBuf,
};

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

        let type_non_builtins = |command: String| {
            let exec = executables.iter().find(|x| {
                x.file_name().is_some_and(|x| {
                    x.len() <= command.len() && *x.to_str().unwrap() == command[..x.len()]
                })
            });
            match exec {
                Some(exec) => println!(
                    "{} is {}",
                    exec.file_name().unwrap().to_str().unwrap(),
                    exec.display()
                ),
                None => {
                    println!("{command}: not found");
                }
            };
        };
        let command = ShellCommand::from(command_buf.clone());
        match command {
            ShellCommand::Exit => {
                break;
            }
            ShellCommand::Echo(txt) => {
                println!("{txt}");
            }
            ShellCommand::Type(ref command) => {
                let c = ShellCommand::from(command.clone());
                let c = match c {
                    ShellCommand::Exit => Some("exit".to_string()),
                    ShellCommand::Echo(_) => Some("echo".to_string()),
                    ShellCommand::Type(_) => Some("type".to_string()),
                    ShellCommand::Foreign { command, .. } => {
                        type_non_builtins(command);
                        None
                    }
                    ShellCommand::Nothing => None,
                    ShellCommand::Pwd => Some("pwd".to_string()),
                    ShellCommand::Cd(_) => Some("cd".to_string()),
                };
                if let Some(c) = c {
                    println!("{c} is a shell builtin");
                }
            }
            ShellCommand::Nothing => (),
            ShellCommand::Foreign { command, args } => exec_non_builtins(command, args),
            ShellCommand::Pwd => {
                println!("{}", pwd.display());
            }
            ShellCommand::Cd(path_buf) => {
                if path_buf.is_absolute() && path_buf.exists() {
                    pwd = path_buf;
                } else if path_buf.is_relative() {
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
    Echo(String),
    Type(String),
    Cd(PathBuf),
    Pwd,
    Foreign { command: String, args: Vec<String> },
}

impl From<String> for ShellCommand {
    fn from(value: String) -> Self {
        let value = value.trim();
        let mut args = value.split_whitespace();
        let command = args.next();
        let args = args.collect::<Vec<_>>();
        match command {
            Some(command) => match command {
                "exit" => Self::Exit,
                "pwd" => Self::Pwd,
                "echo" => Self::Echo(args.join(" ")),
                "type" => Self::Type(args.first().map(|x| x.to_string()).unwrap_or_default()),
                "cd" => Self::Cd(args.first().map(|x| x.parse::<PathBuf>()).unwrap().unwrap()),
                _ => Self::Foreign {
                    command: command.to_string(),
                    args: args.into_iter().map(|x| x.to_string()).collect(),
                },
            },
            None => Self::Nothing,
        }
    }
}
