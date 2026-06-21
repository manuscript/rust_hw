use std::io::{self, Write};

fn main() {
    let word_list_en = [
        "apple", "banana", "cherry", "date", "elderberry",
        "fig", "grape", "honeydew", "kiwi", "lemon",
        "mango", "nectarine", "orange", "papaya", "quince",
    ];

    let word_list_ru = [
        "яблоко", "банан", "вишня", "финик", "смородина",
        "груша", "виноград", "дыня", "киви", "лимон",
        "манго", "персик", "апельсин", "папайя", "айва",
    ];

    let rng_seed = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as usize;

    // Выбираем случайный список (0 — английский, 1 — русский)
    let (word_list, language_msg, expected_alphabet) = if rng_seed % 2 == 0 {
        (&word_list_en, "АНГЛИЙСКОЕ", "EN")
    } else {
        (&word_list_ru, "РУССКОЕ", "RU")
    };

    // Выбираем случайное слово из списка
    let index_word = rng_seed % word_list.len();
    let secret_word = word_list[index_word].to_string();

    let hidden_word = "_".repeat(secret_word.chars().count());

    let max_attempts = 6;
    let mut attempts_left = max_attempts;
    let mut guessed_letters = Vec::new();
    let mut current_state = hidden_word;

    println!("Добро пожаловать в игру 'Виселица'!");
    println!("Угадайте {} слово. Удачи!", language_msg);

    while attempts_left > 0 && current_state.contains('_') {
        print_game_state(&current_state, &guessed_letters, attempts_left);

        let letter = read_letter();
        if letter.is_none() {
            println!("Пожалуйста, введите ровно одну букву.");
            continue;
        }

        let letter = letter.unwrap();

        // Проверяем соответствие языка ввода
        let is_valid_letter = match expected_alphabet {
            "EN" => letter.is_ascii_alphabetic(),
            "RU" => {
                // Русские буквы в UTF-8 — это символы в диапазоне \u{0410}-\u{044F}
                let c = letter as u32;
                (c >= 0x0410 && c <= 0x044F) || (c >= 0x0401 && c <= 0x042F)
            },
            _ => false,
        };

        if !is_valid_letter {
            let expected = if expected_alphabet == "EN" { "английские" } else { "русские" };
            println!("Ожидаются только {} буквы. Попробуйте ещё раз.", expected);
            continue;
        }

        // Проверяем, не вводили ли эту букву раньше
        if guessed_letters.contains(&letter) {
            println!("Вы уже вводили букву '{}'. Попробуйте другую.", letter);
            continue;
        }

        guessed_letters.push(letter);

        if secret_word.contains(letter) {
            current_state = reveal_letter(&secret_word, &current_state, letter);
            println!("Есть такая буква!");
        } else {
            attempts_left -= 1;
            println!("Буквы '{}' нет в слове. Осталось попыток: {}", letter, attempts_left);
        }
    }

    // Итог игры
    let sw_translation = {if expected_alphabet == "EN" { word_list_ru } else { word_list_en }}[index_word].to_string();
    let title = if current_state.contains('_') { "Вы проиграли! Правильное слово было:" } else { "Поздравляем! Вы угадали слово:" };
    println!("\n{} {} ({})", title, secret_word.to_uppercase(), sw_translation);
}

fn print_game_state(word: &str, guessed: &Vec<char>, attempts: i32) {
    println!();
    println!("Слово: {}", word);
    println!("Уже введённые буквы: {}", guessed.iter().collect::<String>());
    println!("Осталось попыток: {}", attempts);
    print!("Введите букву: ");
    io::stdout().flush().expect("Ошибка flush");
}

fn read_letter() -> Option<char> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Ошибка чтения ввода");
    _read_letter(&mut input)
}

fn _read_letter(input: &mut String) -> Option<char> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.chars().count() != 1 {
        return None;
    }

    match trimmed.chars().next() {
        Some(c) if c.is_alphabetic() => Some( c.to_lowercase().next().unwrap_or(c)),
        _ => None,
    }
}

fn reveal_letter(secret: &str, current: &str, letter: char) -> String {
    let secret_chars: Vec<char> = secret.chars().collect();
    let current_chars: Vec<char> = current.chars().collect();

    let mut result = String::new();
    for (i, c) in secret_chars.iter().enumerate() {
        if *c == letter {
            result.push(*c);
        } else {
            result.push(current_chars[i]);
        }
    }
    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reveal_letter_single_match() {
        let secret = "cat";
        let current = "___";
        let letter = 'a';
        let result = reveal_letter(secret, current, letter);
        assert_eq!(result, "_a_");
    }

    #[test]
    fn test_reveal_letter_multiple_matches() {
        let secret = "banana";
        let current = "______";
        let letter = 'a';
        let result = reveal_letter(secret, current, letter);
        assert_eq!(result, "_a_a_a");
    }

    #[test]
    fn test_reveal_letter_no_match() {
        let secret = "dog";
        let current = "___";
        let letter = 'x';
        let result = reveal_letter(secret, current, letter);
        assert_eq!(result, "___");
    }

    #[test]
    fn test_read_letter_ascii_valid() {
        let mut input = String::from("a\n");
        let result = _read_letter(&mut input);
        assert_eq!(result, Some('a'));
    }

    #[test]
    fn test_read_letter_ascii_uppercase() {
        let mut input = String::from("A\n");
        let result = _read_letter(&mut input);
        assert_eq!(result, Some('a'));
    }

    #[test]
    fn test_read_letter_ru_valid() {
        let mut input = String::from("а\n");
        let result = _read_letter(&mut input);
        assert_eq!(result, Some('а'));
    }

    #[test]
    fn test_read_letter_ru_uppercase() {
        let mut input = String::from("А\n");
        let result = _read_letter(&mut input);
        assert_eq!(result, Some('а'));
    }

    #[test]
    fn test_read_letter_empty_input() {
        let mut input = String::from("\n");
        let result = _read_letter(&mut input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_letter_multiple_chars() {
        let mut input = String::from("ab\n");
        let result = _read_letter(&mut input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_letter_non_alphabetic() {
        let mut input1 = String::from("1\n");
        let result1 = _read_letter(&mut input1);
        assert_eq!(result1, None);

        let mut input2 = String::from("#\n");
        let result2 = _read_letter(&mut input2);
        assert_eq!(result2, None);
    }

    #[test]
    fn test_read_letter_mixed_case_ru() {
        let mut input = String::from("Б\n");
        let result = _read_letter(&mut input);
        assert_eq!(result, Some('б'));
    }

    #[test]
    fn test_is_valid_letter_english() {
        assert!(is_valid_letter('a', "EN"));
        assert!(is_valid_letter('Z', "EN"));
        assert!(!is_valid_letter('1', "EN"));
        assert!(!is_valid_letter('#', "EN"));
    }

    #[test]
    fn test_is_valid_letter_russian() {
        assert!(is_valid_letter('а', "RU"));
        assert!(is_valid_letter('Я', "RU"));
        assert!(!is_valid_letter('a', "RU"));
        assert!(!is_valid_letter('1', "RU"));
    }

    fn is_valid_letter(letter: char, alphabet: &str) -> bool {
        match alphabet {
            "EN" => letter.is_ascii_alphabetic(),
            "RU" => {
                let c = letter as u32;
                (c >= 0x0410 && c <= 0x044F) || (c >= 0x0401 && c <= 0x042F)
            },
            _ => false,
        }
    }

    #[test]
    fn test_print_game_state() {
        let word = "cat";
        let guessed = vec!['a', 't'];
        let attempts = 3;
        // Вдруг упадет
        print_game_state(word, &guessed, attempts);
    }

}
