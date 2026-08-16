use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathExpr {
    Number(String),
    Name(String),
    Neg(Box<MathExpr>),
    Add(Vec<MathExpr>),
    Mul(Vec<MathExpr>),
    Div {
        numerator: Box<MathExpr>,
        denominator: Box<MathExpr>,
    },
    Pow {
        base: Box<MathExpr>,
        exponent: i32,
    },
    Call {
        function: String,
        args: Vec<MathExpr>,
    },
    Derivative {
        expr: Box<MathExpr>,
        with_respect_to: String,
        order: u8,
    },
    Gradient(Box<MathExpr>),
    Divergence(Box<MathExpr>),
    Curl(Box<MathExpr>),
    Inner {
        left: Box<MathExpr>,
        right: Box<MathExpr>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MathEquation {
    pub lhs: MathExpr,
    pub rhs: MathExpr,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum LatexError {
    #[error("unexpected token near byte {offset}: {message}")]
    Unexpected { offset: usize, message: String },
    #[error("unclosed LaTeX group")]
    UnclosedGroup,
    #[error("equation must contain exactly one top-level `=`")]
    ExpectedEquation,
    #[error("unsupported LaTeX command `\\{0}`")]
    UnsupportedCommand(String),
}

#[derive(Clone, Debug, PartialEq)]
enum TokKind {
    Name(String),
    Number(String),
    Command(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Eq,
    L,
    R,
    Underscore,
    Comma,
}
#[derive(Clone, Debug, PartialEq)]
struct Tok {
    kind: TokKind,
    offset: usize,
}

pub fn parse_equation(source: &str) -> Result<MathEquation, LatexError> {
    let tokens = lex(source)?;
    let mut depth = 0usize;
    let mut eq = None;
    for (i, token) in tokens.iter().enumerate() {
        match token.kind {
            TokKind::L => depth += 1,
            TokKind::R => depth = depth.saturating_sub(1),
            TokKind::Eq if depth == 0 => {
                if eq.replace(i).is_some() {
                    return Err(LatexError::ExpectedEquation);
                }
            }
            _ => {}
        }
    }
    let Some(index) = eq else {
        return Err(LatexError::ExpectedEquation);
    };
    Ok(MathEquation {
        lhs: parse_tokens(&tokens[..index])?,
        rhs: parse_tokens(&tokens[index + 1..])?,
    })
}

pub fn parse_expr(source: &str) -> Result<MathExpr, LatexError> {
    parse_tokens(&lex(source)?)
}

fn lex(source: &str) -> Result<Vec<Tok>, LatexError> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = source[i..].chars().next().unwrap();
        let width = c.len_utf8();
        match c {
            c if c.is_whitespace() => i += width,
            '\\' => {
                let start = i;
                i += 1;
                let name_start = i;
                while i < bytes.len() {
                    let ch = source[i..].chars().next().unwrap();
                    if !ch.is_alphabetic() {
                        break;
                    }
                    i += ch.len_utf8();
                }
                if i == name_start {
                    return Err(LatexError::Unexpected {
                        offset: start,
                        message: "expected command name after backslash".into(),
                    });
                }
                let name = source[name_start..i].to_string();
                if matches!(name.as_str(), "left" | "right" | "mathrm") {
                    continue;
                }
                if matches!(name.as_str(), "cdot" | "times") {
                    out.push(Tok {
                        kind: TokKind::Star,
                        offset: start,
                    });
                } else {
                    out.push(Tok {
                        kind: TokKind::Command(name),
                        offset: start,
                    });
                }
            }
            '+' => {
                out.push(Tok {
                    kind: TokKind::Plus,
                    offset: i,
                });
                i += 1;
            }
            '-' | '−' => {
                out.push(Tok {
                    kind: TokKind::Minus,
                    offset: i,
                });
                i += width;
            }
            '*' | '·' => {
                out.push(Tok {
                    kind: TokKind::Star,
                    offset: i,
                });
                i += width;
            }
            '/' => {
                out.push(Tok {
                    kind: TokKind::Slash,
                    offset: i,
                });
                i += 1;
            }
            '^' => {
                out.push(Tok {
                    kind: TokKind::Caret,
                    offset: i,
                });
                i += 1;
            }
            '=' => {
                out.push(Tok {
                    kind: TokKind::Eq,
                    offset: i,
                });
                i += 1;
            }
            '{' | '(' | '[' => {
                out.push(Tok {
                    kind: TokKind::L,
                    offset: i,
                });
                i += 1;
            }
            '}' | ')' | ']' => {
                out.push(Tok {
                    kind: TokKind::R,
                    offset: i,
                });
                i += 1;
            }
            '_' => {
                out.push(Tok {
                    kind: TokKind::Underscore,
                    offset: i,
                });
                i += 1;
            }
            ',' => {
                out.push(Tok {
                    kind: TokKind::Comma,
                    offset: i,
                });
                i += 1;
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                i += width;
                while i < bytes.len() {
                    let ch = source[i..].chars().next().unwrap();
                    if !(ch.is_ascii_digit() || matches!(ch, '.' | 'e' | 'E' | '+' | '-')) {
                        break;
                    }
                    i += ch.len_utf8();
                }
                out.push(Tok {
                    kind: TokKind::Number(source[start..i].to_string()),
                    offset: start,
                });
            }
            c if c.is_alphabetic() => {
                let start = i;
                i += width;
                while i < bytes.len() {
                    let ch = source[i..].chars().next().unwrap();
                    if !(ch.is_alphanumeric() || ch == '\'') {
                        break;
                    }
                    i += ch.len_utf8();
                }
                out.push(Tok {
                    kind: TokKind::Name(source[start..i].to_string()),
                    offset: start,
                });
            }
            _ => {
                return Err(LatexError::Unexpected {
                    offset: i,
                    message: format!("unsupported character `{c}`"),
                });
            }
        }
    }
    Ok(out)
}

fn parse_tokens(tokens: &[Tok]) -> Result<MathExpr, LatexError> {
    let mut p = Parser { tokens, pos: 0 };
    let expr = p.add()?;
    if p.pos != tokens.len() {
        let token = &tokens[p.pos];
        return Err(LatexError::Unexpected {
            offset: token.offset,
            message: "trailing input".into(),
        });
    }
    Ok(expr)
}

struct Parser<'a> {
    tokens: &'a [Tok],
    pos: usize,
}

impl Parser<'_> {
    fn add(&mut self) -> Result<MathExpr, LatexError> {
        let mut terms = vec![self.mul()?];
        loop {
            match self.tokens.get(self.pos).map(|t| &t.kind) {
                Some(TokKind::Plus) => {
                    self.pos += 1;
                    terms.push(self.mul()?);
                }
                Some(TokKind::Minus) => {
                    self.pos += 1;
                    terms.push(MathExpr::Neg(Box::new(self.mul()?)));
                }
                _ => break,
            }
        }
        Ok(if terms.len() == 1 {
            terms.pop().unwrap()
        } else {
            MathExpr::Add(terms)
        })
    }

    fn mul(&mut self) -> Result<MathExpr, LatexError> {
        let mut factors = vec![self.pow()?];
        loop {
            match self.tokens.get(self.pos).map(|t| &t.kind) {
                Some(TokKind::Star) => {
                    self.pos += 1;
                    factors.push(self.pow()?);
                }
                Some(TokKind::Slash) => {
                    self.pos += 1;
                    let left = if factors.len() == 1 {
                        factors.pop().unwrap()
                    } else {
                        MathExpr::Mul(std::mem::take(&mut factors))
                    };
                    let right = self.pow()?;
                    factors.push(MathExpr::Div {
                        numerator: Box::new(left),
                        denominator: Box::new(right),
                    });
                }
                Some(kind) if starts_primary(kind) => factors.push(self.pow()?),
                _ => break,
            }
        }
        Ok(if factors.len() == 1 {
            factors.pop().unwrap()
        } else {
            MathExpr::Mul(factors)
        })
    }

    fn pow(&mut self) -> Result<MathExpr, LatexError> {
        let mut value = self.primary()?;
        if matches!(
            self.tokens.get(self.pos).map(|t| &t.kind),
            Some(TokKind::Caret)
        ) {
            self.pos += 1;
            let exp_tokens = self.group_or_one()?;
            let text = exp_tokens
                .iter()
                .map(|t| match &t.kind {
                    TokKind::Number(n) | TokKind::Name(n) => n.clone(),
                    TokKind::Minus => "-".into(),
                    _ => String::new(),
                })
                .collect::<String>();
            let exponent = text.parse().map_err(|_| LatexError::Unexpected {
                offset: exp_tokens.first().map_or(0, |t| t.offset),
                message: "integer powers only in the constrained grammar".into(),
            })?;
            value = MathExpr::Pow {
                base: Box::new(value),
                exponent,
            };
        }
        Ok(value)
    }

    fn primary(&mut self) -> Result<MathExpr, LatexError> {
        let Some(token) = self.tokens.get(self.pos).cloned() else {
            return Err(LatexError::Unexpected {
                offset: 0,
                message: "expected expression".into(),
            });
        };
        self.pos += 1;
        let mut value = match token.kind {
            TokKind::Number(n) => MathExpr::Number(n),
            TokKind::Name(name) => MathExpr::Name(name),
            TokKind::Minus => MathExpr::Neg(Box::new(self.primary()?)),
            TokKind::L => {
                let start = self.pos;
                let end = matching_end(self.tokens, start)?;
                let expr = parse_tokens(&self.tokens[start..end])?;
                self.pos = end + 1;
                expr
            }
            TokKind::Command(command) => self.command(command, token.offset)?,
            _ => {
                return Err(LatexError::Unexpected {
                    offset: token.offset,
                    message: "expected expression".into(),
                });
            }
        };
        if matches!(
            self.tokens.get(self.pos).map(|t| &t.kind),
            Some(TokKind::Underscore)
        ) {
            self.pos += 1;
            let sub = self.group_or_one()?;
            let suffix = sub.iter().map(token_text).collect::<String>();
            if let MathExpr::Name(name) = &mut value {
                name.push('_');
                name.push_str(&suffix);
            }
        }
        Ok(value)
    }

    fn command(&mut self, command: String, offset: usize) -> Result<MathExpr, LatexError> {
        match command.as_str() {
            "frac" => {
                let numerator = self.group_or_one()?;
                let denominator = self.group_or_one()?;
                if let Some((expr, wrt, order)) = derivative_fraction(&numerator, &denominator)? {
                    Ok(MathExpr::Derivative {
                        expr: Box::new(expr),
                        with_respect_to: wrt,
                        order,
                    })
                } else {
                    Ok(MathExpr::Div {
                        numerator: Box::new(parse_tokens(&numerator)?),
                        denominator: Box::new(parse_tokens(&denominator)?),
                    })
                }
            }
            "nabla" => {
                if let Some(Tok {
                    kind: TokKind::Star,
                    ..
                }) = self.tokens.get(self.pos)
                {
                    self.pos += 1;
                    Ok(MathExpr::Divergence(Box::new(self.primary()?)))
                } else if let Some(Tok {
                    kind: TokKind::Command(c),
                    ..
                }) = self.tokens.get(self.pos)
                {
                    if c == "times" {
                        self.pos += 1;
                        return Ok(MathExpr::Curl(Box::new(self.primary()?)));
                    }
                    Ok(MathExpr::Gradient(Box::new(self.primary()?)))
                } else {
                    Ok(MathExpr::Gradient(Box::new(self.primary()?)))
                }
            }
            "operatorname" => {
                let name = self
                    .group_or_one()?
                    .iter()
                    .map(token_text)
                    .collect::<String>();
                let arg = self.primary()?;
                Ok(MathExpr::Call {
                    function: name,
                    args: vec![arg],
                })
            }
            "sin" | "cos" | "tan" | "exp" | "log" | "sqrt" => Ok(MathExpr::Call {
                function: command,
                args: vec![self.primary()?],
            }),
            "rho" | "epsilon" | "varepsilon" | "mu" | "nu" | "sigma" | "lambda" | "phi" | "psi"
            | "theta" | "alpha" | "beta" | "gamma" | "omega" | "Omega" | "Gamma" => {
                Ok(MathExpr::Name(command))
            }
            _ => Err(LatexError::UnsupportedCommand(command)).map_err(|e| match e {
                LatexError::UnsupportedCommand(c) => LatexError::Unexpected {
                    offset,
                    message: format!("unsupported LaTeX command `\\{c}`"),
                },
                other => other,
            }),
        }
    }

    fn group_or_one(&mut self) -> Result<Vec<Tok>, LatexError> {
        if matches!(self.tokens.get(self.pos).map(|t| &t.kind), Some(TokKind::L)) {
            self.pos += 1;
            let start = self.pos;
            let end = matching_end(self.tokens, start)?;
            let out = self.tokens[start..end].to_vec();
            self.pos = end + 1;
            Ok(out)
        } else {
            let Some(token) = self.tokens.get(self.pos).cloned() else {
                return Err(LatexError::UnclosedGroup);
            };
            self.pos += 1;
            Ok(vec![token])
        }
    }
}

fn matching_end(tokens: &[Tok], start: usize) -> Result<usize, LatexError> {
    let mut depth = 0usize;
    for (i, token) in tokens.iter().enumerate().skip(start) {
        match token.kind {
            TokKind::L => depth += 1,
            TokKind::R if depth == 0 => return Ok(i),
            TokKind::R => depth -= 1,
            _ => {}
        }
    }
    Err(LatexError::UnclosedGroup)
}

fn derivative_fraction(
    numerator: &[Tok],
    denominator: &[Tok],
) -> Result<Option<(MathExpr, String, u8)>, LatexError> {
    let Some(Tok {
        kind: TokKind::Command(p),
        ..
    }) = numerator.first()
    else {
        return Ok(None);
    };
    if p != "partial" {
        return Ok(None);
    }
    let Some(Tok {
        kind: TokKind::Command(d),
        ..
    }) = denominator.first()
    else {
        return Ok(None);
    };
    if d != "partial" || denominator.len() < 2 {
        return Ok(None);
    }
    let wrt = token_text(&denominator[1]);
    let expr = parse_tokens(&numerator[1..])?;
    Ok(Some((expr, wrt, 1)))
}

fn token_text(token: &Tok) -> String {
    match &token.kind {
        TokKind::Name(s) | TokKind::Number(s) | TokKind::Command(s) => s.clone(),
        TokKind::Minus => "-".into(),
        TokKind::Plus => "+".into(),
        TokKind::Star => "*".into(),
        TokKind::Slash => "/".into(),
        TokKind::Caret => "^".into(),
        TokKind::Eq => "=".into(),
        TokKind::Underscore => "_".into(),
        TokKind::Comma => ",".into(),
        TokKind::L => "(".into(),
        TokKind::R => ")".into(),
    }
}

fn starts_primary(kind: &TokKind) -> bool {
    matches!(
        kind,
        TokKind::Name(_) | TokKind::Number(_) | TokKind::Command(_) | TokKind::L
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_time_derivative() {
        let eq = parse_equation(r"\rho c_p \frac{\partial T}{\partial t} = Q").unwrap();
        assert!(matches!(eq.lhs, MathExpr::Mul(_)));
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(parse_expr(r"\magic{x}").is_err());
    }
}
