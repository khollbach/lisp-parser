use std::{
    fmt::{self, Display, Formatter},
    mem,
};

use anyhow::{ensure, Result};

fn main() -> Result<()> {
    let s = "(first (list 1 (+ 2 3) 9))";
    let expr = parse(s)?;
    assert_eq!(expr.to_string(), s);
    Ok(())
}

#[derive(Debug)]
pub enum Expr {
    Group(Vec<Expr>),
    Atom(Atom),
}

#[derive(Debug)]
pub enum Atom {
    Ident(String),
    Num(u32),
}

pub fn parse(expr: &str) -> Result<Expr> {
    let tokens = tokenize(expr);
    parse_tokens(tokens)
}

enum Token {
    Lparen,
    Rparen,
    Atom(Atom),
}

fn tokenize(expr: &str) -> impl Iterator<Item = Token> + '_ {
    let mut curr_token = String::new();

    expr.chars().chain([' ']).flat_map(move |c| {
        let atom = if matches!(c, '(' | ')' | ' ') && !curr_token.is_empty() {
            let atom = Atom::new(mem::take(&mut curr_token));
            Some(Token::Atom(atom))
        } else {
            None
        };

        let paren = match c {
            '(' => Some(Token::Lparen),
            ')' => Some(Token::Rparen),
            ' ' => None,
            _ => {
                curr_token.push(c);
                None
            }
        };

        [atom, paren].into_iter().flatten()
    })
}

impl Atom {
    fn new(s: String) -> Self {
        match s.parse() {
            Ok(n) => Self::Num(n),
            Err(_) => Self::Ident(s),
        }
    }
}

fn parse_tokens(tokens: impl Iterator<Item = Token>) -> Result<Expr> {
    // We include a "dummy" outer-most context, to collect up any number of
    // top-level expressions.
    //
    // invariant: `contexts.len() >= 1`.
    let dummy_context = vec![];
    let mut contexts: Vec<Vec<Expr>> = vec![dummy_context];

    for t in tokens {
        match t {
            // Enter a new context.
            Token::Lparen => contexts.push(vec![]),
            // "Close" the current context.
            // It becomes a single `group`, appended to the context that contains it.
            Token::Rparen => {
                // You can't "close" the dummy context.
                ensure!(contexts.len() >= 2, "unexpected closing paren");

                let group = contexts.pop().unwrap();
                contexts.last_mut().unwrap().push(Expr::Group(group));
            }
            // Append an atom to the current context.
            Token::Atom(atom) => contexts.last_mut().unwrap().push(Expr::Atom(atom)),
        }
    }

    ensure!(
        contexts.len() == 1,
        "{} unclosed open paren(s)",
        contexts.len() - 1,
    );
    let mut dummy_context = contexts.pop().unwrap();

    ensure!(
        dummy_context.len() == 1,
        "expected 1 top-level expression, found {}",
        dummy_context.len(),
    );
    let expr = dummy_context.pop().unwrap();

    Ok(expr)
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Group(sub_exprs) => {
                write!(f, "(")?;
                for (i, expr) in sub_exprs.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }

                    write!(f, "{expr}")?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Expr::Atom(atom) => write!(f, "{atom}"),
        }
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Ident(name) => write!(f, "{name}"),
            Atom::Num(x) => write!(f, "{x}"),
        }
    }
}
