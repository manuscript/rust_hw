use std::io::{self, Write};

// --- Состояния конечного автомата ---
#[derive(Debug, PartialEq)]
enum GameState {
    WaitingForInput,
    CheckingGuess(char),
    Won,
    Lost,
}

// --- Структура состояния игры (контекст) ---
struct GameContext {
    secret_word: String,
    current_state: String,       // Например: "_a__b"
    guessed_letters: Vec<char>,
    attempts_left: i32,
    expected_alphabet: String,  // "EN" или "RU"
}

impl GameContext {
    fn new(secret: &str, alphabet: &str) -> Self {
        GameContext {
            secret_word: secret.to_string(),
            current_state: "_".repeat(secret.chars().count()),
            guessed_letters: Vec::new(),
            attempts_left: 6,
            expected_alphabet: alphabet.to_string(),
        }
    }

    // Проверка, является ли буква валидной для текущего алфавита
    fn is_valid_letter(&self, letter: char) -> bool {
        match self.expected_alphabet.as_str() {
            "EN" => letter.is_ascii_alphabetic(),
            "RU" => {
                let c = letter as u32;
                (c >= 0x0410 && c <= 0x044F) || (c >= 0x0401 && c <= 0x042F)
            },
            _ => false,
        }
    }

    fn process_guess(&mut self, letter: char) -> GameState {
        // 1. Проверка на дубликат
        if self.guessed_letters.contains(&letter) {
            println!("Вы уже вводили букву '{}'. Попробуйте другую.", letter);
            return GameState::WaitingForInput;
        }

        // 2. Добавляем букву только если это не дубликат
        self.guessed_letters.push(letter);

        // 3. Проверяем наличие буквы в слове
        if self.secret_word.contains(letter) {
            self.current_state = reveal_letter(&self.secret_word, &self.current_state, letter);
            println!("Есть такая буква!");

            if !self.current_state.contains('_') {
                return GameState::Won;
            }
            return GameState::WaitingForInput;
        } else {
            self.attempts_left -= 1;
            println!("Буквы '{}' нет в слове. Осталось попыток: {}", letter, self.attempts_left);

            if self.attempts_left <= 0 {
                return GameState::Lost;
            }
            return GameState::WaitingForInput;
        }
    }
}

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

    // Выбор языка
    let (word_list, language_msg, expected_alphabet) = if rng_seed % 2 == 0 {
        (&word_list_en, "АНГЛИЙСКОЕ", "EN")
    } else {
        (&word_list_ru, "РУССКОЕ", "RU")
    };

    let index_word = rng_seed % word_list.len();
    let secret_word = word_list[index_word].to_string();
    
    // Инициализация контекста
    let mut ctx = GameContext::new(&secret_word, expected_alphabet);
    let mut state = GameState::WaitingForInput;

    print_words_with_delay(&["Добро ", "пожаловать ", "в игру"], Duration::from_millis(500));
    thread::sleep(Duration::from_millis(1000));
    println!(" 'ВИСЕЛИЦА'!");
    thread::sleep(Duration::from_millis(800));
    type_like_human(&format!("Угадайте {} слово.", language_msg), Duration::from_millis(60));
    thread::sleep(Duration::from_millis(800));
    println!(" Удачи!");

    // ЦИКЛ КОНЕЧНОГО АВТОМАТА
    while state != GameState::Won && state != GameState::Lost {
        match state {
            GameState::WaitingForInput => {
                print_game_state(&ctx.current_state, &ctx.guessed_letters, ctx.attempts_left);

                // Чтение ввода
                let letter = read_letter_raw();
                match letter {
                    Some(l) => {
                        // Валидация языка перед переходом в CheckingGuess
                        if ctx.is_valid_letter(l) {
                            state = GameState::CheckingGuess(l);
                        } else {
                            let expected = if ctx.expected_alphabet == "EN" { "английские" } else { "русские" };
                            println!("Ожидаются только {} буквы. Попробуйте ещё раз.", expected);
                            // Остаемся в WaitingForInput
                            state = GameState::WaitingForInput;
                        }
                    }
                    None => {
                        println!("Пожалуйста, введите ровно одну букву.");
                        state = GameState::WaitingForInput;
                    }
                }
            }
            GameState::CheckingGuess(letter) => {
                // Делегируем логику проверки контексту
                state = ctx.process_guess(letter);
            }
            GameState::Won | GameState::Lost => {
                // Эти состояния обрабатываются условием while, сюда попадать не должны
                break;
            }
        }
    }

    // Финальный вывод
    let sw_translation = {
        if expected_alphabet == "EN" { word_list_ru } else { word_list_en }
    }[index_word].to_string();

    let title = match state {
        GameState::Won => "Поздравляем! Вы угадали слово:",
        GameState::Lost => "Вы проиграли! Правильное слово было:",
        _ => "Игра завершена:", // fallback
    };
    
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

// Обертка над stdin, возвращает Option<char>
fn read_letter_raw() -> Option<char> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Ошибка чтения ввода");
    _parse_letter(&input)
}

// Внутренняя логика парсинга (для тестов)
fn _parse_letter(input: &str) -> Option<char> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.chars().count() != 1 {
        return None;
    }

    match trimmed.chars().next() {
        Some(c) if c.is_alphabetic() => Some(c.to_lowercase().next().unwrap_or(c)),
        _ => None,
    }
}

fn reveal_letter(secret: &str, current: &str, letter: char) -> String {
    let secret_chars: Vec<char> = secret.chars().collect();
    let current_chars: Vec<char> = current.chars().collect();

    let mut result = String::with_capacity(secret.len());
    for (i, c) in secret_chars.iter().enumerate() {
        if *c == letter {
            result.push(*c);
        } else {
            result.push(current_chars[i]);
        }
    }
    result
}

use std::thread;
use std::time::Duration;

/// Печатает слова по отдельности с задержкой между словами
fn print_words_with_delay(words: &[&str], word_delay: Duration) {
    let stdout = io::stdout();
    let mut lock = stdout.lock();

    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            // Небольшая пауза между словами, чтобы выглядело естественно
            thread::sleep(Duration::from_millis(50));
        }
        write!(lock, "{}", word).unwrap();
        io::stdout().flush().unwrap();

        if i < words.len() - 1 {
            thread::sleep(word_delay);
        }
    }
}

/// Печатает строку посимвольно с задержкой (имитация набора)
fn type_like_human(text: &str, delay: Duration) {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    for ch in text.chars() {
        write!(lock, "{}", ch).unwrap();
        io::stdout().flush().unwrap();
        thread::sleep(delay);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn try_enter_letter(ctx: &GameContext, letter: char) -> GameState {
        if ctx.is_valid_letter(letter) {
            GameState::CheckingGuess(letter)
        } else {
            GameState::WaitingForInput
        }
    }

    fn resolve_check(ctx: &mut GameContext, state: GameState) -> GameState {
        match state {
            GameState::CheckingGuess(letter) => ctx.process_guess(letter),
            _ => state,
        }
    }

    #[test]
    fn test_fsm_win_flow() {
        let mut ctx = GameContext::new("cat", "EN");

        // Сразу получаем состояние после ввода, без лишней инициализации
        let mut state = try_enter_letter(&ctx, 'c');
        assert_eq!(state, GameState::CheckingGuess('c'));

        state = resolve_check(&mut ctx, state);
        assert_eq!(state, GameState::WaitingForInput);
        assert_eq!(ctx.current_state, "c__");

        state = try_enter_letter(&ctx, 'a');
        state = resolve_check(&mut ctx, state);
        assert_eq!(state, GameState::WaitingForInput);
        assert_eq!(ctx.current_state, "ca_");

        state = try_enter_letter(&ctx, 't');
        state = resolve_check(&mut ctx, state);
        assert_eq!(state, GameState::Won);
    }

    #[test]
    fn test_fsm_loss_flow() {
        let mut ctx = GameContext::new("dog", "EN");
        let wrong_letters = ['x', 'y', 'z', 'q', 'w', 'v'];

        for &letter in &wrong_letters {
            // Шаг 1: только валидация и формирование CheckingGuess (ctx нужен как &)
            let state = try_enter_letter(&ctx, letter);

            // Шаг 2: обработка попытки (ctx нужен как &mut)
            let state = resolve_check(&mut ctx, state);

            if letter == 'v' {
                assert_eq!(state, GameState::Lost);
                assert_eq!(ctx.attempts_left, 0);
            } else {
                assert_eq!(state, GameState::WaitingForInput);
            }
        }
    }

    #[test]
    fn test_duplicate_letter_handling() {
        let mut ctx = GameContext::new("cat", "EN");

        let mut state = try_enter_letter(&ctx, 'c');
        resolve_check(&mut ctx, state);  // return state
        assert_eq!(ctx.guessed_letters.len(), 1);
        assert_eq!(ctx.attempts_left, 6);

        state = try_enter_letter(&ctx, 'c');
        assert_eq!(state, GameState::CheckingGuess('c'));

        state = resolve_check(&mut ctx, state);
        assert_eq!(state, GameState::WaitingForInput);
        assert_eq!(ctx.guessed_letters.len(), 1);
        assert_eq!(ctx.attempts_left, 6);
    }

    #[test]
    fn test_invalid_alphabet_transition() {
        let ctx = GameContext::new("cat", "EN");
        let state = try_enter_letter(&ctx, '1');
        assert_eq!(state, GameState::WaitingForInput);
    }

    #[test]
    fn test_parse_valid_ascii() {
        assert_eq!(_parse_letter("a\n"), Some('a'));
        assert_eq!(_parse_letter("A\n"), Some('a'));
    }

    #[test]
    fn test_parse_valid_russian() {
        assert_eq!(_parse_letter("а\n"), Some('а'));
        assert_eq!(_parse_letter("А\n"), Some('а'));
    }

    #[test]
    fn test_parse_invalid_inputs() {
        assert_eq!(_parse_letter("\n"), None);
        assert_eq!(_parse_letter("ab\n"), None);
        assert_eq!(_parse_letter("1\n"), None);
        assert_eq!(_parse_letter("#\n"), None);
    }

    #[test]
    fn test_reveal_logic() {
        assert_eq!(reveal_letter("banana", "______", 'a'), "_a_a_a");
        assert_eq!(reveal_letter("dog", "___", 'x'), "___");
        assert_eq!(reveal_letter("cat", "_a_", 'c'), "ca_");
    }
}
