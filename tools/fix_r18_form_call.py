from pathlib import Path
p=Path('src/scientific.rs')
s=p.read_text()
old='''                trace(
                    &name,
                    &form.name,
                    &field_names,
                    &property_map,
                    &constitutive_map,
                    &mut Vec::new(),
                    &mut BTreeSet::new(),
                    None,
                    &mut edges,
                );'''
new='''                trace(
                    &name,
                    &form.name,
                    &context,
                    &mut Vec::new(),
                    &mut BTreeSet::new(),
                    None,
                    &mut edges,
                );'''
if old not in s:
    raise SystemExit('remaining form trace call not found')
p.write_text(s.replace(old,new,1))
