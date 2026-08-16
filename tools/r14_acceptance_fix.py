from pathlib import Path

p = Path("src/scientific.rs")
s = p.read_text()

s = s.replace(
    "pub struct IntegralDecl {\n    pub measure: MeasureV1,\n    pub integrand: Expr,\n}",
    "pub struct IntegralDecl {\n    pub measure: MeasureV1,\n    pub integrand: Expr,\n    pub span: SourceSpan,\n}",
)
s = s.replace(
    "            let measure_name = self.expect_ident_value()?.0;",
    "            let measure_span = self.token().span;\n            let measure_name = self.expect_ident_value()?.0;",
)
s = s.replace(
    "            integrals.push(IntegralDecl { measure, integrand });",
    "            integrals.push(IntegralDecl { measure, integrand, span: measure_span });",
)

old = '''pub fn semantic_digest(module: &ScientificModule) -> String {
    let bytes = serde_json::to_vec(module).expect("scientific module serialization is infallible");
    blake3::hash(&bytes).to_hex().to_string()
}
'''
new = '''pub fn semantic_digest(module: &ScientificModule) -> String {
    // Spans are provenance, not scientific meaning. Strip them before hashing so
    // whitespace/comments/formatting do not perturb the physics identity.
    let mut value = serde_json::to_value(module)
        .expect("scientific module serialization is infallible");
    fn strip_spans(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("span");
                for child in map.values_mut() { strip_spans(child); }
            }
            serde_json::Value::Array(items) => {
                for child in items { strip_spans(child); }
            }
            _ => {}
        }
    }
    strip_spans(&mut value);
    let bytes = serde_json::to_vec(&value)
        .expect("semantic projection serialization is infallible");
    blake3::hash(&bytes).to_hex().to_string()
}
'''
if old not in s:
    raise SystemExit("semantic_digest target not found")
s = s.replace(old, new)

# Preserve unit declarations in the canonical formatter.
old = '''fn format_value_decl(kind: &str, d: &ValueDecl) -> String {
    let ty = d
        .quantity_kind
        .as_ref()
        .map(|x| format!(": {}", x.0))
        .unwrap_or_default();
    let value = d
        .value
        .as_ref()
        .map(|x| format!(" = {}", format_expr(x)))
        .unwrap_or_default();
    format!("    {kind} {}{ty}{value};\\n", d.name)
}'''
new = '''fn format_value_decl(kind: &str, d: &ValueDecl) -> String {
    let ty = d
        .quantity_kind
        .as_ref()
        .map(|x| format!(": {}", x.0))
        .unwrap_or_default();
    let unit = d
        .unit
        .as_ref()
        .map(|x| format!(" [{}]", x.0))
        .unwrap_or_default();
    let value = d
        .value
        .as_ref()
        .map(|x| format!(" = {}", format_expr(x)))
        .unwrap_or_default();
    format!("    {kind} {}{ty}{unit}{value};\\n", d.name)
}'''
if old not in s:
    raise SystemExit("format_value_decl target not found")
s = s.replace(old, new)

# Inject the declarations that the original formatter dropped. This is done at
# stable anchors so re-running the migration is idempotent.
needle = '''        for equation in &model.equations {'''
insert = '''        for property in &model.properties {
            out.push_str(&format!("    property {} = {};\\n", property.name, format_expr(&property.value)));
        }
        for law in &model.constitutive_laws {
            out.push_str(&format!("    constitutive {} = {};\\n", law.name, format_expr(&law.law)));
        }
        for equation in &model.equations {'''
if needle in s and "for law in &model.constitutive_laws" not in s[s.index("pub fn format_scientific_module"):s.index("fn coordinate_name")]:
    s = s.replace(needle, insert, 1)

needle = '''        for o in &model.observables {
            out.push_str(&format!(
                "    observable {} {{ {}; }}\\n",
                o.name,
                format_expr(&o.value)
            ));
        }
        out.push_str("}\\n\\n");'''
insert = '''        for form in &model.forms {
            out.push_str(&format!("    form {} {{\\n", form.name));
            for integral in &form.integrals {
                let (measure, target) = match &integral.measure {
                    MeasureV1::Cell(target) => ("cell", target),
                    MeasureV1::Boundary(target) => ("boundary", target),
                    MeasureV1::InteriorFacet(target) => ("interior_facet", target),
                };
                out.push_str(&format!("        {measure}({target}): {};\\n", format_expr(&integral.integrand)));
            }
            out.push_str("    }\\n");
        }
        if !model.initial_conditions.is_empty() {
            out.push_str("    initial {\\n");
            for c in &model.initial_conditions {
                out.push_str(&format!("        {} = {};\\n", c.target, format_expr(&c.value)));
            }
            out.push_str("    }\\n");
        }
        for bc in &model.boundary_conditions {
            let kind = match bc.kind {
                BoundaryConditionKind::Dirichlet => "dirichlet",
                BoundaryConditionKind::Neumann => "neumann",
                BoundaryConditionKind::Robin => "robin",
                BoundaryConditionKind::Interface => "interface",
            };
            out.push_str(&format!("    boundary {} on {} {{\\n        {kind} {} = {};\\n    }}\\n", bc.name, format_expr(&bc.region), bc.target, format_expr(&bc.value)));
        }
        for bc in &model.interface_conditions {
            out.push_str(&format!("    interface {} on {} {{\\n        interface {} = {};\\n    }}\\n", bc.name, format_expr(&bc.region), bc.target, format_expr(&bc.value)));
        }
        for o in &model.observables {
            out.push_str(&format!("    observable {} {{ {}; }}\\n", o.name, format_expr(&o.value)));
        }
        for i in &model.invariants {
            out.push_str(&format!("    invariant {} {{ {}; }}\\n", i.name, format_expr(&i.value)));
        }
        for v in &model.verifications {
            out.push_str(&format!("    @{}", v.name));
            if !v.args.is_empty() {
                out.push('(');
                out.push_str(&v.args.iter().map(|(k, x)| format!("{k} = {}", format_expr(x))).collect::<Vec<_>>().join(", "));
                out.push(')');
            }
            out.push_str(";\\n");
        }
        out.push_str("}\\n\\n");'''
if needle not in s:
    raise SystemExit("formatter tail target not found")
s = s.replace(needle, insert, 1)

p.write_text(s)
