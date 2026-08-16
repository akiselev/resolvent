use resolvent::author::{elaborate, parse_model};
use resolvent::freeze::SemanticLock;
use resolvent::generated_verify::check_transpose;
use resolvent::reference::{ScalarEllipticProblem2d, assemble_scalar_elliptic_p1};
use resolvent::semantic_check::check_model;
use resolvent::structural::dae::analyze_dae;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(usage)?;
    match command.as_str() {
        "check" => {
            let path = args.next().ok_or_else(usage)?;
            let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let parsed = parse_model(&source).map_err(render_author)?;
            let diagnostics = check_model(&parsed);
            if diagnostics.is_empty() {
                let _ = elaborate(&source).map_err(render_author)?;
                println!("{}",serde_json::to_string_pretty(&serde_json::json!({"ok":true,"model":parsed.name,"source_digest":parsed.source_digest})).unwrap());
                Ok(())
            } else {
                println!("{}", serde_json::to_string_pretty(&diagnostics).unwrap());
                Err(format!("{} semantic diagnostic(s)", diagnostics.len()))
            }
        }
        "inspect" => {
            let path = args.next().ok_or_else(usage)?;
            let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let parsed = parse_model(&source).map_err(render_author)?;
            println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
            Ok(())
        }
        "structural" => {
            let path = args.next().ok_or_else(usage)?;
            let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let model = elaborate(&source).map_err(render_author)?;
            let system = model
                .context
                .system(model.spec.model)
                .ok_or("missing model system")?;
            let analysis = analyze_dae(system, &model.context.exprs).map_err(|e| e.to_string())?;
            println!("{}", serde_json::to_string_pretty(&analysis).unwrap());
            Ok(())
        }
        "freeze" => {
            let path = args.next().ok_or_else(usage)?;
            let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let lock = SemanticLock::from_source(&source).map_err(|e| e.to_string())?;
            let json = serde_json::to_string_pretty(&lock).unwrap();
            if let Some(output) = args.next() {
                fs::write(output, json).map_err(|e| e.to_string())?;
            } else {
                println!("{json}");
            }
            Ok(())
        }
        "reference-scalar2d" => {
            let path = args.next().ok_or_else(usage)?;
            let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let problem: ScalarEllipticProblem2d =
                serde_json::from_str(&json).map_err(|e| e.to_string())?;
            let assembled = assemble_scalar_elliptic_p1(&problem).map_err(|e| e.to_string())?;
            let x = vec![1.0; assembled.stiffness_free.cols];
            let y = vec![0.5; assembled.stiffness_free.rows];
            let adjoint = check_transpose(&assembled.stiffness_free, &x, &y, 1e-12)
                .map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"operator":assembled,"transpose_check":adjoint})
                )
                .unwrap()
            );
            Ok(())
        }
        _ => Err(usage()),
    }
}
fn render_author(error: resolvent::author::AuthorError) -> String {
    if error.diagnostics().is_empty() {
        error.to_string()
    } else {
        serde_json::to_string_pretty(error.diagnostics()).unwrap_or_else(|_| error.to_string())
    }
}
fn usage() -> String {
    "usage: resolvent <check|inspect|structural|freeze|reference-scalar2d> <file> [output]".into()
}
