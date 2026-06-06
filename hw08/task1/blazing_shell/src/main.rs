use std::io::{self, Write};
use std::process::Command;

fn main() -> io::Result<()> {
    loop {
        // 1. Печатаем prompt
        print!("blazing_shell> ");
        io::stdout().flush()?;

        // 2. Читаем строку из stdin
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Убираем символ новой строки
        let input = input.trim();

        // 3. Проверяем команду exit
        if input == "exit" {
            break;
        }

        // Пропускаем пустые строки
        if input.is_empty() {
            continue;
        }

        // 4. Разбиваем строку на команду и аргументы
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();

        // 5. Запускаем команду и ждём завершения
        execute_command(command, &args)?;
    }

    Ok(())
}

// 6. Функция для запуска команды
fn execute_command(command: &str, args: &[&str]) -> io::Result<()> {
    let output = Command::new(command)
        .args(args)
        .output()?;

    // Выводим stdout
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    }

    // Выводим stderr, если есть ошибки
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
    }

    // Проверяем код возврата
    if !output.status.success() {
        eprintln!("Command failed with exit code: {}", output.status);
    }

    Ok(())
}
