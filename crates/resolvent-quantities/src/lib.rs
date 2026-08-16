#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

/// SI base-dimension exponent vector ordered M, L, T, I, Θ, N, J.
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

    pub const fn product(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        Self([
            a[0] + b[0],
            a[1] + b[1],
            a[2] + b[2],
            a[3] + b[3],
            a[4] + b[4],
            a[5] + b[5],
            a[6] + b[6],
        ])
    }

    pub const fn quotient(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        Self([
            a[0] - b[0],
            a[1] - b[1],
            a[2] - b[2],
            a[3] - b[3],
            a[4] - b[4],
            a[5] - b[5],
            a[6] - b[6],
        ])
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const NAMES: [&str; 7] = ["kg", "m", "s", "A", "K", "mol", "cd"];
        let mut first = true;
        for (name, exponent) in NAMES.into_iter().zip(self.0) {
            if exponent == 0 {
                continue;
            }
            if !first {
                write!(f, " ")?;
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct QuantityKindId(pub String);

impl QuantityKindId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn thermodynamic_temperature() -> Self {
        Self("si:ThermodynamicTemperature".into())
    }
    pub fn temperature_difference() -> Self {
        Self("si:TemperatureDifference".into())
    }
    pub fn energy() -> Self {
        Self("si:Energy".into())
    }
    pub fn moment_of_force() -> Self {
        Self("si:MomentOfForce".into())
    }
    pub fn pressure() -> Self {
        Self("si:Pressure".into())
    }
    pub fn stress() -> Self {
        Self("resolvent:Stress".into())
    }
    pub fn strain() -> Self {
        Self("resolvent:Strain".into())
    }
    pub fn thermal_conductivity() -> Self {
        Self("resolvent:ThermalConductivity".into())
    }
    pub fn electrical_conductivity() -> Self {
        Self("resolvent:ElectricalConductivity".into())
    }
    pub fn specific_heat_capacity() -> Self {
        Self("resolvent:SpecificHeatCapacity".into())
    }
    pub fn density() -> Self {
        Self("resolvent:MassDensity".into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UnitId(pub String);
impl UnitId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Exact decimal/rational scale. A value is numerator/denominator * 10^power10.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactScale {
    pub numerator: i128,
    pub denominator: i128,
    pub power10: i32,
}
impl ExactScale {
    pub const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
        power10: 0,
    };
    pub const fn new(numerator: i128, denominator: i128, power10: i32) -> Self {
        Self {
            numerator,
            denominator,
            power10,
        }
    }
    pub fn as_f64(self) -> f64 {
        (self.numerator as f64 / self.denominator as f64) * 10_f64.powi(self.power10)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitProvenance {
    pub authority: String,
    pub version: String,
    pub persistent_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: UnitId,
    pub symbol: String,
    pub dimension: Dimension,
    pub scale_to_si: ExactScale,
    pub offset_to_si: Option<ExactScale>,
    pub quantity_kind_constraints: Vec<QuantityKindId>,
    pub interval_form: Option<UnitId>,
    pub provenance: UnitProvenance,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuantityLiteral {
    pub value: f64,
    pub unit: UnitId,
    pub quantity_kind: QuantityKindId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanonicalQuantity {
    pub value_si: f64,
    pub dimension: Dimension,
    pub quantity_kind: QuantityKindId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayUnit(pub UnitId);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bound<T> {
    pub value: T,
    pub inclusive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KindStrictness {
    DimensionOnly,
    KindCompatible,
    ExactKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitRegistry {
    units: BTreeMap<UnitId, UnitDef>,
    symbols: BTreeMap<String, UnitId>,
    pub snapshot: RegistrySnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub source: String,
    pub version: String,
    pub retrieval_date: String,
    pub content_digest: String,
    pub generator_version: String,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum QuantityError {
    #[error("unknown unit `{0}`")]
    UnknownUnit(String),
    #[error("unit `{unit}` is dimensionally incompatible with quantity kind `{kind}`")]
    KindMismatch { unit: String, kind: String },
    #[error("offset unit `{0}` cannot be used as an interval without an interval form")]
    MissingIntervalForm(String),
    #[error("non-finite quantity value")]
    NonFinite,
}

impl UnitRegistry {
    pub fn empty(snapshot: RegistrySnapshot) -> Self {
        Self {
            units: BTreeMap::new(),
            symbols: BTreeMap::new(),
            snapshot,
        }
    }

    pub fn insert(&mut self, def: UnitDef) {
        self.symbols.insert(def.symbol.clone(), def.id.clone());
        self.units.insert(def.id.clone(), def);
    }

    pub fn get(&self, id: &UnitId) -> Option<&UnitDef> {
        self.units.get(id)
    }
    pub fn by_symbol(&self, symbol: &str) -> Option<&UnitDef> {
        self.symbols.get(symbol).and_then(|id| self.units.get(id))
    }

    pub fn canonicalize(
        &self,
        literal: &QuantityLiteral,
    ) -> Result<CanonicalQuantity, QuantityError> {
        if !literal.value.is_finite() {
            return Err(QuantityError::NonFinite);
        }
        let mut unit = self
            .get(&literal.unit)
            .ok_or_else(|| QuantityError::UnknownUnit(literal.unit.0.clone()))?;
        let is_interval = literal.quantity_kind == QuantityKindId::temperature_difference();
        if is_interval && unit.offset_to_si.is_some() {
            let interval = unit
                .interval_form
                .as_ref()
                .ok_or_else(|| QuantityError::MissingIntervalForm(unit.symbol.clone()))?;
            unit = self
                .get(interval)
                .ok_or_else(|| QuantityError::UnknownUnit(interval.0.clone()))?;
        }
        if !unit.quantity_kind_constraints.is_empty()
            && !unit
                .quantity_kind_constraints
                .iter()
                .any(|kind| kind == &literal.quantity_kind)
        {
            return Err(QuantityError::KindMismatch {
                unit: unit.symbol.clone(),
                kind: literal.quantity_kind.0.clone(),
            });
        }
        let mut value = literal.value * unit.scale_to_si.as_f64();
        if !is_interval {
            if let Some(offset) = unit.offset_to_si {
                value += offset.as_f64();
            }
        }
        Ok(CanonicalQuantity {
            value_si: value,
            dimension: unit.dimension,
            quantity_kind: literal.quantity_kind.clone(),
        })
    }

    pub fn standard() -> Self {
        let sirp = UnitProvenance {
            authority: "BIPM SI Reference Point".into(),
            version: "1.0.0-vendored-subset".into(),
            persistent_id: None,
        };
        let mut out = Self::empty(RegistrySnapshot {
            source: "vendored://bipm-si-reference-point".into(),
            version: "1.0.0-vendored-subset".into(),
            retrieval_date: "2026-08-16".into(),
            content_digest: "bootstrap-subset; regenerate with tools/update-sirp".into(),
            generator_version: "resolvent/update-sirp/1".into(),
        });
        let mut add = |id: &str,
                       symbol: &str,
                       dimension: Dimension,
                       scale: ExactScale,
                       offset: Option<ExactScale>,
                       kinds: Vec<QuantityKindId>,
                       interval: Option<&str>| {
            out.insert(UnitDef {
                id: UnitId::new(id),
                symbol: symbol.into(),
                dimension,
                scale_to_si: scale,
                offset_to_si: offset,
                quantity_kind_constraints: kinds,
                interval_form: interval.map(UnitId::new),
                provenance: sirp.clone(),
            });
        };
        add(
            "si:kelvin",
            "K",
            Dimension::TEMPERATURE,
            ExactScale::ONE,
            None,
            vec![
                QuantityKindId::thermodynamic_temperature(),
                QuantityKindId::temperature_difference(),
            ],
            None,
        );
        add(
            "si:degree-celsius",
            "degC",
            Dimension::TEMPERATURE,
            ExactScale::ONE,
            Some(ExactScale::new(27315, 100, 0)),
            vec![QuantityKindId::thermodynamic_temperature()],
            Some("resolvent:degree-celsius-difference"),
        );
        add(
            "resolvent:degree-celsius-difference",
            "delta_degC",
            Dimension::TEMPERATURE,
            ExactScale::ONE,
            None,
            vec![QuantityKindId::temperature_difference()],
            None,
        );
        let energy_dim = Dimension::MASS
            .product(Dimension::LENGTH.powi(2))
            .quotient(Dimension::TIME.powi(2));
        add(
            "si:joule",
            "J",
            energy_dim,
            ExactScale::ONE,
            None,
            vec![QuantityKindId::energy()],
            None,
        );
        add(
            "si:newton-metre-torque",
            "N*m",
            energy_dim,
            ExactScale::ONE,
            None,
            vec![QuantityKindId::moment_of_force()],
            None,
        );
        add(
            "si:pascal",
            "Pa",
            Dimension::MASS
                .quotient(Dimension::LENGTH)
                .quotient(Dimension::TIME.powi(2)),
            ExactScale::ONE,
            None,
            vec![QuantityKindId::pressure(), QuantityKindId::stress()],
            None,
        );
        add(
            "si:kilogram-per-cubic-metre",
            "kg/m^3",
            Dimension::MASS.quotient(Dimension::LENGTH.powi(3)),
            ExactScale::ONE,
            None,
            vec![QuantityKindId::density()],
            None,
        );
        add(
            "si:watt-per-metre-kelvin",
            "W/(m*K)",
            Dimension::MASS
                .product(Dimension::LENGTH)
                .quotient(Dimension::TIME.powi(3))
                .quotient(Dimension::TEMPERATURE),
            ExactScale::ONE,
            None,
            vec![QuantityKindId::thermal_conductivity()],
            None,
        );
        add(
            "si:joule-per-kilogram-kelvin",
            "J/(kg*K)",
            Dimension::LENGTH
                .powi(2)
                .quotient(Dimension::TIME.powi(2))
                .quotient(Dimension::TEMPERATURE),
            ExactScale::ONE,
            None,
            vec![QuantityKindId::specific_heat_capacity()],
            None,
        );
        out
    }
}

pub fn kinds_compatible(
    a: &QuantityKindId,
    b: &QuantityKindId,
    dimension: Dimension,
    strictness: KindStrictness,
) -> bool {
    match strictness {
        KindStrictness::DimensionOnly => true,
        KindStrictness::ExactKind => a == b,
        KindStrictness::KindCompatible => {
            if a == b {
                return true;
            }
            // Pressure and stress are intentionally compatible scalar normal-traction kinds,
            // while dimension equality alone must not make energy and torque interchangeable.
            let pressure_stress = (a == &QuantityKindId::pressure()
                && b == &QuantityKindId::stress())
                || (b == &QuantityKindId::pressure() && a == &QuantityKindId::stress());
            pressure_stress
                && dimension
                    == Dimension::MASS
                        .quotient(Dimension::LENGTH)
                        .quotient(Dimension::TIME.powi(2))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_and_interval_celsius_are_distinct() {
        let reg = UnitRegistry::standard();
        let absolute = reg
            .canonicalize(&QuantityLiteral {
                value: 25.0,
                unit: UnitId::new("si:degree-celsius"),
                quantity_kind: QuantityKindId::thermodynamic_temperature(),
            })
            .unwrap();
        let interval = reg
            .canonicalize(&QuantityLiteral {
                value: 10.0,
                unit: UnitId::new("si:degree-celsius"),
                quantity_kind: QuantityKindId::temperature_difference(),
            })
            .unwrap();
        assert!((absolute.value_si - 298.15).abs() < 1e-12);
        assert!((interval.value_si - 10.0).abs() < 1e-12);
    }

    #[test]
    fn same_dimension_does_not_imply_same_kind() {
        let reg = UnitRegistry::standard();
        let j = reg.by_symbol("J").unwrap();
        let torque = reg.by_symbol("N*m").unwrap();
        assert_eq!(j.dimension, torque.dimension);
        assert_ne!(
            j.quantity_kind_constraints,
            torque.quantity_kind_constraints
        );
        assert!(!kinds_compatible(
            &QuantityKindId::energy(),
            &QuantityKindId::moment_of_force(),
            j.dimension,
            KindStrictness::KindCompatible
        ));
    }
}
