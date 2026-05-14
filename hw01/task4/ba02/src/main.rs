use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut lines = 0;
    let mut words = 0;
    let mut bytes = 0;

    for line in stdin.lock().lines() {
        let line = line?;
        bytes += line.len() + 1; // +1 для символа \n
        lines += 1;

        // Подсчёт слов: разбиваем строку по пробельным символам и фильтруем пустые строки
        words += line.split_whitespace().count();
    }

    println!("{} {} {}", lines, words, bytes);
    Ok(())
}
