use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut buf = [0; 1024];
    let mut stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let len = stdin.read(&mut buf)?;
        let command = String::from_utf8(buf[..len].to_vec()).unwrap();
        println!("{command}: command not found");
    }
}
