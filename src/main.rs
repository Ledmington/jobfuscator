use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read};

fn main() -> io::Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    let file = File::open(&filename)?;
    let mut reader = BufReader::new(file);
    let mut file_bytes: Vec<u8> = vec![0u8; reader.capacity()];
    reader.read_to_end(&mut file_bytes)?;

    for i in 0..file_bytes.len() {
        let b = file_bytes[i];
        print!(" {:#04x}", b);
        if i % 16 == 15 {
            println!();
        }
    }
    println!();

    Ok(())
}
