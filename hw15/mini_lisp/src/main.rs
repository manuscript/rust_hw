use std::collections::HashMap;
use std::io::{self, Write};

type Env = HashMap<String, f64>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Symbol(String),
    List(Vec<Expr>),
}

// -------------------------
// Tokenizer
// -------------------------
pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '(' | ')' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(ch.to_string());
            }
            ' ' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

// -------------------------
// Parser
// -------------------------
pub fn parse(tokens: &[String]) -> Result<Expr, String> {
    if tokens.is_empty() {
        return Err("empty input".to_string());
    }
    let (expr, pos) = parse_expr(tokens, 0)?;
    if pos != tokens.len() {
        return Err(format!("unexpected tokens after expression: {:?}", &tokens[pos..]));
    }
    Ok(expr)
}

fn parse_expr(tokens: &[String], pos: usize) -> Result<(Expr, usize), String> {
    if pos >= tokens.len() {
        return Err("unexpected end of input".to_string());
    }

    let token = &tokens[pos];

    match token.as_str() {
        "(" => parse_list(tokens, pos),
        ")" => Err("unexpected ')'".to_string()),
        _ => {
            // число или символ
            if let Ok(num) = token.parse::<f64>() {
                Ok((Expr::Number(num), pos + 1))
            } else {
                // символ (переменная или имя операции)
                Ok((Expr::Symbol(token.clone()), pos + 1))
            }
        }
    }
}

fn parse_list(tokens: &[String], start: usize) -> Result<(Expr, usize), String> {
    // tokens[start] == "("
    if start >= tokens.len() || tokens[start] != "(" {
        return Err("expected '('".to_string());
    }

    let mut items = Vec::new();
    let mut pos = start + 1;

    while pos < tokens.len() && tokens[pos] != ")" {
        let (expr, next_pos) = parse_expr(tokens, pos)?;
        items.push(expr);
        pos = next_pos;
    }

    if pos >= tokens.len() {
        return Err("missing ')'".to_string());
    }

    // skip ')'
    pos += 1;
    Ok((Expr::List(items), pos))
}

// -------------------------
// Evaluator
// -------------------------
pub fn eval(expr: &Expr, env: &mut Env) -> Result<f64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Symbol(name) => {
            env.get(name)
                .copied()
                .ok_or_else(|| format!("unknown variable: {}", name))
        }
        Expr::List(items) => {
            if items.is_empty() {
                return Err("empty list expression".to_string());
            }

            let op = match &items[0] {
                Expr::Symbol(s) => s.clone(),
                _ => return Err("first element of list must be an operator symbol".to_string()),
            };

            eval_list(&items[1..], &op, env)
        }
    }
}

fn eval_list(args: &[Expr], op: &str, env: &mut Env) -> Result<f64, String> {
    match op {
        "define" => eval_define(args, env),
        "+" => eval_sum(args, env),
        "-" => eval_sub(args, env),
        "*" => eval_product(args, env),
        "/" => eval_division(args, env),
        // доп. задание: сравнения
        "=" => eval_eq(args, env),
        "<" => eval_lt(args, env),
        ">" => eval_gt(args, env),
        _ => Err(format!("unknown operator: {}", op)),
    }
}

fn eval_define(args: &[Expr], env: &mut Env) -> Result<f64, String> {
    if args.len() != 2 {
        return Err("'define' expects exactly 2 arguments: (define name value)".to_string());
    }

    let name = match &args[0] {
        Expr::Symbol(s) => s.clone(),
        _ => return Err("'define' first argument must be a symbol".to_string()),
    };

    let value = eval(&args[1], env)?;
    env.insert(name.clone(), value);
    Ok(value)
}

fn eval_binary_check(args: &[Expr], expected: usize, msg: &str) -> Result<(), String> {
    if args.len() != expected {
        return Err(msg.to_string());
    }
    Ok(())
}

fn eval_sum(args: &[Expr], env: &mut Env) -> Result<f64, String> {
    let mut sum = 0.0;
    for arg in args {
        sum += eval(arg, env)?;
    }
    Ok(sum)
}

fn eval_sub(args: &[Expr], env: &mut Env) -> Result<f64, String> {
    eval_binary_check(args, 2, "'-' expects exactly 2 arguments")?;
    let a = eval(&args[0], env)?;
    let b = eval(&args[1], env)?;
    Ok(a - b)
}

fn eval_product(args: &[Expr], env: &mut Env) -> Result<f64, String> {
    let mut prod = 1.0;
    for arg in args {
        prod *= eval(arg, env)?;
    }
    Ok(prod)
}

fn eval_division(args: &[Expr], env: &mut Env) -> Result<f64, String> {
    eval_binary_check(args, 2, "'/' expects exactly 2 arguments")?;
    let numerator = eval(&args[0], env)?;
    let denominator = eval(&args[1], env)?;
    if denominator == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(numerator / denominator)
}

// доп. задание: сравнения (1.0 = true, 0.0 = false)
fn eval_eq(args: &[Expr], env: &mut Env) -> Result<f64, String> {
    eval_binary_check(args, 2, "'=' expects exactly 2 arguments")?;
    let a = eval(&args[0], env)?;
    let b = eval(&args[1], env)?;
    Ok(if a == b { 1.0 } else { 0.0 })
}

fn eval_lt(args: &[Expr], env: &mut Env) -> Result<f64, String> {
    eval_binary_check(args, 2, "'<' expects exactly 2 arguments")?;
    let a = eval(&args[0], env)?;
    let b = eval(&args[1], env)?;
    Ok(if a < b { 1.0 } else { 0.0 })
}

fn eval_gt(args: &[Expr], env: &mut Env) -> Result<f64, String> {
    eval_binary_check(args, 2, "'>' expects exactly 2 arguments")?;
    let a = eval(&args[0], env)?;
    let b = eval(&args[1], env)?;
    Ok(if a > b { 1.0 } else { 0.0 })
}

pub fn run_line(line: &str, env: &mut Env) -> Result<f64, String> {
    let tokens = tokenize(line);
    let ast = parse(&tokens)?;
    eval(&ast, env)
}

// -------------------------
// Main
// -------------------------
fn main() {
    let mut env = Env::new();

    greeting();
    print!("> "); io::stdout().flush().unwrap();

    let mut line = String::new();
    while io::stdin().read_line(&mut line).is_ok() {
        let trimmed = line.trim();
        if trimmed == "quit" || trimmed == "exit" {
            break;
        }
        if trimmed.is_empty() {
            line.clear();
            print!("> "); io::stdout().flush().unwrap();
            continue;
        }

        match run_line(trimmed, &mut env) {
            Ok(val) => {
                // печатаем как целое, если это целое значение
                if val.fract() == 0.0 {
                    println!("{}", val as i64);
                } else {
                    println!("{}", val);
                }
            }
            Err(e) => println!("error: {}", e),
        }

        line.clear();
        print!("> "); io::stdout().flush().unwrap();
    }
}

use std::thread;
use std::time::Duration;

fn greeting() {
    print_words_with_delay(&[">>> ", "This ", "is "], Duration::from_millis(500));
    thread::sleep(Duration::from_millis(800));
    print!("PYTHON!"); io::stdout().flush().unwrap();
    thread::sleep(Duration::from_millis(1000));
    print!("  Oops"); io::stdout().flush().unwrap();
    thread::sleep(Duration::from_millis(800));
    type_like_human(", no, something's wrong.", Duration::from_millis(60));
    println!(); io::stdout().flush().unwrap();

    thread::sleep(Duration::from_millis(800));
    print!(">>> "); io::stdout().flush().unwrap();
    type_like_human("This is Lisp", Duration::from_millis(70));
    thread::sleep(Duration::from_millis(800));
    //type_like_human("...", Duration::from_millis(100));
    println!(", MINI LISP!"); io::stdout().flush().unwrap();
}

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

    #[test]
    fn test_tokenize_basic() {
        assert_eq!(
            tokenize("(+ 1 2)"),
            vec!["(", "+", "1", "2", ")"]
        );
        assert_eq!(
            tokenize("(* (+ 2 3) 4)"),
            vec!["(", "*", "(", "+", "2", "3", ")", "4", ")"]
        );
    }

    #[test]
    fn test_parse_number() {
        let tokens = tokenize("42");
        let expr = parse(&tokens).unwrap();
        assert_eq!(expr, Expr::Number(42.0));
    }

    #[test]
    fn test_parse_symbol() {
        let tokens = tokenize("x");
        let expr = parse(&tokens).unwrap();
        assert_eq!(expr, Expr::Symbol("x".to_string()));
    }

    #[test]
    fn test_parse_list() {
        let tokens = tokenize("(+ 1 2)");
        let expr = parse(&tokens).unwrap();
        assert_eq!(
            expr,
            Expr::List(vec![Expr::Symbol("+".to_string()), Expr::Number(1.0), Expr::Number(2.0)])
        );
    }

    #[test]
    fn test_eval_number() {
        let mut env = Env::new();
        let result = eval(&Expr::Number(10.0), &mut env).unwrap();
        assert!((result - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_eval_variable_unknown() {
        let mut env = Env::new();
        let result = eval(&Expr::Symbol("unknown".to_string()), &mut env);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown variable"));
    }

    #[test]
    fn test_eval_add() {
        let mut env = Env::new();
        let ast = parse(&tokenize("(+ 1 2 3)")).unwrap();
        let result = eval(&ast, &mut env).unwrap();
        assert!((result - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_eval_nested() {
        let mut env = Env::new();
        let ast = parse(&tokenize("(* (+ 2 3) 4)")).unwrap();
        let result = eval(&ast, &mut env).unwrap();
        assert!((result - 20.0).abs() < 1e-9);
    }

    #[test]
    fn test_define_and_use() {
        let mut env = Env::new();
        run_line("(define x 10)", &mut env).unwrap();
        let val = run_line("(+ x 5)", &mut env).unwrap();
        assert!((val - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_division() {
        let mut env = Env::new();
        let ast = parse(&tokenize("(/ 20 4)")).unwrap();
        let result = eval(&ast, &mut env).unwrap();
        assert!((result - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_division_by_zero() {
        let mut env = Env::new();
        let ast = parse(&tokenize("(/ 10 0)")).unwrap();
        let result = eval(&ast, &mut env);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("division by zero"));
    }

    #[test]
    fn test_missing_paren() {
        let tokens = tokenize("(+ 1 2");
        let result = parse(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_comparison_eq() {
        let mut env = Env::new();
        let ast = parse(&tokenize("(= 2 2)")).unwrap();
        let result = eval(&ast, &mut env).unwrap();
        assert!((result - 1.0).abs() < 1e-9); // true

        let ast2 = parse(&tokenize("(= 2 3)")).unwrap();
        let result2 = eval(&ast2, &mut env).unwrap();
        assert!((result2 - 0.0).abs() < 1e-9); // false
    }

    #[test]
    fn test_comparison_lt() {
        let mut env = Env::new();
        let ast = parse(&tokenize("(< 1 2)")).unwrap();
        let result = eval(&ast, &mut env).unwrap();
        assert!((result - 1.0).abs() < 1e-9);

        let ast2 = parse(&tokenize("(< 3 2)")).unwrap();
        let result2 = eval(&ast2, &mut env).unwrap();
        assert!((result2 - 0.0).abs() < 1e-9);
    }
}

