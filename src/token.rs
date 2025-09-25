//! Command tokenizer.

use crate::{QuipError, QuipResult};

#[macro_export]
macro_rules! unwrap_token {
    ($iter:expr, $msg:expr) => {
        match $iter.next() {
            Some(value) => value,
            None => return Err(QuipError::Parse($msg.into())),
        }
    };
}

/// Simple tokenizer with quote.
pub fn tokenize(input: impl AsRef<str>) -> QuipResult<Vec<String>> {
    let input = input.as_ref();

    let mut in_quote = false;
    let mut in_escape = false;

    let mut res = Vec::new();
    let mut curr = String::new();

    let mut chars = input.trim().chars();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' if !in_escape => {
                in_escape = true;
            }
            '"' if !in_escape => {
                in_quote = !in_quote;
            }
            ' ' if !in_quote && !in_escape => {
                if !curr.is_empty() {
                    res.push(curr.clone());
                    curr.clear();
                }
            }
            _ => {
                curr.push(ch);
                in_escape = false;
            }
        }
    }
    if in_quote || in_escape {
        return Err(QuipError::Parse(format!(
            "Unexpected EOL when parsing {{{}}}",
            input
        )));
    }

    if !curr.is_empty() {
        res.push(curr);
    }

    Ok(res)
}

/// Undo the tokenize result.
pub fn detokenize(input: &Vec<impl AsRef<str>>) -> String {
    let mut res = Vec::new();
    for item in input {
        let item = item.as_ref();
        let mut curr = String::new();

        if item.contains(' ') {
            curr.push('\"');
            curr += &escape_token(&item);
            curr.push('\"');
        } else {
            curr = escape_token(&item);
        }

        res.push(curr);
    }

    let res = res.join(" ");
    res
}

fn escape_token(input: &str) -> String {
    input.chars().fold(String::new(), |mut s, ch| {
        match ch {
            '\"' => s += "\\\"",
            '\\' => s += "\\\\",
            _ => s.push(ch),
        }
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_plaintext() {
        let res = tokenize("A000 Login Hello").unwrap();
        let target: Vec<String> = ["A000", "Login", "Hello"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(res, target);
    }

    #[test]
    fn test_tokenize_quote() {
        let res = tokenize("A000 Login \"Hello  How R U\"").unwrap();
        let target: Vec<String> = ["A000", "Login", "Hello  How R U"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(res, target);
    }

    #[test]
    fn test_tokenize_escape() {
        let res = tokenize("A000 Login \\\" \\\"").unwrap();
        let target: Vec<String> = ["A000", "Login", "\"", "\""]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(res, target);
    }

    #[test]
    fn test_tokenize_failed() {
        let res = tokenize("A000 Login \"Invalid");
        assert!(res.is_err());

        let res = tokenize("A000 Login Invalid\\");
        assert!(res.is_err());
    }

    #[test]
    fn test_detokenize_plaintext() {
        let res = detokenize(&vec!["A000", "Login", "Hello"]);
        let target = "A000 Login Hello";

        assert_eq!(res, target);
    }

    #[test]
    fn test_detokenize_quote() {
        let res = detokenize(&vec!["A000", "Login", "Hello  How R U"]);
        let target = "A000 Login \"Hello  How R U\"";

        assert_eq!(res, target);
    }

    #[test]
    fn test_detokenize_escape() {
        let res = detokenize(&vec!["A000", "Login", "\"", "\""]);
        let target = "A000 Login \\\" \\\"";

        assert_eq!(res, target);
    }
}
