use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// ==================== AST ====================

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Number(f64),
    Bool(bool),
    Symbol(String),
    List(Vec<Expr>),
}

// ==================== Values & Environment ====================

type Env = Rc<RefCell<Environment>>;
type BuiltinFn = fn(&[Value]) -> Result<Value, String>;

// УБРАЛИ PartialEq из derive! Оставили только Debug и Clone.
#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    Bool(bool),
    Function(Rc<Closure>),
    Builtin(BuiltinFn),
    Nil,
}

// Ручная реализация PartialEq для Value
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            // Для функций сравниваем указатели Rc: равны, только если это один и тот же экземпляр
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            // Для Builtin сравниваем указатели функций
            (Value::Builtin(a), Value::Builtin(b)) => std::ptr::eq(a, b),
            _ => false,
        }
    }
}

#[derive(Clone)]
struct Closure {
    params: Vec<String>,
    body: Vec<Expr>,
    env: Env,
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Closure(params={})", self.params.len())
    }
}

struct Environment {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

impl Environment {
    fn new() -> Env {
        Rc::new(RefCell::new(Environment {
            vars: HashMap::new(),
            parent: None,
        }))
    }

    fn extend(parent: Env) -> Env {
        Rc::new(RefCell::new(Environment {
            vars: HashMap::new(),
            parent: Some(parent),
        }))
    }

    fn lookup(env: &Env, name: &str) -> Option<Value> {
        let mut current = env.clone();
        loop {
            if let Some(val) = {
                let borrowed = current.borrow();
                borrowed.vars.get(name).cloned()
            } {
                return Some(val);
            }

            let parent = {
                let borrowed = current.borrow();
                borrowed.parent.clone()
            };

            match parent {
                Some(p) => current = p,
                None => break,
            }
        }
        None
    }

    fn define(env: &Env, name: String, value: Value) {
        env.borrow_mut().vars.insert(name, value);
    }

    fn assign(env: &Env, name: &str, value: Value) -> Result<(), String> {
        let mut current = env.clone();
        loop {
            {
                let mut borrowed = current.borrow_mut();
                if borrowed.vars.contains_key(name) {
                    borrowed.vars.insert(name.to_string(), value);
                    return Ok(());
                }
            }

            let parent = {
                let borrowed = current.borrow();
                borrowed.parent.clone()
            };

            match parent {
                Some(p) => current = p,
                None => break,
            }
        }
        Err(format!("unknown variable: {}", name))
    }
}

// ==================== Tokenizer ====================

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '(' || c == ')' {
            tokens.push(c.to_string());
            i += 1;
            continue;
        }
        // number, bool, symbol
        let start = i;
        if c == '-' || c.is_digit(10) || c == '.' {
            // number: allow optional leading '-' and digits/dot
            while i < chars.len() && (chars[i].is_digit(10) || chars[i] == '.' || (i == start && chars[i] == '-')) {
                i += 1;
            }
            // check if it's actually a number and not a symbol like "-x"
            let candidate = &input[start..i];
            if candidate.parse::<f64>().is_ok() {
                tokens.push(candidate.to_string());
                continue;
            } else {
                // fallback to symbol
                i = start;
            }
        }
        // symbol or bool
        while i < chars.len()
            && !chars[i].is_whitespace()
            && chars[i] != '('
            && chars[i] != ')'
        {
            i += 1;
        }
        tokens.push(input[start..i].to_string());
    }
    tokens
}

// ==================== Parser ====================

fn parse(tokens: &[String]) -> Result<Expr, String> {
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
    if token == "(" {
        return parse_list(tokens, pos);
    } else if token == ")" {
        return Err("unexpected ')'".to_string());
    } else if token == "true" {
        return Ok((Expr::Bool(true), pos + 1));
    } else if token == "false" {
        return Ok((Expr::Bool(false), pos + 1));
    } else {
        // number or symbol
        if let Ok(num) = token.parse::<f64>() {
            return Ok((Expr::Number(num), pos + 1));
        }
        return Ok((Expr::Symbol(token.clone()), pos + 1));
    }
}

fn parse_list(tokens: &[String], start: usize) -> Result<(Expr, usize), String> {
    // tokens[start] == "("
    let mut pos = start + 1;
    let mut items = Vec::new();
    while pos < tokens.len() && tokens[pos] != ")" {
        let (expr, new_pos) = parse_expr(tokens, pos)?;
        items.push(expr);
        pos = new_pos;
    }
    if pos >= tokens.len() || tokens[pos] != ")" {
        return Err("missing closing ')'".to_string());
    }
    Ok((Expr::List(items), pos + 1))
}

// ==================== Helpers for Values ====================

fn expect_number(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err("expected number".to_string()),
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(false) => false,
        Value::Nil => false,
        _ => true,
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            if *n == (*n).trunc() {
                (n.trunc() as i64).to_string()
            } else {
                n.to_string()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Function(_) => "<function>".to_string(),
        Value::Builtin(_) => "<builtin>".to_string(),
        Value::Nil => "nil".to_string(),
    }
}

// ==================== Builtins ====================

fn builtin_add(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("'+' expects at least 1 argument".to_string());
    }
    let sum = args.iter()
        .map(|v| expect_number(v))
        .sum::<Result<f64, String>>()?;
    Ok(Value::Number(sum))
}

fn builtin_sub(args: &[Value]) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("'-' expects at least 1 argument".to_string());
    }
    let first = expect_number(&args[0])?;
    if args.len() == 1 {
        Ok(Value::Number(-first))
    } else {
        let rest_sum = args[1..].iter()
            .map(|v| expect_number(v))
            .sum::<Result<f64, String>>()?;
        Ok(Value::Number(first - rest_sum))
    }
}

fn builtin_mul(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("'*' expects at least 1 argument".to_string());
    }
    let prod = args.iter()
        .map(|v| expect_number(v))
        .product::<Result<f64, String>>()?;
    Ok(Value::Number(prod))
}

fn builtin_div(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("'/' expects at least 2 arguments".to_string());
    }
    let first = expect_number(&args[0])?;
    let mut result = first;
    for v in &args[1..] {
        let d = expect_number(v)?;
        if d == 0.0 {
            return Err("division by zero".to_string());
        }
        result /= d;
    }
    Ok(Value::Number(result))
}

fn builtin_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'=' expects exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(*a == *b)),
        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a == *b)),
        _ => Ok(Value::Bool(false)),
    }
}

fn builtin_lt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'<' expects exactly 2 arguments".to_string());
    }
    let a = expect_number(&args[0])?;
    let b = expect_number(&args[1])?;
    Ok(Value::Bool(a < b))
}

fn builtin_gt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'>' expects exactly 2 arguments".to_string());
    }
    let a = expect_number(&args[0])?;
    let b = expect_number(&args[1])?;
    Ok(Value::Bool(a > b))
}

fn default_env() -> Env {
    let env = Environment::new();
    Environment::define(&env, "+".to_string(), Value::Builtin(builtin_add));
    Environment::define(&env, "-".to_string(), Value::Builtin(builtin_sub));
    Environment::define(&env, "*".to_string(), Value::Builtin(builtin_mul));
    Environment::define(&env, "/".to_string(), Value::Builtin(builtin_div));
    Environment::define(&env, "=".to_string(), Value::Builtin(builtin_eq));
    Environment::define(&env, "<".to_string(), Value::Builtin(builtin_lt));
    Environment::define(&env, ">".to_string(), Value::Builtin(builtin_gt));
    env
}

// ==================== Evaluator ====================

fn eval_sequence(body: &[Expr], env: Env) -> Result<Value, String> {
    let mut last = Value::Nil;
    for expr in body {
        last = eval(expr, env.clone())?;
    }
    Ok(last)
}

fn apply(func: Value, args: Vec<Value>) -> Result<Value, String> {
    match func {
        Value::Builtin(f) => f(&args),
        Value::Function(closure) => {
            if args.len() != closure.params.len() {
                return Err(format!(
                    "function expects {} arguments, got {}",
                    closure.params.len(),
                    args.len()
                ));
            }
            let call_env = Environment::extend(closure.env.clone());
            for (param, arg) in closure.params.iter().zip(args) {
                Environment::define(&call_env, param.clone(), arg);
            }
            eval_sequence(&closure.body, call_env)
        }
        _ => Err("not a function".to_string()),
    }
}

fn eval(expr: &Expr, env: Env) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Symbol(name) => {
            Environment::lookup(&env, name)
                .ok_or_else(|| format!("unknown variable: {}", name))
        }
        Expr::List(items) => {
            if items.is_empty() {
                return Ok(Value::Nil);
            }
            match &items[0] {
                Expr::Symbol(op) => match op.as_str() {
                    "define" => eval_define(&items[1..], env),
                    "set!" => eval_set(&items[1..], env),
                    "lambda" => eval_lambda(&items[1..], env),
                    "if" => eval_if(&items[1..], env),
                    "let" => eval_let(&items[1..], env),
                    _ => eval_call(items, env),
                },
                _ => eval_call(items, env),
            }
        }
    }
}

fn eval_define(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() != 2 {
        return Err("'define' expects exactly 2 arguments: (define name expr)".to_string());
    }
    let name = match &items[0] {
        Expr::Symbol(s) => s.clone(),
        _ => return Err("'define' name must be a symbol".to_string()),
    };
    let value = eval(&items[1], env.clone())?;
    Environment::define(&env, name, value);
    Ok(Value::Nil)
}

fn eval_set(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() != 2 {
        return Err("'set!' expects exactly 2 arguments: (set! name expr)".to_string());
    }
    let name = match &items[0] {
        Expr::Symbol(s) => s.as_str(),
        _ => return Err("'set!' name must be a symbol".to_string()),
    };
    let value = eval(&items[1], env.clone())?;
    Environment::assign(&env, name, value)?;
    Ok(Value::Nil)
}

fn eval_lambda(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() < 2 {
        return Err("'lambda' expects at least 2 arguments: (lambda (params...) body...)".to_string());
    }
    let params = match &items[0] {
        Expr::List(p) => p.iter()
            .filter_map(|e| match e {
                Expr::Symbol(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        _ => return Err("'lambda' parameters must be a list of symbols".to_string()),
    };
    let body = items[1..].to_vec();
    Ok(Value::Function(Rc::new(Closure {
        params,
        body,
        env: env.clone(), // сохраняем окружение определения — это и есть замыкание
    })))
}

fn eval_if(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() != 3 {
        return Err("'if' expects exactly 3 arguments: (if cond then else)".to_string());
    }
    let cond_val = eval(&items[0], env.clone())?;
    if is_truthy(&cond_val) {
        eval(&items[1], env)
    } else {
        eval(&items[2], env)
    }
}

fn eval_let(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.is_empty() {
        return Err("'let' expects at least a bindings list and body".to_string());
    }

    let bindings = match &items[0] {
        Expr::List(b) => b,
        _ => return Err("'let' bindings must be a list".to_string()),
    };

    // bindings: ((name1 expr1) (name2 expr2) ...)
    let local_env = Environment::extend(env.clone());

    for binding in bindings {
        let pair = match binding {
            Expr::List(pair) if pair.len() == 2 => pair,
            _ => return Err("each 'let' binding must be (name expr)".to_string()),
        };
        let name = match &pair[0] {
            Expr::Symbol(s) => s.clone(),
            _ => return Err("let binding name must be a symbol".to_string()),
        };
        let value = eval(&pair[1], local_env.clone())?;
        Environment::define(&local_env, name, value);
    }

    // тело let — это все остальные элементы
    let body = &items[1..];
    eval_sequence(body, local_env)
}

fn eval_call(items: &[Expr], env: Env) -> Result<Value, String> {
    let func_val = eval(&items[0], env.clone())?;
    let mut args = Vec::with_capacity(items.len() - 1);
    for arg in &items[1..] {
        args.push(eval(arg, env.clone())?);
    }
    apply(func_val, args)
}

fn run_line(line: &str, env: Env) -> Result<Value, String> {
    let tokens = tokenize(line);
    if tokens.is_empty() {
        return Ok(Value::Nil);
    }
    let ast = parse(&tokens)?;
    eval(&ast, env)
}

fn main() {
    let env = default_env();
    let stdin = std::io::stdin();

    greeting();  // а просто так
    print!(">>> "); io::stdout().flush().unwrap();
    thread::sleep(Duration::from_millis(500));
    for _ in 0..=3 {
        print!("{}", '\x08'); // \b - backspace
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(200));
    }
    print!("\r>  \n> ");
    io::stdout().flush().unwrap();

    for line in stdin.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed == "quit" || trimmed == "exit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        match run_line(trimmed, env.clone()) {
            Ok(val) => println!("{}", format_value(&val)),
            Err(e) => eprintln!("error: {}", e),
        }
        print!("> "); io::stdout().flush().unwrap();
    }
}

use std::io::{self, Write};
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
    println!(", HARD LISP!"); io::stdout().flush().unwrap();
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

    fn run_expr(expr: &str) -> Result<Value, String> {
        let tokens = tokenize(expr);
        let ast = parse(&tokens)?;
        eval(&ast, default_env())
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(run_expr("(+ 1 2)").unwrap(), Value::Number(3.0));
        assert_eq!(run_expr("(* (+ 2 3) 4)").unwrap(), Value::Number(20.0));
    }

    #[test]
    fn test_variables() {
        // определяем x, потом используем
        let env = default_env();
        run_line("(define x 10)", env.clone()).unwrap();
        let val = run_line("(+ x 5)", env).unwrap();
        assert_eq!(val, Value::Number(15.0));
    }

    #[test]
    fn test_lambda_and_call() {
        let env = default_env();
        run_line("(define add (lambda (a b) (+ a b)))", env.clone()).unwrap();
        let val = run_line("(add 2 3)", env).unwrap();
        assert_eq!(val, Value::Number(5.0));
    }

    #[test]
    fn test_lexical_scoping() {
        let env = default_env();
        run_line("(define x 10)", env.clone()).unwrap();
        run_line("(define f (lambda (y) (+ x y)))", env.clone()).unwrap();
        // в let переопределяем x=100, но f использует x из своего замыкания (10)
        let val = run_line("(let ((x 100)) (f 1))", env).unwrap();
        assert_eq!(val, Value::Number(11.0)); // 10 + 1
    }

    #[test]
    fn test_closures_make_adder() {
        let env = default_env();
        run_line("(define make-adder (lambda (x) (lambda (y) (+ x y))))", env.clone()).unwrap();
        run_line("(define add10 (make-adder 10))", env.clone()).unwrap();
        run_line("(define add20 (make-adder 20))", env.clone()).unwrap();

        let v1 = run_line("(add10 5)", env.clone()).unwrap();
        let v2 = run_line("(add20 5)", env).unwrap();
        assert_eq!(v1, Value::Number(15.0));
        assert_eq!(v2, Value::Number(25.0));
    }

    #[test]
    fn test_if() {
        assert_eq!(run_expr("(if true 10 20)").unwrap(), Value::Number(10.0));
        assert_eq!(run_expr("(if false 10 20)").unwrap(), Value::Number(20.0));
    }

    #[test]
    fn test_let() {
        let env = default_env();
        run_line("(define x 10)", env.clone()).unwrap();
        let val = run_line("(let ((x 100) (y 5)) (+ x y))", env.clone()).unwrap();
        assert_eq!(val, Value::Number(105.0));
        // внешняя x осталась 10
        let x_val = run_line("x", env).unwrap();
        assert_eq!(x_val, Value::Number(10.0));
    }

    #[test]
    fn test_set_and_counter() {
        let env = default_env();
        run_line("(define make-counter (lambda () (let ((count 0)) (lambda () (set! count (+ count 1)) count))))", env.clone()).unwrap();
        run_line("(define c1 (make-counter))", env.clone()).unwrap();
        run_line("(define c2 (make-counter))", env.clone()).unwrap();

        assert_eq!(run_line("(c1)", env.clone()).unwrap(), Value::Number(1.0));
        assert_eq!(run_line("(c1)", env.clone()).unwrap(), Value::Number(2.0));
        assert_eq!(run_line("(c2)", env.clone()).unwrap(), Value::Number(1.0));
        assert_eq!(run_line("(c1)", env).unwrap(), Value::Number(3.0));
    }

    #[test]
    fn test_factorial_recursive() {
        let env = default_env();
        run_line(r#"
(define fact
  (lambda (n)
    (if (= n 0)
        1
        (* n (fact (- n 1))))))
"#, env.clone()).unwrap();
        let val = run_line("(fact 5)", env).unwrap();
        assert_eq!(val, Value::Number(120.0));
    }

    #[test]
    fn test_errors() {
        // неизвестная переменная
        assert!(run_expr("x").is_err());
        // деление на ноль
        assert!(run_expr("(/ 1 0)").is_err());
        // set! несуществующей переменной
        let env = default_env();
        run_line("(define y 10)", env.clone()).unwrap();
        assert!(run_line("(set! z 5)", env).is_err()); // z не определена
    }
}
