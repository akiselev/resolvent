//! Deterministic reference execution for FC4 indexed QFunctions and element factorizations.

use crate::requirements::{GeometryPreprocessingRequirement, InputSourceRequirement};
use crate::semantic::{SemanticMeasure, SymbolId};
use crate::tensor::{
    OperatorFactorization, QFunctionProgram, TensorAxisId, TensorBinaryOp, TensorBinding,
    TensorInputRole, TensorReductionOp, TensorScalarExpr, TensorUnaryOp,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DenseTensor {
    pub shape: Vec<usize>,
    pub data: Vec<f64>,
}

impl DenseTensor {
    pub fn new(shape: Vec<usize>, data: Vec<f64>) -> Result<Self, TensorInterpretError> {
        let expected = element_count(&shape)?;
        if expected != data.len() {
            return Err(TensorInterpretError::Shape(format!(
                "shape {shape:?} contains {expected} values, got {}",
                data.len()
            )));
        }
        Ok(Self { shape, data })
    }

    pub fn scalar(value: f64) -> Self {
        Self {
            shape: Vec::new(),
            data: vec![value],
        }
    }

    fn value(&self, indices: &[usize]) -> Result<f64, TensorInterpretError> {
        if indices.len() != self.shape.len() {
            return Err(TensorInterpretError::Shape(format!(
                "tensor rank {} indexed with rank {}",
                self.shape.len(),
                indices.len()
            )));
        }
        let mut offset = 0usize;
        for (index, extent) in indices.iter().zip(&self.shape) {
            if index >= extent {
                return Err(TensorInterpretError::IndexOutOfBounds {
                    index: *index,
                    extent: *extent,
                });
            }
            offset = offset * extent + index;
        }
        Ok(self.data[offset])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorAction {
    Residual,
    Jvp,
}

/// Caller-owned element data. Basis tables have shape `[quadrature, dof, evaluation...]` and
/// point values have shape `[quadrature, evaluation...]` (or just `evaluation...` for constants).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementExecutionContext {
    pub element_dofs: BTreeMap<SymbolId, DenseTensor>,
    pub direction_dofs: BTreeMap<SymbolId, DenseTensor>,
    pub basis: BTreeMap<TensorBinding, DenseTensor>,
    pub point_values: BTreeMap<TensorBinding, DenseTensor>,
    pub geometry: BTreeMap<GeometryPreprocessingRequirement, DenseTensor>,
    pub quadrature_weights: Vec<f64>,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum TensorInterpretError {
    #[error("TENSOR_INTERPRET_SHAPE: {0}")]
    Shape(String),
    #[error("TENSOR_INTERPRET_INDEX: index {index} is outside extent {extent}")]
    IndexOutOfBounds { index: usize, extent: usize },
    #[error("TENSOR_INTERPRET_INPUT: QFunction input {0} is not bound")]
    MissingInput(usize),
    #[error("TENSOR_INTERPRET_AXIS: tensor axis {0} is not bound")]
    MissingAxis(u32),
    #[error("TENSOR_INTERPRET_DOF: no element values were supplied for symbol {0}")]
    MissingDofs(SymbolId),
    #[error("TENSOR_INTERPRET_DIRECTION: no direction values were supplied for symbol {0}")]
    MissingDirection(SymbolId),
    #[error("TENSOR_INTERPRET_BASIS: no basis table was supplied for {0:?}")]
    MissingBasis(TensorBinding),
    #[error("TENSOR_INTERPRET_POINT: no point values were supplied for {0:?}")]
    MissingPointValue(TensorBinding),
    #[error("TENSOR_INTERPRET_ACTIVE_SOURCE: active input {0:?} is not basis-backed")]
    NonBasisActiveInput(TensorBinding),
    #[error("TENSOR_INTERPRET_GEOMETRY: no geometry values were supplied for {0:?}")]
    MissingGeometry(GeometryPreprocessingRequirement),
    #[error("TENSOR_INTERPRET_NUMERIC: operation {0} is undefined at the supplied values")]
    Numeric(&'static str),
}

/// Interpret one point QFunction using dense point-local inputs in program order.
pub fn interpret_qfunction(
    program: &QFunctionProgram,
    inputs: &[DenseTensor],
) -> Result<Vec<DenseTensor>, TensorInterpretError> {
    if inputs.len() != program.inputs.len() {
        return Err(TensorInterpretError::Shape(format!(
            "program expects {} inputs, got {}",
            program.inputs.len(),
            inputs.len()
        )));
    }
    for (declaration, value) in program.inputs.iter().zip(inputs) {
        if declaration.shape != value.shape {
            return Err(TensorInterpretError::Shape(format!(
                "input {:?} expects shape {:?}, got {:?}",
                declaration.binding, declaration.shape, value.shape
            )));
        }
    }
    let interpreter = PointInterpreter { inputs };
    program
        .outputs
        .iter()
        .map(|output| {
            let mut axes = BTreeMap::new();
            let mut data = Vec::with_capacity(element_count(&output.shape)?);
            visit_indices(&output.shape, &mut Vec::new(), &mut |index| {
                for (axis, value) in output.free_axes.iter().zip(index) {
                    axes.insert(axis.id, *value);
                }
                data.push(interpreter.expression(&output.expression, &mut axes)?);
                Ok(())
            })?;
            DenseTensor::new(output.shape.clone(), data)
        })
        .collect()
}

/// Execute the explicit gather/basis/QFunction/weighted-transpose/scatter element semantics.
/// Mesh traversal and global assembly are intentionally absent; FC6 will bind those stages.
pub fn interpret_element_operator(
    factorization: &OperatorFactorization,
    action: OperatorAction,
    context: &ElementExecutionContext,
) -> Result<BTreeMap<SymbolId, DenseTensor>, TensorInterpretError> {
    let quadrature_points = context.quadrature_weights.len();
    let mut residuals = BTreeMap::<SymbolId, DenseTensor>::new();
    for integral in &factorization.integrals {
        let program = match action {
            OperatorAction::Residual => &integral.primal,
            OperatorAction::Jvp => &integral.jvp,
        };
        for quadrature in 0..quadrature_points {
            let inputs = program
                .inputs
                .iter()
                .map(|input| point_input(input, quadrature, quadrature_points, context))
                .collect::<Result<Vec<_>, _>>()?;
            let outputs = interpret_qfunction(program, &inputs)?;
            let weight = context.quadrature_weights[quadrature]
                * measure_scale(&integral.measure, quadrature, quadrature_points, context)?;
            for (output, value) in program.outputs.iter().zip(outputs) {
                let basis = context
                    .basis
                    .get(&output.binding)
                    .ok_or_else(|| TensorInterpretError::MissingBasis(output.binding.clone()))?;
                validate_basis_shape(basis, quadrature_points, &output.shape, &output.binding)?;
                let dofs = basis.shape[1];
                let residual =
                    residuals
                        .entry(output.binding.symbol)
                        .or_insert_with(|| DenseTensor {
                            shape: vec![dofs],
                            data: vec![0.0; dofs],
                        });
                if residual.shape != [dofs] {
                    return Err(TensorInterpretError::Shape(format!(
                        "test symbol {} has inconsistent element dof counts",
                        output.binding.symbol
                    )));
                }
                for dof in 0..dofs {
                    let mut contribution = 0.0;
                    visit_indices(&output.shape, &mut Vec::new(), &mut |component| {
                        let mut basis_index = vec![quadrature, dof];
                        basis_index.extend_from_slice(component);
                        contribution += basis.value(&basis_index)? * value.value(component)?;
                        Ok(())
                    })?;
                    residual.data[dof] += weight * contribution;
                }
            }
        }
    }
    Ok(residuals)
}

fn point_input(
    input: &crate::tensor::QFunctionInput,
    quadrature: usize,
    quadrature_points: usize,
    context: &ElementExecutionContext,
) -> Result<DenseTensor, TensorInterpretError> {
    if matches!(
        input.role,
        TensorInputRole::Active | TensorInputRole::Direction { .. }
    ) && input.source != InputSourceRequirement::Basis
    {
        return Err(TensorInterpretError::NonBasisActiveInput(
            input.binding.clone(),
        ));
    }
    if input.source == InputSourceRequirement::Basis {
        let dofs = match input.role {
            TensorInputRole::Direction { .. } => context
                .direction_dofs
                .get(&input.binding.symbol)
                .ok_or(TensorInterpretError::MissingDirection(input.binding.symbol))?,
            _ => context
                .element_dofs
                .get(&input.binding.symbol)
                .ok_or(TensorInterpretError::MissingDofs(input.binding.symbol))?,
        };
        let basis = context
            .basis
            .get(&input.binding)
            .ok_or_else(|| TensorInterpretError::MissingBasis(input.binding.clone()))?;
        basis_forward(
            basis,
            dofs,
            quadrature,
            quadrature_points,
            &input.shape,
            &input.binding,
        )
    } else {
        let values = context
            .point_values
            .get(&input.binding)
            .ok_or_else(|| TensorInterpretError::MissingPointValue(input.binding.clone()))?;
        point_slice(values, quadrature, quadrature_points, &input.shape)
    }
}

fn basis_forward(
    basis: &DenseTensor,
    dofs: &DenseTensor,
    quadrature: usize,
    quadrature_points: usize,
    output_shape: &[usize],
    binding: &TensorBinding,
) -> Result<DenseTensor, TensorInterpretError> {
    validate_basis_shape(basis, quadrature_points, output_shape, binding)?;
    let dof_count = basis.shape[1];
    if dofs.shape != [dof_count] {
        return Err(TensorInterpretError::Shape(format!(
            "symbol {} expects {dof_count} dofs, got shape {:?}",
            binding.symbol, dofs.shape
        )));
    }
    let mut data = Vec::with_capacity(element_count(output_shape)?);
    visit_indices(output_shape, &mut Vec::new(), &mut |component| {
        let mut value = 0.0;
        for dof in 0..dof_count {
            let mut index = vec![quadrature, dof];
            index.extend_from_slice(component);
            value += basis.value(&index)? * dofs.data[dof];
        }
        data.push(value);
        Ok(())
    })?;
    DenseTensor::new(output_shape.to_vec(), data)
}

fn validate_basis_shape(
    basis: &DenseTensor,
    quadrature_points: usize,
    value_shape: &[usize],
    binding: &TensorBinding,
) -> Result<(), TensorInterpretError> {
    let mut expected_tail = value_shape.to_vec();
    if basis.shape.len() != value_shape.len() + 2
        || basis.shape.first() != Some(&quadrature_points)
        || basis.shape[2..] != expected_tail
    {
        expected_tail.insert(0, 0);
        expected_tail.insert(0, quadrature_points);
        return Err(TensorInterpretError::Shape(format!(
            "basis {:?} expects shape [quadrature, dof, {:?}], got {:?}",
            binding, value_shape, basis.shape
        )));
    }
    Ok(())
}

fn point_slice(
    values: &DenseTensor,
    quadrature: usize,
    quadrature_points: usize,
    value_shape: &[usize],
) -> Result<DenseTensor, TensorInterpretError> {
    if values.shape == value_shape {
        return Ok(values.clone());
    }
    let mut expected = vec![quadrature_points];
    expected.extend_from_slice(value_shape);
    if values.shape != expected {
        return Err(TensorInterpretError::Shape(format!(
            "point values expect shape {value_shape:?} or {expected:?}, got {:?}",
            values.shape
        )));
    }
    let stride = element_count(value_shape)?;
    let start = quadrature * stride;
    DenseTensor::new(
        value_shape.to_vec(),
        values.data[start..start + stride].to_vec(),
    )
}

fn measure_scale(
    measure: &SemanticMeasure,
    quadrature: usize,
    quadrature_points: usize,
    context: &ElementExecutionContext,
) -> Result<f64, TensorInterpretError> {
    let requirement = match measure {
        SemanticMeasure::Cell { .. } => Some(GeometryPreprocessingRequirement::JacobianDeterminant),
        SemanticMeasure::ExteriorFacet { .. }
        | SemanticMeasure::InteriorFacet { .. }
        | SemanticMeasure::Interface { .. } => {
            Some(GeometryPreprocessingRequirement::FacetJacobian)
        }
        SemanticMeasure::Point { .. } => None,
    };
    let Some(requirement) = requirement else {
        return Ok(1.0);
    };
    let values = context
        .geometry
        .get(&requirement)
        .ok_or(TensorInterpretError::MissingGeometry(requirement))?;
    if values.shape.is_empty() {
        Ok(values.data[0])
    } else if values.shape == [quadrature_points] {
        Ok(values.data[quadrature])
    } else {
        Err(TensorInterpretError::Shape(format!(
            "geometry {requirement:?} expects scalar or [{quadrature_points}], got {:?}",
            values.shape
        )))
    }
}

struct PointInterpreter<'a> {
    inputs: &'a [DenseTensor],
}

impl PointInterpreter<'_> {
    fn expression(
        &self,
        expression: &TensorScalarExpr,
        axes: &mut BTreeMap<TensorAxisId, usize>,
    ) -> Result<f64, TensorInterpretError> {
        Ok(match expression {
            TensorScalarExpr::Constant { value } => *value,
            TensorScalarExpr::Input { input, indices } => {
                let value = self
                    .inputs
                    .get(input.index())
                    .ok_or(TensorInterpretError::MissingInput(input.index()))?;
                let indices = indices
                    .iter()
                    .map(|axis| {
                        axes.get(axis)
                            .copied()
                            .ok_or(TensorInterpretError::MissingAxis(axis.0))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                value.value(&indices)?
            }
            TensorScalarExpr::Unary { op, arg } => {
                let value = self.expression(arg, axes)?;
                match op {
                    TensorUnaryOp::Neg => -value,
                    TensorUnaryOp::Abs => value.abs(),
                    TensorUnaryOp::Sqrt => value.sqrt(),
                    TensorUnaryOp::Exp => value.exp(),
                    TensorUnaryOp::Ln => value.ln(),
                    TensorUnaryOp::Sin => value.sin(),
                    TensorUnaryOp::Cos => value.cos(),
                    TensorUnaryOp::Tan => value.tan(),
                    TensorUnaryOp::Floor => value.floor(),
                    TensorUnaryOp::Ceil => value.ceil(),
                }
            }
            TensorScalarExpr::Binary { op, lhs, rhs } => {
                let lhs = self.expression(lhs, axes)?;
                let rhs = self.expression(rhs, axes)?;
                match op {
                    TensorBinaryOp::Add => lhs + rhs,
                    TensorBinaryOp::Sub => lhs - rhs,
                    TensorBinaryOp::Mul => lhs * rhs,
                    TensorBinaryOp::Div => lhs / rhs,
                    TensorBinaryOp::Pow => lhs.powf(rhs),
                    TensorBinaryOp::Min => lhs.min(rhs),
                    TensorBinaryOp::Max => lhs.max(rhs),
                    TensorBinaryOp::Atan2 => lhs.atan2(rhs),
                }
            }
            TensorScalarExpr::IndexEqual { lhs, rhs } => {
                let lhs = axes
                    .get(lhs)
                    .ok_or(TensorInterpretError::MissingAxis(lhs.0))?;
                let rhs = axes
                    .get(rhs)
                    .ok_or(TensorInterpretError::MissingAxis(rhs.0))?;
                if lhs == rhs { 1.0 } else { 0.0 }
            }
            TensorScalarExpr::Reduction {
                op: TensorReductionOp::Sum,
                axis,
                expression,
            } => {
                let previous = axes.get(&axis.id).copied();
                let mut total = 0.0;
                for index in 0..axis.extent {
                    axes.insert(axis.id, index);
                    total += self.expression(expression, axes)?;
                }
                if let Some(previous) = previous {
                    axes.insert(axis.id, previous);
                } else {
                    axes.remove(&axis.id);
                }
                total
            }
        })
    }
}

fn element_count(shape: &[usize]) -> Result<usize, TensorInterpretError> {
    shape.iter().try_fold(1usize, |size, extent| {
        size.checked_mul(*extent)
            .ok_or_else(|| TensorInterpretError::Shape("tensor extent overflow".into()))
    })
}

fn visit_indices(
    shape: &[usize],
    index: &mut Vec<usize>,
    visitor: &mut impl FnMut(&[usize]) -> Result<(), TensorInterpretError>,
) -> Result<(), TensorInterpretError> {
    if index.len() == shape.len() {
        return visitor(index);
    }
    let extent = shape[index.len()];
    for value in 0..extent {
        index.push(value);
        visitor_indices_tail(shape, index, visitor)?;
        index.pop();
    }
    Ok(())
}

fn visitor_indices_tail(
    shape: &[usize],
    index: &mut Vec<usize>,
    visitor: &mut impl FnMut(&[usize]) -> Result<(), TensorInterpretError>,
) -> Result<(), TensorInterpretError> {
    if index.len() == shape.len() {
        visitor(index)
    } else {
        let extent = shape[index.len()];
        for value in 0..extent {
            index.push(value);
            visitor_indices_tail(shape, index, visitor)?;
            index.pop();
        }
        Ok(())
    }
}
