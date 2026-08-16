use resolvent::scientific::{
    derive_coupling_graph, execution_plan, format_scientific_module, parse_scientific_module,
    semantic_digest,
};
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(usage)?;
    let rest = args.collect::<Vec<_>>();
    let json = rest.iter().any(|arg| arg == "--json");
    let mut positional = rest.iter().filter(|arg| arg.as_str() != "--json");
    let file = positional.next().ok_or_else(usage)?;
    let selector = positional.next().map(String::as_str);
    let source = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let module = parse_scientific_module(&source).map_err(|errors| {
        errors
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    match command.as_str() {
        "check" => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "models": module.models.len(),
                        "semantic_digest": semantic_digest(&module),
                    }))
                    .map_err(|e| e.to_string())?
                );
            } else {
                println!(
                    "ok: {} model(s), digest {}",
                    module.models.len(),
                    semantic_digest(&module)
                );
            }
        }
        "parse" | "elaborate" => println!(
            "{}",
            serde_json::to_string_pretty(&module).map_err(|e| e.to_string())?
        ),
        "fmt" => print!("{}", format_scientific_module(&module)),
        "freeze" => println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema":"resolvent-scientific-lock/1",
                "module":module.name,
                "semantic_digest":semantic_digest(&module)
            }))
            .map_err(|e|e.to_string())?
        ),
        "inspect" => {
            let models = module
                .models
                .iter()
                .map(|model| {
                    serde_json::json!({
                        "name": model.name,
                        "domains": model.domains.len(),
                        "fields": model.fields,
                        "parameters": model.parameters,
                        "constants": model.constants,
                        "sources": model.sources,
                        "properties": model.properties,
                        "constitutive_laws": model.constitutive_laws,
                        "equations": model.equations,
                        "forms": model.forms,
                        "initial_conditions": model.initial_conditions,
                        "boundary_conditions": model.boundary_conditions,
                        "interface_conditions": model.interface_conditions,
                        "observables": model.observables,
                        "invariants": model.invariants,
                        "verifications": model.verifications,
                    })
                })
                .collect::<Vec<_>>();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema": "resolvent-scientific-inspect/1",
                    "module": module.name,
                    "semantic_digest": semantic_digest(&module),
                    "models": models,
                }))
                .map_err(|e| e.to_string())?
            );
        }
        "coupling" => {
            let model = module.models.first().ok_or("module has no model")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&derive_coupling_graph(model))
                    .map_err(|e| e.to_string())?
            );
        }
        "explain" => {
            let model = module.models.first().ok_or("module has no model")?;
            let graph = derive_coupling_graph(model);
            let edges = graph
                .edges
                .iter()
                .filter(|edge| {
                    selector.is_none_or(|name| edge.from == name || edge.to == name)
                })
                .collect::<Vec<_>>();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema": "resolvent-scientific-explain/1",
                    "selector": selector,
                    "edges": edges,
                }))
                .map_err(|e| e.to_string())?
            );
        }
        "plan" => {
            let model = module.models.first().ok_or("module has no model")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&execution_plan(model)).map_err(|e| e.to_string())?
            );
        }
        _ => return Err(usage()),
    }
    Ok(())
}

fn usage() -> String {
    "usage: resolvent-science <check|fmt|parse|elaborate|inspect|freeze|explain|coupling|plan> [--json] <model.res> [selector]".into()
}
