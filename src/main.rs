use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut command = String::new();
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let _ = stdin.read_line(&mut command)?;
        command.pop();
        println!("{command}: command not found");
        command.clear();
    }
}
