use std::fs;
use std::path::Path;

fn rust_sources(path: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(path).expect("source directory must be readable") {
        let path = entry.expect("source entry must be readable").path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn production_source_contains_no_consumer_owned_vocabulary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    rust_sources(&root.join("src"), &mut sources);
    let forbidden = [
        ".res",
        "SemanticModel",
        "ExprId",
        "cadabra-exact",
        "cadabra-scalar",
        "BRep",
        "HalfEdge",
        "Methodus",
        "Solverang",
        "ConstraintGraph",
        "QFunction",
        "TensorProgram",
    ];
    for source in sources {
        let text = fs::read_to_string(&source).expect("Rust source must be UTF-8");
        for needle in forbidden {
            assert!(
                !text.contains(needle),
                "{} contains consumer-owned vocabulary {needle:?}",
                source.display()
            );
        }
    }
}

#[test]
fn deleted_cadabra_facades_are_not_dependencies() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in ["Cargo.toml", "Cargo.lock"] {
        let text = fs::read_to_string(root.join(name)).expect("manifest must be readable");
        assert!(!text.contains("cadabra-exact"));
        assert!(!text.contains("cadabra-scalar"));
    }
}
