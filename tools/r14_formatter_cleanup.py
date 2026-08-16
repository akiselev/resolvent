from pathlib import Path
p=Path('src/scientific.rs')
s=p.read_text()
s=s.replace('''            if f.quantity_kind.is_some() || f.unit.is_some() || f.nominal.is_some() {''','''            if f.quantity_kind.is_some()
                || f.unit.is_some()
                || f.nominal.is_some()
                || f.physical_min.is_some()
                || f.physical_max.is_some()
                || f.time_role.is_some()
            {''')
s=s.replace('''                if let Some(n) = &f.nominal {
                    out.push_str(&format!("        nominal = {} {};\\n", n.value, n.unit.0));
                }
                out.push_str("    }");''','''                if let Some(n) = &f.nominal {
                    out.push_str(&format!("        nominal = {} {};\\n", n.value, n.unit.0));
                }
                if let Some(n) = &f.physical_min {
                    out.push_str(&format!("        min = {} {};\\n", n.value, n.unit.0));
                }
                if let Some(n) = &f.physical_max {
                    out.push_str(&format!("        max = {} {};\\n", n.value, n.unit.0));
                }
                if let Some(role) = f.time_role {
                    let role = match role {
                        TimeRole::Differential => "differential",
                        TimeRole::Algebraic => "algebraic",
                    };
                    out.push_str(&format!("        time_role = {role};\\n"));
                }
                out.push_str("    }");''')
old='''        for c in &model.initial_conditions {
            out.push_str(&format!(
                "    initial {{ {} = {}; }}\\n",
                c.target,
                format_expr(&c.value)
            ));
        }
'''
if old not in s:
    raise SystemExit('duplicate initial formatter block not found')
s=s.replace(old,'',1)
p.write_text(s)
