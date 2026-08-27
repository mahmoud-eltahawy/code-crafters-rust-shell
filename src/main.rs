use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut command = String::new();
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let _ = stdin.read_line(&mut command)?;
        command.pop();
        match command.as_str() {
            "exit" => break,
            other if other.starts_with("echo") => {
                let txt = other.strip_prefix("echo ").unwrap_or_default();
                println!("{txt}");
            }
            _ => {
                println!("{command}: command not found");
            }
        }
        command.clear();
    }
    Ok(())
}
