use quantitas::UnitRegistry;
use resolvent::{
    IncidenceSystem, SourceDiagnostic, compile_schedule, compile_variational_form,
    derive_coupling_graph, derive_variational_form, elaborate_module, format_scientific_module,
    parse_scientific_module_diagnostics, semantic_arena_digest, semantic_digest,
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
    let module = parse_scientific_module_diagnostics(&source)
        .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
    match command.as_str() {
        "check" => {
            let semantic = elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "models": module.models.len(),
                        "semantic_digest": semantic_digest(&module),
                        "semantic_arena_digest": semantic_arena_digest(&semantic),
                        "expressions": semantic.models.iter().map(|model| model.expressions.len()).sum::<usize>(),
                    }))
                    .map_err(|e| e.to_string())?
                );
            } else {
                println!(
                    "ok: {} model(s), digest {}",
                    module.models.len(),
                    semantic_arena_digest(&semantic)
                );
            }
        }
        "parse" => println!(
            "{}",
            serde_json::to_string_pretty(&module).map_err(|e| e.to_string())?
        ),
        "fmt" => print!("{}", format_scientific_module(&module)),
        "freeze" => {
            let semantic = elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema":"resolvent-scientific-lock/1",
                    "module":module.name,
                    "source_digest":semantic_digest(&module),
                    "semantic_digest":semantic_arena_digest(&semantic)
                }))
                .map_err(|e| e.to_string())?
            );
        }
        "inspect" => {
            let semantic = elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
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
                    "semantic_arena_digest": semantic_arena_digest(&semantic),
                    "semantic_models": semantic.models,
                    "models": models,
                }))
                .map_err(|e| e.to_string())?
            );
        }
        "coupling" => {
            elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
            let model = module.models.first().ok_or("module has no model")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&derive_coupling_graph(model))
                    .map_err(|e| e.to_string())?
            );
        }
        "structural" => {
            elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
            let model = module.models.first().ok_or("module has no model")?;
            let incidence = IncidenceSystem::from_model(model).map_err(|e| e.to_string())?;
            let output = match compile_schedule(&incidence) {
                Ok(schedule) => serde_json::json!({
                    "incidence": incidence,
                    "schedule": schedule,
                }),
                Err(error) => serde_json::json!({
                    "incidence": incidence,
                    "schedule_error": error.to_string(),
                }),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
            );
        }
        "explain" => {
            elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
            let model = module.models.first().ok_or("module has no model")?;
            let graph = derive_coupling_graph(model);
            let edges = graph
                .edges
                .iter()
                .filter(|edge| selector.is_none_or(|name| edge.from == name || edge.to == name))
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
        "form" => {
            let semantic = elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
            let model = semantic.models.first().ok_or("module has no model")?;
            let form_name = selector.ok_or("form command requires a form name")?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &compile_variational_form(&semantic, &model.name, form_name)
                        .map_err(|e| e.to_string())?
                )
                .map_err(|e| e.to_string())?
            );
        }
        "derive-form" => {
            let semantic = elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
            let model = semantic.models.first().ok_or("module has no model")?;
            let equation_name = selector.ok_or("derive-form command requires an equation name")?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &derive_variational_form(&semantic, &model.name, equation_name)
                        .map_err(|e| e.to_string())?
                )
                .map_err(|e| e.to_string())?
            );
        }
        "elaborate" => {
            let semantic = elaborate_module(&module, &UnitRegistry::si_bootstrap())
                .map_err(|diagnostics| render_diagnostics(&source, &diagnostics, json))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&semantic).map_err(|error| error.to_string())?
            );
        }
        _ => return Err(usage()),
    }
    Ok(())
}

fn usage() -> String {
    "usage: resolvent <check|fmt|parse|elaborate|inspect|freeze|explain|coupling|structural|form|derive-form> [--json] <model.res> [selector]".into()
}

fn render_diagnostics(source: &str, diagnostics: &[SourceDiagnostic], json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(&serde_json::json!({
            "ok": false,
            "diagnostics": diagnostics,
        }))
        .unwrap_or_else(|error| error.to_string());
    }
    diagnostics
        .iter()
        .map(|diagnostic| {
            let (line, column) = line_column(source, diagnostic.span.start);
            format!(
                "{}:{}: {} [{}] {}",
                line,
                column,
                match diagnostic.severity {
                    resolvent::SourceSeverity::Note => "note",
                    resolvent::SourceSeverity::Warning => "warning",
                    resolvent::SourceSeverity::Error => "error",
                },
                diagnostic.code,
                diagnostic.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn line_column(source: &str, byte_offset: usize) -> (usize, usize) {
    let offset = byte_offset.min(source.len());
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.chars().count(), |(_, tail)| tail.chars().count())
        + 1;
    (line, column)
}
