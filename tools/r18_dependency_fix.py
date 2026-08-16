from pathlib import Path
p=Path('src/scientific.rs')
s=p.read_text()

old='''    fn strip_spans(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("span");
                for child in map.values_mut() {
                    strip_spans(child);
                }
            }
            serde_json::Value::Array(items) => {
                for child in items {
                    strip_spans(child);
                }
            }
            _ => {}
        }
    }
    strip_spans(&mut value);'''
new='''    fn canonicalize(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("span");
                for child in map.values_mut() { canonicalize(child); }
            }
            serde_json::Value::Array(items) => {
                for child in items.iter_mut() { canonicalize(child); }
                if items.iter().all(|item| item.get("name").and_then(|x| x.as_str()).is_some()) {
                    items.sort_by(|a, b| {
                        a.get("name").and_then(|x| x.as_str()).cmp(&b.get("name").and_then(|x| x.as_str()))
                    });
                } else if items.iter().all(|item| item.as_str().is_some()) {
                    items.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
                }
            }
            _ => {}
        }
    }
    canonicalize(&mut value);'''
if old not in s:
    raise SystemExit('semantic canonicalizer target not found')
s=s.replace(old,new,1)

start=s.index('pub fn derive_coupling_graph(model: &ScientificModel) -> CouplingGraph {')
end=s.index('// ---------------- R13 execution staging / canonical heat ----------------', start)
new_fn=r'''pub fn derive_coupling_graph(model: &ScientificModel) -> CouplingGraph {
    let field_names: BTreeSet<_> = model
        .fields
        .iter()
        .filter(|f| matches!(f.role, FieldRoleV1::State | FieldRoleV1::Unknown))
        .map(|f| f.name.clone())
        .collect();
    let property_map: BTreeMap<_, _> = model
        .properties
        .iter()
        .map(|p| (p.name.clone(), p.value.clone()))
        .collect();
    let constitutive_map: BTreeMap<_, _> = model
        .constitutive_laws
        .iter()
        .map(|law| (law.name.clone(), law.law.clone()))
        .collect();

    fn trace(
        symbol: &str,
        residual: &str,
        field_names: &BTreeSet<String>,
        property_map: &BTreeMap<String, Expr>,
        constitutive_map: &BTreeMap<String, Expr>,
        path: &mut Vec<String>,
        seen: &mut BTreeSet<String>,
        reason: Option<CouplingReason>,
        out: &mut Vec<CouplingEdge>,
    ) {
        if field_names.contains(symbol) {
            let mut full = vec![symbol.to_string()];
            full.extend(path.iter().cloned());
            full.push(residual.to_string());
            out.push(CouplingEdge {
                from: symbol.to_string(),
                to: residual.to_string(),
                reason: reason.unwrap_or(CouplingReason::DirectFieldUse),
                path: full,
            });
            return;
        }
        if !seen.insert(symbol.to_string()) {
            return;
        }
        if let Some(expr) = property_map.get(symbol) {
            path.insert(0, symbol.to_string());
            let mut names = BTreeSet::new();
            expr.names(&mut names);
            for name in names {
                trace(
                    &name,
                    residual,
                    field_names,
                    property_map,
                    constitutive_map,
                    path,
                    seen,
                    Some(CouplingReason::PropertyDependency(symbol.to_string())),
                    out,
                );
            }
            path.remove(0);
        } else if let Some(expr) = constitutive_map.get(symbol) {
            path.insert(0, symbol.to_string());
            let mut names = BTreeSet::new();
            expr.names(&mut names);
            for name in names {
                trace(
                    &name,
                    residual,
                    field_names,
                    property_map,
                    constitutive_map,
                    path,
                    seen,
                    Some(CouplingReason::ConstitutiveDependency(symbol.to_string())),
                    out,
                );
            }
            path.remove(0);
        }
        seen.remove(symbol);
    }

    let unknowns = field_names
        .iter()
        .map(|f| UnknownBlock { name: f.clone(), field: f.clone() })
        .collect::<Vec<_>>();
    let mut residual_blocks = model.equations.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
    residual_blocks.extend(model.forms.iter().map(|f| f.name.clone()));
    residual_blocks.sort();
    residual_blocks.dedup();

    let mut edges = vec![];
    for equation in &model.equations {
        let mut names = BTreeSet::new();
        equation.lhs.names(&mut names);
        equation.rhs.names(&mut names);
        for name in names {
            trace(
                &name,
                &equation.name,
                &field_names,
                &property_map,
                &constitutive_map,
                &mut Vec::new(),
                &mut BTreeSet::new(),
                None,
                &mut edges,
            );
        }
    }
    for form in &model.forms {
        for integral in &form.integrals {
            let mut names = BTreeSet::new();
            integral.integrand.names(&mut names);
            for name in names {
                trace(
                    &name,
                    &form.name,
                    &field_names,
                    &property_map,
                    &constitutive_map,
                    &mut Vec::new(),
                    &mut BTreeSet::new(),
                    None,
                    &mut edges,
                );
            }
        }
    }
    // Conditions contribute to the residual block for their target field. Their region/value
    // dependencies are explicit interface/boundary coupling rather than invisible runtime state.
    for condition in model
        .boundary_conditions
        .iter()
        .chain(model.interface_conditions.iter())
    {
        let mut names = BTreeSet::new();
        condition.region.names(&mut names);
        condition.value.names(&mut names);
        for name in names {
            let before = edges.len();
            trace(
                &name,
                &condition.target,
                &field_names,
                &property_map,
                &constitutive_map,
                &mut Vec::new(),
                &mut BTreeSet::new(),
                Some(CouplingReason::InterfaceTerm),
                &mut edges,
            );
            for edge in &mut edges[before..] {
                edge.reason = CouplingReason::InterfaceTerm;
            }
        }
    }

    edges.sort_by(|a, b| (&a.to, &a.from, &a.path).cmp(&(&b.to, &b.from, &b.path)));
    edges.dedup_by(|a, b| a.from == b.from && a.to == b.to && a.path == b.path);
    let mut derivatives = Vec::new();
    for residual in &residual_blocks {
        for unknown in &field_names {
            derivatives.push(BlockDerivative {
                residual: residual.clone(),
                unknown: unknown.clone(),
                structurally_nonzero: edges.iter().any(|edge| &edge.to == residual && &edge.from == unknown),
            });
        }
    }
    CouplingGraph { unknowns, residual_blocks, edges, derivatives }
}

'''
s=s[:start]+new_fn+s[end:]
p.write_text(s)
