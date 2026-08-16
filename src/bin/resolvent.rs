use resolvent::{IncidenceSystem, compile_schedule, parse_and_elaborate, parse_rsl};
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    match run() { Ok(()) => ExitCode::SUCCESS, Err(message) => { eprintln!("{message}"); ExitCode::from(2) } }
}

fn run() -> Result<(),String> {
    let mut args=env::args().skip(1);let command=args.next().ok_or_else(usage)?;let file=args.next().ok_or_else(usage)?;let json=args.any(|a|a=="--format=json"||a=="--json");let source=fs::read_to_string(&file).map_err(|e|format!("{file}: {e}"))?;
    match command.as_str(){
        "check"=>match parse_and_elaborate(&source){Ok((_ctx,model,out))=>{if json{println!("{}",serde_json::json!({"ok":true,"model":model.name,"equations":out.system.equations.len(),"fields":out.fields.len()}))}else{println!("ok: {} ({} equation(s), {} field(s))",model.name,out.system.equations.len(),out.fields.len())}Ok(())},Err(e)=>emit_error(e,json)},
        "inspect"=>{let(_ctx,model,out)=parse_and_elaborate(&source).map_err(|e|physics_error(e,json))?;if json{println!("{}",serde_json::to_string_pretty(&serde_json::json!({"source":model,"semantic":out})).unwrap())}else{println!("model {}\ndomains: {}\nfields: {}\nequations: {}\nproperties: {}",model.name,model.domains.len(),model.fields.len(),model.equations.len(),model.properties.len())}Ok(())},
        "lower"=>{let(ctx,_model,out)=parse_and_elaborate(&source).map_err(|e|physics_error(e,json))?;let target=env::args().find_map(|a|a.strip_prefix("--to=").map(str::to_owned)).unwrap_or_else(||"system".into());match target.as_str(){"system"=>println!("{}",serde_json::to_string_pretty(&out.system).unwrap()),"spec"=>println!("{}",serde_json::to_string_pretty(&out.spec).unwrap()),"context"=>println!("{}",serde_json::to_string_pretty(&ctx).unwrap()),_=>return Err(format!("lowering target `{target}` requires a method/discretization declaration; supported source-level targets: system, spec, context"))}Ok(())},
        "structural"=>{let(ctx,_model,out)=parse_and_elaborate(&source).map_err(|e|physics_error(e,json))?;let incidence=IncidenceSystem::from_system(&out.system,&ctx.exprs).map_err(|e|e.to_string())?;let schedule=compile_schedule(&incidence);if json{println!("{}",serde_json::to_string_pretty(&serde_json::json!({"incidence":incidence,"schedule":schedule.as_ref().ok().map(|s|&s.blocks),"error":schedule.as_ref().err().map(ToString::to_string)})).unwrap())}else{println!("{} equations / {} variables",incidence.n_equations(),incidence.n_variables());match schedule{Ok(s)=>for(i,b)in s.blocks.iter().enumerate(){println!("block {i}: {:?} equations={:?} solved={:?} tearing={:?}",b.kind,b.equations,b.solved_vars,b.tearing_vars)},Err(e)=>println!("not causalized: {e}")}}Ok(())},
        "freeze"=>{let lock=resolvent::physics::freeze(&source).map_err(|e|physics_error(e,json))?;println!("{}",serde_json::to_string_pretty(&lock).unwrap());Ok(())},
        "trace"=>{let(_ctx,_model,out)=parse_and_elaborate(&source).map_err(|e|physics_error(e,json))?;if json{println!("{}",serde_json::to_string_pretty(&out.source_map).unwrap())}else{for(name,span)in out.source_map{println!("{name}: {}..{}",span.start,span.end)}}Ok(())},
        "parse"=>match parse_rsl(&source){Ok(m)=>{println!("{}",serde_json::to_string_pretty(&m).unwrap());Ok(())},Err(d)=>{if json{println!("{}",serde_json::to_string_pretty(&d).unwrap())}else{for x in d{eprintln!("{} [{}..{}] {}",x.code,x.span.start,x.span.end,x.message)}}Err("source contains diagnostics".into())}},
        _=>Err(usage()),
    }
}
fn usage()->String{"usage: resolvent <check|parse|inspect|lower|structural|trace|freeze> <model.res> [--json] [--to=system]".into()}
fn emit_error(e:resolvent::physics::PhysicsError,json:bool)->Result<(),String>{Err(physics_error(e,json))}
fn physics_error(e:resolvent::physics::PhysicsError,json:bool)->String{match e{resolvent::physics::PhysicsError::Diagnostics(d)=>{if json{serde_json::to_string_pretty(&d).unwrap_or_else(|_|"diagnostics could not be serialized".into())}else{d.into_iter().map(|x|format!("{} [{}..{}] {}{}",x.code,x.span.start,x.span.end,x.message,if x.hints.is_empty(){String::new()}else{format!("\n  hint: {}",x.hints.join("; "))})).collect::<Vec<_>>().join("\n")}},other=>other.to_string()}}
