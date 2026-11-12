use std::io;
use std::iter::Peekable;

fn main() {
    println!("Advanced Calculator REPL. Type 'end' to exit.");

    loop {
        // Read input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        // Check for exit command
        if input.eq_ignore_ascii_case("end") {
            break;
        }

        // Skip empty lines
        if input.is_empty() {
            continue;
        }

        // Evaluate and print result
        match evaluate(input) {
            Ok(result) => println!("= {}", result),
            Err(e) => println!("Error: {}", e),
        }
    }
}

/// Sets up the character iterator, ignoring all whitespace
fn evaluate(expr: &str) -> Result<f64, String> {
    let mut chars = expr.chars().filter(|c| !c.is_whitespace()).peekable();
    let result = parse_expr(&mut chars)?;

    // If there are still characters left, the expression was invalid (e.g., "1 + 2 3")
    if chars.peek().is_some() {
        return Err("Unexpected characters at the end of the expression".to_string());
    }

    Ok(result)
}

/// Parses addition and subtraction
fn parse_expr<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> Result<f64, String> {
    let mut left = parse_term(chars)?;

    while let Some(&op) = chars.peek() {
        if op == '+' || op == '-' {
            chars.next(); // Consume the operator
            let right = parse_term(chars)?;
            left = if op == '+' { left + right } else { left - right };
        } else {
            break;
        }
    }
    Ok(left)
}

/// Parses multiplication and division
fn parse_term<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> Result<f64, String> {
    let mut left = parse_factor(chars)?;

    while let Some(&op) = chars.peek() {
        if op == '*' || op == '/' {
            chars.next(); // Consume the operator
            let right = parse_factor(chars)?;
            left = if op == '*' { left * right } else { left / right };
        } else {
            break;
        }
    }
    Ok(left)
}

/// Parses numbers, parentheses, and unary signs
fn parse_factor<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> Result<f64, String> {
    if let Some(&c) = chars.peek() {
        // Handle Parentheses
        if c == '(' {
            chars.next(); // Consume '('
            let val = parse_expr(chars)?;
            if chars.next() != Some(')') {
                return Err("Missing closing parenthesis ')'".to_string());
            }
            return Ok(val);
        }
        // Handle unary positive/negative signs (e.g., -5)
        else if c == '+' || c == '-' {
            let sign = chars.next().unwrap();
            let val = parse_factor(chars)?;
            return Ok(if sign == '-' { -val } else { val });
        }
        // Handle numbers (including decimals)
        else if c.is_digit(10) || c == '.' {
            let mut num_str = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_digit(10) || ch == '.' {
                    num_str.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            return num_str.parse::<f64>().map_err(|_| "Invalid number format".to_string());
        } else {
            return Err(format!("Unexpected character: {}", c));
        }
    }
    Err("Unexpected end of expression".to_string())
}