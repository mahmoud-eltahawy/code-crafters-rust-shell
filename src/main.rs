use std::os::unix::fs::PermissionsExt;
use std::{
    io::{self, Write},
    path::PathBuf,
};

fn main() -> io::Result<()> {
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

    let mut command_buf = String::new();
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let _ = stdin.read_line(&mut command_buf)?;
        command_buf.pop();

        let handle_non_builtins = |command: String| {
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
                    println!("{command}: command not found");
                }
            };
        };
        let command = BuiltinCommand::from(command_buf.clone());
        match command {
            BuiltinCommand::Exit => {
                break;
            }
            BuiltinCommand::Echo(txt) => {
                println!("{txt}");
            }
            BuiltinCommand::Type(ref command) => {
                let c = BuiltinCommand::from(command.clone());
                let c = match c {
                    BuiltinCommand::Exit => Some("exit".to_string()),
                    BuiltinCommand::Echo(_) => Some("echo".to_string()),
                    BuiltinCommand::Type(_) => Some("type".to_string()),
                    BuiltinCommand::Foreign { command, .. } => {
                        handle_non_builtins(command);
                        None
                    }
                    BuiltinCommand::Nothing => None,
                };
                if let Some(c) = c {
                    println!("{c} is a shell builtin");
                }
            }
            BuiltinCommand::Nothing => (),
            BuiltinCommand::Foreign { command, .. } => handle_non_builtins(command),
        };

        command_buf.clear();
    }
    Ok(())
}

#[derive(Debug)]
enum BuiltinCommand {
    Nothing,
    Exit,
    Echo(String),
    Type(String),
    Foreign { command: String, _args: Vec<String> },
}

impl From<String> for BuiltinCommand {
    fn from(value: String) -> Self {
        let value = value.trim();
        let mut args = value.split_whitespace();
        let command = args.next();
        let args = args.collect::<Vec<_>>();
        match command {
            Some(command) => match command {
                "exit" => Self::Exit,
                "echo" => Self::Echo(args.join(" ")),
                "type" => Self::Type(args.first().unwrap().to_string()),
                _ => Self::Foreign {
                    command: command.to_string(),
                    _args: args.into_iter().map(|x| x.to_string()).collect(),
                },
            },
            None => Self::Nothing,
        }
    }
}
