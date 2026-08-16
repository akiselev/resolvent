use crate::source::{SourceDiagnostic, SourceSpan};
use serde::{Deserialize, Serialize};

/// Deliberately constrained scientific-LaTeX AST. This is source syntax, never the semantic
/// Resolvent IR. Unsupported TeX is rejected instead of guessed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathExpr {
    Number(String),
    Name(String),
    Neg(Box<MathExpr>),
    Add(Vec<MathExpr>),
    Mul(Vec<MathExpr>),
    Div(Box<MathExpr>, Box<MathExpr>),
    Pow(Box<MathExpr>, i32),
    Call { name: String, args: Vec<MathExpr> },
    Grad(Box<MathExpr>),
    DivOp(Box<MathExpr>),
    Curl(Box<MathExpr>),
    Dt(Box<MathExpr>),
    Dot(Box<MathExpr>, Box<MathExpr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Name(String),
    Number(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Comma,
    Grad,
    DivOp,
    Curl,
    Dt,
    Dot,
}

pub fn parse_scientific_latex(input: &str) -> Result<MathExpr, Vec<SourceDiagnostic>> {
    let normalized = normalize(input)?;
    let tokens = lex(&normalized)?;
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
    };
    match parser.expr() {
        Ok(expr) if parser.pos == tokens.len() => Ok(expr),
        Ok(_) => Err(vec![
            SourceDiagnostic::error(
                "RSL-LATEX-003",
                "unexpected trailing mathematical tokens",
                SourceSpan::new(0, input.len()),
            )
            .phase("latex"),
        ]),
        Err(message) => Err(vec![
            SourceDiagnostic::error("RSL-LATEX-002", message, SourceSpan::new(0, input.len()))
                .phase("latex"),
        ]),
    }
}

/// Rewrites a small, explicit TeX vocabulary into a lexer-friendly source. The transform is
/// mechanical and intentionally fail-closed: arbitrary TeX macros never enter the compiler.
fn normalize(input: &str) -> Result<String, Vec<SourceDiagnostic>> {
    let mut s = input.replace("\\left", "").replace("\\right", "");
    s = s.replace("\\cdot", " @dot ");
    s = s.replace("\\times", " * ");
    s = s.replace("\\nabla \\cdot", " @div ");
    s = s.replace("\\nabla\\cdot", " @div ");
    s = s.replace("\\nabla \\times", " @curl ");
    s = s.replace("\\nabla\\times", " @curl ");
    s = s.replace("\\nabla", " @grad ");

    // Common partial-time derivative spellings.
    while let Some(start) = s.find("\\frac{\\partial ") {
        let body_start = start + "\\frac{\\partial ".len();
        let Some(body_end_rel) = s[body_start..].find("}{\\partial t}") else {
            break;
        };
        let body_end = body_start + body_end_rel;
        let body = s[body_start..body_end].trim();
        s.replace_range(
            start..body_end + "}{\\partial t}".len(),
            &format!(" @dt ({body}) "),
        );
    }
    while let Some(start) = s.find("\\partial_t ") {
        let tail = &s[start + "\\partial_t ".len()..];
        let name_len = tail
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .map(char::len_utf8)
            .sum::<usize>();
        if name_len == 0 {
            break;
        }
        let name = &tail[..name_len];
        s.replace_range(
            start..start + "\\partial_t ".len() + name_len,
            &format!(" @dt ({name}) "),
        );
    }

    s = replace_frac(s)?;
    s = replace_braces(s);
    for (tex, plain) in [
        ("\\rho", "rho"),
        ("\\epsilon", "epsilon"),
        ("\\varepsilon", "epsilon"),
        ("\\mu", "mu"),
        ("\\sigma", "sigma"),
        ("\\lambda", "lambda"),
        ("\\theta", "theta"),
        ("\\phi", "phi"),
        ("\\psi", "psi"),
        ("\\omega", "omega"),
        ("\\alpha", "alpha"),
        ("\\beta", "beta"),
        ("\\gamma", "gamma"),
        ("\\Delta", "Delta"),
    ] {
        s = s.replace(tex, plain);
    }

    if let Some(pos) = s.find('\\') {
        return Err(vec![SourceDiagnostic::error("RSL-LATEX-001", format!("unsupported LaTeX command near `{}`", &s[pos..].chars().take(24).collect::<String>()), SourceSpan::new(pos, (pos + 1).min(input.len()))).hint("use the constrained scientific vocabulary or register a semantic function explicitly").phase("latex")]);
    }
    Ok(s)
}

fn replace_frac(mut s: String) -> Result<String, Vec<SourceDiagnostic>> {
    loop {
        let Some(start) = s.find("\\frac{") else {
            break;
        };
        let num_open = start + "\\frac".len();
        let Some((num, after_num)) = braced(&s, num_open) else {
            return Err(vec![
                SourceDiagnostic::error(
                    "RSL-LATEX-004",
                    "malformed \\frac numerator",
                    SourceSpan::new(start, s.len()),
                )
                .phase("latex"),
            ]);
        };
        let Some((den, after_den)) = braced(&s, after_num) else {
            return Err(vec![
                SourceDiagnostic::error(
                    "RSL-LATEX-004",
                    "malformed \\frac denominator",
                    SourceSpan::new(start, s.len()),
                )
                .phase("latex"),
            ]);
        };
        s.replace_range(start..after_den, &format!("(({num})/({den}))"));
    }
    Ok(s)
}

fn braced(s: &str, open: usize) -> Option<(String, usize)> {
    if s.as_bytes().get(open).copied()? != b'{' {
        return None;
    }
    let mut depth = 0usize;
    for (off, ch) in s[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((s[open + 1..open + off].to_string(), open + off + 1));
                }
            }
            _ => {}
        }
    }
    None
}

fn replace_braces(s: String) -> String {
    s.replace('{', "(").replace('}', ")")
}

fn lex(s: &str) -> Result<Vec<Tok>, Vec<SourceDiagnostic>> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        if ch.is_whitespace() {
            i += ch.len_utf8();
            continue;
        }
        let rem = &s[i..];
        for (prefix, tok) in [
            ("@grad", Tok::Grad),
            ("@div", Tok::DivOp),
            ("@curl", Tok::Curl),
            ("@dt", Tok::Dt),
            ("@dot", Tok::Dot),
        ] {
            if rem.starts_with(prefix) {
                out.push(tok);
                i += prefix.len();
                break;
            }
        }
        if i >= s.len() {
            break;
        }
        if matches!(
            out.last(),
            Some(Tok::Grad | Tok::DivOp | Tok::Curl | Tok::Dt | Tok::Dot)
        ) && rem.starts_with('@')
        {
            continue;
        }
        let ch = s[i..].chars().next().unwrap();
        let single = match ch {
            '+' => Some(Tok::Plus),
            '-' => Some(Tok::Minus),
            '*' => Some(Tok::Star),
            '/' => Some(Tok::Slash),
            '^' => Some(Tok::Caret),
            '(' => Some(Tok::LParen),
            ')' => Some(Tok::RParen),
            ',' => Some(Tok::Comma),
            _ => None,
        };
        if let Some(t) = single {
            out.push(t);
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() || ch == '.' {
            let start = i;
            i += ch.len_utf8();
            while i < s.len() {
                let c = s[i..].chars().next().unwrap();
                if c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-') {
                    i += c.len_utf8();
                } else {
                    break;
                }
            }
            out.push(Tok::Number(s[start..i].to_string()));
            continue;
        }
        if ch.is_alphabetic() || ch == '_' {
            let start = i;
            i += ch.len_utf8();
            while i < s.len() {
                let c = s[i..].chars().next().unwrap();
                if c.is_alphanumeric() || c == '_' {
                    i += c.len_utf8();
                } else {
                    break;
                }
            }
            out.push(Tok::Name(s[start..i].to_string()));
            continue;
        }
        return Err(vec![
            SourceDiagnostic::error(
                "RSL-LATEX-005",
                format!("unsupported character `{ch}`"),
                SourceSpan::new(i, i + ch.len_utf8()),
            )
            .phase("latex"),
        ]);
    }
    Ok(out)
}

struct Parser<'a> {
    tokens: &'a [Tok],
    pos: usize,
}
impl Parser<'_> {
    fn expr(&mut self) -> Result<MathExpr, String> {
        let mut terms = vec![self.product()?];
        loop {
            match self.peek() {
                Some(Tok::Plus) => {
                    self.pos += 1;
                    terms.push(self.product()?);
                }
                Some(Tok::Minus) => {
                    self.pos += 1;
                    terms.push(MathExpr::Neg(Box::new(self.product()?)));
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
    fn product(&mut self) -> Result<MathExpr, String> {
        let mut lhs = self.power()?;
        loop {
            match self.peek() {
                Some(Tok::Star) => {
                    self.pos += 1;
                    lhs = mul(lhs, self.power()?);
                }
                Some(Tok::Slash) => {
                    self.pos += 1;
                    lhs = MathExpr::Div(Box::new(lhs), Box::new(self.power()?));
                }
                Some(Tok::Dot) => {
                    self.pos += 1;
                    lhs = MathExpr::Dot(Box::new(lhs), Box::new(self.power()?));
                }
                Some(
                    Tok::Name(_)
                    | Tok::Number(_)
                    | Tok::LParen
                    | Tok::Grad
                    | Tok::DivOp
                    | Tok::Curl
                    | Tok::Dt,
                ) => {
                    lhs = mul(lhs, self.power()?);
                }
                _ => break,
            }
        }
        Ok(lhs)
    }
    fn power(&mut self) -> Result<MathExpr, String> {
        let mut v = self.unary()?;
        if matches!(self.peek(), Some(Tok::Caret)) {
            self.pos += 1;
            let sign = if matches!(self.peek(), Some(Tok::Minus)) {
                self.pos += 1;
                -1
            } else {
                1
            };
            let Some(Tok::Number(n)) = self.peek().cloned() else {
                return Err("integer exponent required after ^".into());
            };
            self.pos += 1;
            let p: i32 = n.parse().map_err(|_| "integer exponent required after ^")?;
            v = MathExpr::Pow(Box::new(v), sign * p);
        }
        Ok(v)
    }
    fn unary(&mut self) -> Result<MathExpr, String> {
        match self.peek() {
            Some(Tok::Minus) => {
                self.pos += 1;
                Ok(MathExpr::Neg(Box::new(self.unary()?)))
            }
            Some(Tok::Grad) => {
                self.pos += 1;
                Ok(MathExpr::Grad(Box::new(self.unary()?)))
            }
            Some(Tok::DivOp) => {
                self.pos += 1;
                Ok(MathExpr::DivOp(Box::new(self.unary()?)))
            }
            Some(Tok::Curl) => {
                self.pos += 1;
                Ok(MathExpr::Curl(Box::new(self.unary()?)))
            }
            Some(Tok::Dt) => {
                self.pos += 1;
                Ok(MathExpr::Dt(Box::new(self.unary()?)))
            }
            _ => self.atom(),
        }
    }
    fn atom(&mut self) -> Result<MathExpr, String> {
        let tok = self.peek().cloned().ok_or("expected expression")?;
        self.pos += 1;
        match tok {
            Tok::Number(n) => Ok(MathExpr::Number(n)),
            Tok::Name(name) => {
                if matches!(self.peek(), Some(Tok::LParen)) {
                    self.pos += 1;
                    let mut args = vec![];
                    if !matches!(self.peek(), Some(Tok::RParen)) {
                        loop {
                            args.push(self.expr()?);
                            if matches!(self.peek(), Some(Tok::Comma)) {
                                self.pos += 1;
                            } else {
                                break;
                            }
                        }
                    }
                    if !matches!(self.peek(), Some(Tok::RParen)) {
                        return Err(format!("missing ) after call to {name}"));
                    }
                    self.pos += 1;
                    Ok(MathExpr::Call { name, args })
                } else {
                    Ok(MathExpr::Name(name))
                }
            }
            Tok::LParen => {
                let v = self.expr()?;
                if !matches!(self.peek(), Some(Tok::RParen)) {
                    return Err("missing closing )".into());
                }
                self.pos += 1;
                Ok(v)
            }
            _ => Err("expected number, symbol or parenthesized expression".into()),
        }
    }
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos)
    }
}

fn mul(a: MathExpr, b: MathExpr) -> MathExpr {
    match (a, b) {
        (MathExpr::Mul(mut a), MathExpr::Mul(b)) => {
            a.extend(b);
            MathExpr::Mul(a)
        }
        (MathExpr::Mul(mut a), b) => {
            a.push(b);
            MathExpr::Mul(a)
        }
        (a, MathExpr::Mul(mut b)) => {
            b.insert(0, a);
            MathExpr::Mul(b)
        }
        (a, b) => MathExpr::Mul(vec![a, b]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_nonlinear_heat_fragment() {
        let e = parse_scientific_latex(
            r"\rho c_p(T) \frac{\partial T}{\partial t} + k(T) \nabla T \cdot \nabla T",
        )
        .unwrap();
        assert!(matches!(e, MathExpr::Add(_)));
    }
    #[test]
    fn rejects_unknown_tex() {
        assert!(parse_scientific_latex(r"\unknown{x}").is_err());
    }
}
