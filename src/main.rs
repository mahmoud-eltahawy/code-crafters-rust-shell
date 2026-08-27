use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut buf = [0; 1024];
    let mut stdin = io::stdin();
    loop {
        print!("$ ");
        let len = stdin.read(&mut buf)?;
        let command = String::from_utf8(buf[..len].to_vec()).unwrap();
        print!("{command}: command not found");
        io::stdout().flush().unwrap();
    }
}
