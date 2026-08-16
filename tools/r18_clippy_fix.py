from pathlib import Path
p=Path('src/scientific.rs')
s=p.read_text()
old='''    fn trace(
        symbol: &str,
        residual: &str,
        field_names: &BTreeSet<String>,
        property_map: &BTreeMap<String, Expr>,
        constitutive_map: &BTreeMap<String, Expr>,
        path: &mut Vec<String>,
        seen: &mut BTreeSet<String>,
        reason: Option<CouplingReason>,
        out: &mut Vec<CouplingEdge>,
    ) {'''
new='''    struct TraceContext<'a> {
        field_names: &'a BTreeSet<String>,
        property_map: &'a BTreeMap<String, Expr>,
        constitutive_map: &'a BTreeMap<String, Expr>,
    }
    fn trace(
        symbol: &str,
        residual: &str,
        context: &TraceContext<'_>,
        path: &mut Vec<String>,
        seen: &mut BTreeSet<String>,
        reason: Option<CouplingReason>,
        out: &mut Vec<CouplingEdge>,
    ) {'''
if old not in s:
    raise SystemExit('trace signature target not found')
s=s.replace(old,new,1)
s=s.replace('if field_names.contains(symbol) {','if context.field_names.contains(symbol) {',1)
s=s.replace('if let Some(expr) = property_map.get(symbol) {','if let Some(expr) = context.property_map.get(symbol) {',1)
s=s.replace('} else if let Some(expr) = constitutive_map.get(symbol) {','} else if let Some(expr) = context.constitutive_map.get(symbol) {',1)
s=s.replace('''                    field_names,
                    property_map,
                    constitutive_map,
                    path,''','''                    context,
                    path,''')
# second recursive branch may have been replaced by the global replacement above too.
context_decl='''    let context = TraceContext {
        field_names: &field_names,
        property_map: &property_map,
        constitutive_map: &constitutive_map,
    };
'''
anchor='''    let mut edges = vec![];
'''
if context_decl not in s:
    s=s.replace(anchor,anchor+context_decl,1)
s=s.replace('''                &field_names,
                &property_map,
                &constitutive_map,
                &mut Vec::new(),''','''                &context,
                &mut Vec::new(),''')
p.write_text(s)
