use resolvent::scientific::{derive_coupling_graph, execution_plan, format_scientific_module, parse_scientific_module, semantic_digest};
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    match run() { Ok(()) => ExitCode::SUCCESS, Err(e) => { eprintln!("{e}"); ExitCode::from(2) } }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(usage)?;
    let file = args.next().ok_or_else(usage)?;
    let source = fs::read_to_string(&file).map_err(|e| format!("{file}: {e}"))?;
    let module = parse_scientific_module(&source).map_err(|errors| {
        errors.into_iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    })?;
    match command.as_str() {
        "check" => {
            println!("ok: {} model(s), digest {}", module.models.len(), semantic_digest(&module));
        }
        "parse" | "elaborate" => println!("{}", serde_json::to_string_pretty(&module).map_err(|e| e.to_string())?),
        "fmt" => print!("{}", format_scientific_module(&module)),
        "freeze" => println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schema":"resolvent-scientific-lock/1","module":module.name,"semantic_digest":semantic_digest(&module)})).map_err(|e|e.to_string())?),
        "coupling" => {
            let model = module.models.first().ok_or("module has no model")?;
            println!("{}", serde_json::to_string_pretty(&derive_coupling_graph(model)).map_err(|e| e.to_string())?);
        }
        "plan" => {
            let model = module.models.first().ok_or("module has no model")?;
            println!("{}", serde_json::to_string_pretty(&execution_plan(model)).map_err(|e| e.to_string())?);
        }
        _ => return Err(usage()),
    }
    Ok(())
}

fn usage() -> String {
    "usage: resolvent-science <check|parse|elaborate|fmt|freeze|coupling|plan> <model.res>".into()
}
