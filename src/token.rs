//! Command tokenizer.

use crate::{QuipError, QuipResult};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_plaintext() {
        let res = tokenize("A000 Login Hello").unwrap();
        let target: Vec<String> = ["A000", "Login", "Hello"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(res, target);
    }

    #[test]
    fn test_tokenizer_quote() {
        let res = tokenize("A000 Login \"Hello  How R U\"").unwrap();
        let target: Vec<String> = ["A000", "Login", "Hello  How R U"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(res, target);
    }

    #[test]
    fn test_tokenizer_escape() {
        let res = tokenize("A000 Login \\\" \\\"").unwrap();
        let target: Vec<String> = ["A000", "Login", "\"", "\""]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(res, target);
    }

    #[test]
    fn test_tokenizer_failed() {
        let res = tokenize("A000 Login \"Invalid");
        assert!(res.is_err());

        let res = tokenize("A000 Login Invalid\\");
        assert!(res.is_err());
    }
}
