use std::io::{self, Write};
use std::process::Command;

fn main() -> io::Result<()> {
    loop {
        print!("blazing_shell> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" {
            break;
        }

        // Разбиваем строку на команду и аргументы
        let mut parts = input.split_whitespace();
        let command = match parts.next() {
            Some(cmd) => cmd,
            None => continue, // на случай, если split вернул None (крайне маловероятно после trim, но вспомнил анекдот про программиста и два стакана)
        };
        let args: Vec<&str> = parts.collect();
        match execute_command(command, &args) {
            Ok(_) => {} // команда выполнена успешно
            Err(e) => eprintln!("{}", e),
        }
    }

    Ok(())
}


fn execute_command(command: &str, args: &[&str]) -> Result<(), String> {
    let output = match Command::new(command).args(args).output() {
        Ok(output) => output,
        Err(e) => {
            // Требование № 1: если команда не найдена
            return Err(format!("Ошибка запуска '{}': {}", command, e));
        }
    };

    // Выводим stdout, если есть данные
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    }

    // Выводим stderr, если есть ошибки
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
    }

    // Требование № 2: если процесс завершился с ненулевым статусом
    if !output.status.success() {
        return Err(format!(
            "Команда '{}' завершилась с кодом ошибки: {}",
            command,
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}
