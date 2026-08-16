use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Div, Mul};

/// SI base-dimension exponent vector in the order M, L, T, I, Θ, N, J.
///
/// Dimensions are semantic compiler data: addition requires equality while multiplication
/// and division compose exponents. Unit scale/offset belongs to the authoring layer; the
/// scientific IR stores canonical-SI dimensions.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Dimension(pub [i8; 7]);

impl Dimension {
    pub const DIMENSIONLESS: Self = Self([0; 7]);
    pub const MASS: Self = Self([1, 0, 0, 0, 0, 0, 0]);
    pub const LENGTH: Self = Self([0, 1, 0, 0, 0, 0, 0]);
    pub const TIME: Self = Self([0, 0, 1, 0, 0, 0, 0]);
    pub const CURRENT: Self = Self([0, 0, 0, 1, 0, 0, 0]);
    pub const TEMPERATURE: Self = Self([0, 0, 0, 0, 1, 0, 0]);
    pub const AMOUNT: Self = Self([0, 0, 0, 0, 0, 1, 0]);
    pub const LUMINOUS_INTENSITY: Self = Self([0, 0, 0, 0, 0, 0, 1]);

    pub const fn powi(self, n: i8) -> Self {
        let a = self.0;
        Self([
            a[0] * n,
            a[1] * n,
            a[2] * n,
            a[3] * n,
            a[4] * n,
            a[5] * n,
            a[6] * n,
        ])
    }

    pub fn parse(unit: &str) -> Result<Self, UnitError> {
        UnitParser::new(unit).parse()
    }
}

impl Mul for Dimension {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut out = [0; 7];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.0[i] + rhs.0[i];
        }
        Self(out)
    }
}

impl Div for Dimension {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let mut out = [0; 7];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.0[i] - rhs.0[i];
        }
        Self(out)
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const NAMES: [&str; 7] = ["kg", "m", "s", "A", "K", "mol", "cd"];
        let mut first = true;
        for (name, exp) in NAMES.into_iter().zip(self.0) {
            if exp == 0 {
                continue;
            }
            if !first {
                write!(f, " ")?;
            }
            first = false;
            if exp == 1 {
                write!(f, "{name}")?;
            } else {
                write!(f, "{name}^{exp}")?;
            }
        }
        if first { write!(f, "1") } else { Ok(()) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum UnitError {
    #[error("unknown unit `{0}`")]
    Unknown(String),
    #[error("invalid unit exponent in `{0}`")]
    BadExponent(String),
    #[error("unexpected token in unit expression near `{0}`")]
    Syntax(String),
}

struct UnitParser<'a> {
    input: &'a str,
    pos: usize,
}
impl<'a> UnitParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
    fn parse(mut self) -> Result<Dimension, UnitError> {
        self.skip_ws();
        if self.input[self.pos..].trim().is_empty() {
            return Ok(Dimension::DIMENSIONLESS);
        }
        let mut value = self.factor()?;
        loop {
            self.skip_ws();
            if self.pos >= self.input.len() {
                break;
            }
            match self.peek() {
                Some('*') => {
                    self.pos += 1;
                    value = value * self.factor()?;
                }
                Some('/') => {
                    self.pos += 1;
                    value = value / self.factor()?;
                }
                Some(c) if c.is_alphabetic() || c == '(' => {
                    value = value * self.factor()?;
                }
                _ => return Err(UnitError::Syntax(self.input[self.pos..].to_string())),
            }
        }
        Ok(value)
    }
    fn factor(&mut self) -> Result<Dimension, UnitError> {
        self.skip_ws();
        let mut value = if self.peek() == Some('(') {
            self.pos += 1;
            let start = self.pos;
            let mut depth = 1usize;
            while self.pos < self.input.len() && depth > 0 {
                match self.peek() {
                    Some('(') => depth += 1,
                    Some(')') => depth -= 1,
                    _ => {}
                }
                self.pos += 1;
            }
            if depth != 0 {
                return Err(UnitError::Syntax(self.input[start..].to_string()));
            }
            Dimension::parse(&self.input[start..self.pos - 1])?
        } else {
            let start = self.pos;
            while matches!(self.peek(), Some(c) if c.is_alphabetic() || c == '_' || c == '1') {
                self.pos += 1;
            }
            let name = &self.input[start..self.pos];
            unit_atom(name).ok_or_else(|| UnitError::Unknown(name.to_string()))?
        };
        self.skip_ws();
        if self.peek() == Some('^') {
            self.pos += 1;
            let start = self.pos;
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
            let text = &self.input[start..self.pos];
            let n: i8 = text
                .parse()
                .map_err(|_| UnitError::BadExponent(text.to_string()))?;
            value = value.powi(n);
        }
        Ok(value)
    }
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.pos += c.len_utf8();
        }
    }
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }
}

fn unit_atom(name: &str) -> Option<Dimension> {
    let m = Dimension::MASS;
    let l = Dimension::LENGTH;
    let t = Dimension::TIME;
    let i = Dimension::CURRENT;
    let th = Dimension::TEMPERATURE;
    let n = Dimension::AMOUNT;
    Some(match name {
        "" | "1" | "rad" => Dimension::DIMENSIONLESS,
        "kg" => m,
        "m" => l,
        "s" => t,
        "A" => i,
        "K" => th,
        "mol" => n,
        "cd" => Dimension::LUMINOUS_INTENSITY,
        "Hz" => t.powi(-1),
        "N" => m * l / t.powi(2),
        "Pa" => m / l / t.powi(2),
        "J" => m * l.powi(2) / t.powi(2),
        "W" => m * l.powi(2) / t.powi(3),
        "C" => i * t,
        "V" => m * l.powi(2) / t.powi(3) / i,
        "Ohm" => m * l.powi(2) / t.powi(3) / i.powi(2),
        "F" => t.powi(4) * i.powi(2) / m / l.powi(2),
        "H" => m * l.powi(2) / t.powi(2) / i.powi(2),
        "T" => m / t.powi(2) / i,
        "Wb" => m * l.powi(2) / t.powi(2) / i,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_thermal_conductivity() {
        assert_eq!(
            Dimension::parse("W / (m K)").unwrap(),
            Dimension::MASS * Dimension::LENGTH / Dimension::TIME.powi(3) / Dimension::TEMPERATURE
        );
    }
    #[test]
    fn parses_density() {
        assert_eq!(
            Dimension::parse("kg / m^3").unwrap(),
            Dimension::MASS / Dimension::LENGTH.powi(3)
        );
    }
}
