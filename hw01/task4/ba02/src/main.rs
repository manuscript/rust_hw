use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut buffer = Vec::new();
    
    // Читаем все данные из stdin до EOF
    stdin.read_to_end(&mut buffer)?;
    
    let mut lines = 0;
    let mut words = 0;
    let mut in_word = false;
    
    for &byte in &buffer {
        // Подсчёт строк: каждый символ \n увеличивает счётчик
        if byte == b'\n' {
            lines += 1;
        }
        
        // Подсчёт слов с отслеживанием состояния
        if byte.is_ascii_whitespace() {
            // Пробельный символ — выходим из слова (если были внутри)
            in_word = false;
        } else {
            // Непробельный символ
            if !in_word {
                // Начало нового слова
                words += 1;
                in_word = true;
            }
        }
    }
    
    let bytes = buffer.len();
    
    println!("{} {} {}", lines, words, bytes);
    Ok(())
}
