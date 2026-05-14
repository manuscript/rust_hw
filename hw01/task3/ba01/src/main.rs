use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut buffer = Vec::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let bytes_read = handle.read_to_end(&mut buffer)?;

    println!("{}", bytes_read);
    Ok(())
}
