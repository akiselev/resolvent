use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// SI base-dimension exponents in the order length, mass, time, current,
/// temperature, amount of substance, luminous intensity.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Dimension(pub [i8; 7]);

impl Dimension {
    pub const DIMENSIONLESS: Self = Self([0; 7]);
    pub const LENGTH: Self = Self([1, 0, 0, 0, 0, 0, 0]);
    pub const MASS: Self = Self([0, 1, 0, 0, 0, 0, 0]);
    pub const TIME: Self = Self([0, 0, 1, 0, 0, 0, 0]);
    pub const CURRENT: Self = Self([0, 0, 0, 1, 0, 0, 0]);
    pub const TEMPERATURE: Self = Self([0, 0, 0, 0, 1, 0, 0]);
    pub const AMOUNT: Self = Self([0, 0, 0, 0, 0, 1, 0]);
    pub const LUMINOUS_INTENSITY: Self = Self([0, 0, 0, 0, 0, 0, 1]);

    pub fn mul(self, rhs: Self) -> Self {
        let mut out = [0; 7];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.0[i].saturating_add(rhs.0[i]);
        }
        Self(out)
    }

    pub fn div(self, rhs: Self) -> Self {
        self.mul(rhs.powi(-1))
    }

    pub fn powi(self, exponent: i8) -> Self {
        let mut out = [0; 7];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.0[i].saturating_mul(exponent);
        }
        Self(out)
    }

    pub fn named(name: &str) -> Option<Self> {
        let l = Self::LENGTH;
        let m = Self::MASS;
        let t = Self::TIME;
        let i = Self::CURRENT;
        match name {
            "1" => Some(Self::DIMENSIONLESS),
            "m" => Some(l),
            "kg" => Some(m),
            "s" => Some(t),
            "A" => Some(i),
            "K" => Some(Self::TEMPERATURE),
            "mol" => Some(Self::AMOUNT),
            "cd" => Some(Self::LUMINOUS_INTENSITY),
            "Hz" => Some(t.powi(-1)),
            "N" => Some(m.mul(l).div(t.powi(2))),
            "Pa" => Some(m.div(l).div(t.powi(2))),
            "J" => Some(m.mul(l.powi(2)).div(t.powi(2))),
            "W" => Some(m.mul(l.powi(2)).div(t.powi(3))),
            "C" => Some(i.mul(t)),
            "V" => Some(m.mul(l.powi(2)).div(t.powi(3)).div(i)),
            "ohm" | "Ohm" | "Ω" => Some(m.mul(l.powi(2)).div(t.powi(3)).div(i.powi(2))),
            "F" => Some(m.powi(-1).mul(l.powi(-2)).mul(t.powi(4)).mul(i.powi(2))),
            "H" => Some(m.mul(l.powi(2)).div(t.powi(2)).div(i.powi(2))),
            _ => None,
        }
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const NAMES: [&str; 7] = ["m", "kg", "s", "A", "K", "mol", "cd"];
        let mut first = true;
        for (name, exponent) in NAMES.iter().zip(self.0) {
            if exponent == 0 {
                continue;
            }
            if !first {
                write!(f, "*")?;
            }
            first = false;
            if exponent == 1 {
                write!(f, "{name}")?;
            } else {
                write!(f, "{name}^{exponent}")?;
            }
        }
        if first { write!(f, "1") } else { Ok(()) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitExpr {
    pub source: String,
    pub dimension: Dimension,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UnitError {
    #[error("unknown unit `{0}`")]
    UnknownUnit(String),
    #[error("expected a unit atom")]
    ExpectedAtom,
    #[error("expected integer exponent")]
    ExpectedExponent,
    #[error("unclosed unit parenthesis")]
    UnclosedParenthesis,
    #[error("unexpected token in unit expression")]
    UnexpectedToken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Name(String),
    Mul,
    Div,
    Pow,
    LParen,
    RParen,
    Int(i8),
}

pub fn parse_unit(source: &str) -> Result<UnitExpr, UnitError> {
    let tokens = lex(source)?;
    let mut parser = UnitParser {
        tokens: &tokens,
        pos: 0,
    };
    let dimension = parser.expr()?;
    if parser.pos != tokens.len() {
        return Err(UnitError::UnexpectedToken);
    }
    Ok(UnitExpr {
        source: source.trim().to_string(),
        dimension,
    })
}

fn lex(source: &str) -> Result<Vec<Tok>, UnitError> {
    let mut out = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            c if c.is_whitespace() => i += 1,
            '*' | '·' => {
                out.push(Tok::Mul);
                i += 1;
            }
            '/' => {
                out.push(Tok::Div);
                i += 1;
            }
            '^' => {
                out.push(Tok::Pow);
                i += 1;
            }
            '(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            c if c.is_ascii_digit() || c == '-' || c == '+' => {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let value = text.parse().map_err(|_| UnitError::ExpectedExponent)?;
                out.push(Tok::Int(value));
            }
            c if c.is_alphabetic() || c == 'Ω' => {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                out.push(Tok::Name(chars[start..i].iter().collect()));
            }
            _ => return Err(UnitError::UnexpectedToken),
        }
    }
    Ok(out)
}

struct UnitParser<'a> {
    tokens: &'a [Tok],
    pos: usize,
}

impl UnitParser<'_> {
    fn expr(&mut self) -> Result<Dimension, UnitError> {
        let mut value = self.factor()?;
        loop {
            match self.tokens.get(self.pos) {
                Some(Tok::Mul) => {
                    self.pos += 1;
                    value = value.mul(self.factor()?);
                }
                Some(Tok::Div) => {
                    self.pos += 1;
                    value = value.div(self.factor()?);
                }
                Some(Tok::Name(_)) | Some(Tok::LParen) => value = value.mul(self.factor()?),
                _ => break,
            }
        }
        Ok(value)
    }

    fn factor(&mut self) -> Result<Dimension, UnitError> {
        let mut value = match self.tokens.get(self.pos) {
            Some(Tok::Name(name)) => {
                self.pos += 1;
                Dimension::named(name).ok_or_else(|| UnitError::UnknownUnit(name.clone()))?
            }
            Some(Tok::Int(1)) => {
                self.pos += 1;
                Dimension::DIMENSIONLESS
            }
            Some(Tok::LParen) => {
                self.pos += 1;
                let inner = self.expr()?;
                if !matches!(self.tokens.get(self.pos), Some(Tok::RParen)) {
                    return Err(UnitError::UnclosedParenthesis);
                }
                self.pos += 1;
                inner
            }
            _ => return Err(UnitError::ExpectedAtom),
        };
        if matches!(self.tokens.get(self.pos), Some(Tok::Pow)) {
            self.pos += 1;
            let Some(Tok::Int(exp)) = self.tokens.get(self.pos) else {
                return Err(UnitError::ExpectedExponent);
            };
            value = value.powi(*exp);
            self.pos += 1;
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_thermal_conductivity() {
        let got = parse_unit("W / (m K)").unwrap().dimension;
        let want = Dimension::named("W")
            .unwrap()
            .div(Dimension::LENGTH)
            .div(Dimension::TEMPERATURE);
        assert_eq!(got, want);
    }

    #[test]
    fn parses_heat_capacity_density() {
        let got = parse_unit("J / (m^3 K)").unwrap().dimension;
        assert_eq!(
            got,
            Dimension::named("J")
                .unwrap()
                .div(Dimension::LENGTH.powi(3))
                .div(Dimension::TEMPERATURE)
        );
    }
}
