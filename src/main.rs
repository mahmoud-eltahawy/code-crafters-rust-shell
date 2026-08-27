use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut command = String::new();
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let _ = stdin.read_line(&mut command)?;
        command.pop();
        let b = match BuiltinCommand::try_from(command.clone()) {
            Ok(b) => b,
            Err(_) => {
                println!("{command}: command not found");
                command.clear();
                continue;
            }
        };

        match b {
            BuiltinCommand::Exit => {
                break;
            }
            BuiltinCommand::Echo(txt) => {
                println!("{txt}");
            }
            BuiltinCommand::Type(ref command) => {
                match BuiltinCommand::try_from(command.clone()) {
                    Ok(c) => {
                        let c = match c {
                            BuiltinCommand::Exit => "exit",
                            BuiltinCommand::Echo(_) => "echo",
                            BuiltinCommand::Type(_) => "type",
                        };
                        println!("{c} is a shell builtin");
                    }
                    Err(_) => {
                        println!("{command}: command not found");
                    }
                };
            }
        }
        command.clear();
    }
    Ok(())
}

#[derive(Debug)]
enum BuiltinCommand {
    Exit,
    Echo(String),
    Type(String),
}

impl TryFrom<String> for BuiltinCommand {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value == "exit" {
            return Ok(Self::Exit);
        } else if value.starts_with("echo") {
            let txt = value.strip_prefix("echo ").unwrap_or_default();
            return Ok(Self::Echo(txt.to_string()));
        } else if value.starts_with("type") {
            let txt = value.strip_prefix("type ").unwrap_or_default();
            return Ok(Self::Type(txt.trim().to_string()));
        }
        Err("not found command")
    }
}
