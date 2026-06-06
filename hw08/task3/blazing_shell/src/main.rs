use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    loop {
        print!("blazing_shell> ");
        io::stdout().flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {},
            Err(e) => {
                eprintln!("Ошибка чтения ввода: {}", e);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" {
            break;
        }

        if input.contains('|') {
            match handle_pipe(input) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e),
            }
        } else {
            let mut parts = input.split_whitespace();
            let command = match parts.next() {
                Some(cmd) => cmd,
                None => continue,
            };
            let args: Vec<&str> = parts.collect();

            match execute_command(command, &args) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    Ok(())
}

fn handle_pipe(input: &str) -> Result<(), String> {
    let parts: Vec<&str> = input.split('|').collect();
    if parts.len() != 2 {
        return Err("Поддерживается только один символ |".to_string());
    }

    let left_cmd = parts[0].trim();
    let right_cmd = parts[1].trim();

    // Исправленная обработка левой команды
    let mut left_parts = left_cmd.split_whitespace();
    let left_command = match left_parts.next() {
        Some(cmd) if !cmd.is_empty() => cmd,
        _ => return Err("Пустая левая команда".to_string()),
    };
    let left_args: Vec<&str> = left_parts.collect();

    // Исправленная обработка правой команды
    let mut right_parts = right_cmd.split_whitespace();
    let right_command = match right_parts.next() {
        Some(cmd) if !cmd.is_empty() => cmd,
        _ => return Err("Пустая правая команда".to_string()),
    };
    let right_args: Vec<&str> = right_parts.collect();

    let mut left_process = match Command::new(left_command)
        .args(&left_args)
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(process) => process,
        Err(e) => return Err(format!("Ошибка запуска '{}': {}", left_command, e)),
    };

    let left_stdout = match left_process.stdout.take() {
        Some(stdout) => stdout,
        None => return Err("Не удалось получить stdout левой команды".to_string()),
    };

    let output = match Command::new(right_command)
        .args(&right_args)
        .stdin(Stdio::from(left_stdout))
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(format!("Ошибка запуска '{}': {}", right_command, e)),
    };

    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    }

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
    }

    if !output.status.success() {
        return Err(format!(
            "Команда '{}' завершилась с кодом ошибки: {}",
            right_command,
            output.status.code().unwrap_or(-1)
        ));
    }

    match left_process.wait() {
        Ok(status) if !status.success() => {
            return Err(format!(
                "Левая команда '{}' завершилась с ошибкой: {}",
                left_command,
                status.code().unwrap_or(-1)
            ));
        }
        Err(e) => {
            return Err(format!("Ошибка ожидания левой команды: {}", e));
        }
        _ => {}
    }

    Ok(())
}

fn execute_command(command: &str, args: &[&str]) -> Result<(), String> {
    let output = match Command::new(command).args(args).output() {
        Ok(output) => output,
        Err(e) => {
            return Err(format!("Ошибка запуска '{}': {}", command, e));
        }
    };

    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    }

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
    }

    if !output.status.success() {
        return Err(format!(
            "Команда '{}' завершилась с кодом ошибки: {}",
            command,
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}
