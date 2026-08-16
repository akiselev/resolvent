from pathlib import Path
p = Path('src/scientific.rs')
s = p.read_text()
old = '''        self.expect_punct('{');
        let lhs = self.expr(0)?;
        if !self.eat_op("=") {
            self.error("equation requires `=`".into());
        }
        let rhs = self.expr(0)?;'''
new = '''        self.expect_punct('{');
        // Top-level equality belongs to the equation declaration, not the expression tree.
        // Parse above equality precedence so comparisons remain legal inside each side.
        let lhs = self.expr(2)?;
        if !self.eat_op("=") {
            self.error("equation requires `=`".into());
        }
        let rhs = self.expr(0)?;'''
if old not in s:
    raise SystemExit('equation parser target not found')
p.write_text(s.replace(old, new, 1))
