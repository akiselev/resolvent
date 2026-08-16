namespace Resolvent

/-- Stable semantic handles exported to the Rust side. -/
structure ExprId where
  raw : Nat
  deriving Repr, DecidableEq

structure ObservableId where
  raw : Nat
  deriving Repr, DecidableEq

/-- Applicability is part of the claim rather than prose metadata. -/
structure Scope where
  label : String
  parameterRegion : Option String := none
  spatialDomain : Option String := none
  temporalDomain : Option String := none
  regularity : Array String := #[]
  conventions : Array String := #[]
  restrictions : Array String := #[]
  deriving Repr, DecidableEq

structure Parameter where
  name : String
  value : ExprId
  unit : Option String := none
  deriving Repr, DecidableEq

structure StateVariable where
  name : String
  symbol : ExprId
  unit : Option String := none
  domain : Option String := none
  deriving Repr, DecidableEq

structure Law where
  name : String
  residual : ExprId
  source : Option String := none
  deriving Repr, DecidableEq

structure Observable where
  id : ObservableId
  name : String
  expression : ExprId
  unit : Option String := none
  measurementModel : Option ExprId := none
  uncertaintyModel : Option ExprId := none
  source : Option String := none
  deriving Repr, DecidableEq

inductive ValidationContract where
  | invariant (name : String) (predicate : ExprId)
  | conservation (name : String) (balance : ExprId)
  | bound (name : String) (predicate : ExprId)
  | symmetry (name : String) (relation : ExprId)
  | convergence (name : String) (expectedOrder : Option String := none)
  | referenceCase (name : String) (predicate : ExprId)
  | observableAgreement (observable : ObservableId) (metric : String)
  deriving Repr, DecidableEq

/--
The deep, serializable scientific specification shared with the Rust compiler.

This is deliberately not a replacement for native Lean mathematics. A domain package should
construct a `ScientificSpec` and prove that its denotation agrees with the native statement it
intends to export. The proof, not the exporter implementation, is the semantic authority.
-/
structure ScientificSpec where
  name : String
  scope : Scope
  assumptions : Array String := #[]
  parameters : Array Parameter := #[]
  state : Array StateVariable := #[]
  laws : Array Law := #[]
  observables : Array Observable := #[]
  validationContracts : Array ValidationContract := #[]
  formalSource : Option String := none
  deriving Repr, DecidableEq

/-- Semantic relationship asserted between two compilation artifacts. -/
inductive RefinementRelation where
  | definitionallyEqual
  | mathematicallyEquivalent
  | logicalConsequence
  | specialization
  | reformulation
  | strongToWeakForm
  | indexReduced
  | discretization (method : String) (consistency convergence : Option String := none)
  | approximation (metric : String) (bound : Option String := none)
  | finitePrecisionImplementation (arithmetic : String) (errorModel : Option String := none)
  | compiledImplementation (backend : String)
  | observableInterpretation
  deriving Repr, DecidableEq

/--
A minimal denotation hook. Domain bridges should instantiate richer typed semantics rather than
encoding mathematics as strings; this proposition-level hook keeps the first export contract
small while allowing soundness theorems to be stated now.
-/
class Denotes (α : Type) where
  denote : α → Prop

end Resolvent
