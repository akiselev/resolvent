use crate::discrete::DiscreteProgram;
use crate::expr::{ExprStore, Symbol, SymbolTable};
use crate::form::FormProgram;
use crate::id::{DiscreteProgramId, FieldId, FormId, OperatorId, RefinementId, SymbolId, SystemId};
use crate::model::System;
use crate::operator::OperatorProgram;
use crate::refinement::{ArtifactKind, ArtifactRef, RefinementError, RefinementRecord};
use serde::{Deserialize, Serialize};

/// One caller-owned semantic world. The stores are separate dialects but share stable
/// handles; no global interner/cache is needed to move a scientific model through the
/// compiler.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Context {
    pub symbols: SymbolTable,
    pub exprs: ExprStore,
    systems: Vec<System>,
    forms: Vec<FormProgram>,
    discrete: Vec<DiscreteProgram>,
    operators: Vec<OperatorProgram>,
    refinements: Vec<RefinementRecord>,
    next_field: u32,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rebuild_indexes(&mut self) {
        self.symbols.rebuild_index();
        self.exprs.rebuild_index();
    }

    pub fn declare_symbol(&mut self, symbol: Symbol) -> SymbolId {
        self.symbols.declare(symbol)
    }

    pub fn allocate_field_id(&mut self) -> FieldId {
        let id = FieldId(self.next_field);
        self.next_field += 1;
        id
    }

    pub fn insert_system(&mut self, system: System) -> SystemId {
        let id = SystemId(self.systems.len() as u32);
        self.systems.push(system);
        id
    }

    pub fn system(&self, id: SystemId) -> Option<&System> {
        self.systems.get(id.index())
    }

    pub fn insert_form(&mut self, form: FormProgram) -> FormId {
        let id = FormId(self.forms.len() as u32);
        self.forms.push(form);
        id
    }

    pub fn form(&self, id: FormId) -> Option<&FormProgram> {
        self.forms.get(id.index())
    }

    pub fn insert_discrete(&mut self, program: DiscreteProgram) -> DiscreteProgramId {
        let id = DiscreteProgramId(self.discrete.len() as u32);
        self.discrete.push(program);
        id
    }

    pub fn discrete(&self, id: DiscreteProgramId) -> Option<&DiscreteProgram> {
        self.discrete.get(id.index())
    }

    pub fn insert_operator(&mut self, operator: OperatorProgram) -> OperatorId {
        let id = OperatorId(self.operators.len() as u32);
        self.operators.push(operator);
        id
    }

    pub fn operator(&self, id: OperatorId) -> Option<&OperatorProgram> {
        self.operators.get(id.index())
    }

    pub fn record_refinement(&mut self, refinement: RefinementRecord) -> RefinementId {
        let id = RefinementId(self.refinements.len() as u32);
        self.refinements.push(refinement);
        id
    }

    pub fn refinement(&self, id: RefinementId) -> Option<&RefinementRecord> {
        self.refinements.get(id.index())
    }

    pub fn refinements(&self) -> &[RefinementRecord] {
        &self.refinements
    }

    /// Content-address a handle-bearing root together with the frozen semantic context that
    /// gives all of its ids meaning. Hashing a bare `ScientificSpec`, `System`, `Form`, or
    /// `OperatorProgram` is insufficient because their arena handles are only meaningful
    /// relative to this context.
    ///
    /// This first implementation hashes the complete context snapshot for correctness. A
    /// future subgraph-minimizing packager may make artifacts smaller, but it must preserve
    /// the same self-contained semantic property.
    pub fn rooted_artifact_ref<T: Serialize>(
        &self,
        kind: ArtifactKind,
        root: &T,
    ) -> Result<ArtifactRef, RefinementError> {
        #[derive(Serialize)]
        struct Rooted<'a, T> {
            schema_version: &'static str,
            context: &'a Context,
            root: &'a T,
        }

        ArtifactRef::of(
            kind,
            &Rooted {
                schema_version: crate::SCIENTIFIC_SCHEMA_VERSION,
                context: self,
                root,
            },
        )
    }
}
