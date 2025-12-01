use std::{env, path::PathBuf, process};

use stm32_bindings_gen::{Gen, Options};

fn main() {
    let out_dir = PathBuf::from("build/stm32-bindings");
    let sources_dir = PathBuf::from("sources");
    let target_triple = resolve_target_triple();

    let opts = Options {
        out_dir,
        sources_dir,
        target_triple,
    };
    Gen::new(opts).run_gen();
}

fn resolve_target_triple() -> String {
    let mut args = env::args().skip(1);
    let mut positional: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                eprintln!("Usage: stm32-bindings-gen [--target <triple>] [triple]");
                process::exit(0);
            }
            "--target" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("Expected a value after --target");
                    process::exit(1);
                });
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    eprintln!("Target triple cannot be empty.");
                    process::exit(1);
                }
                return trimmed.to_string();
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--target=") {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        eprintln!("Target triple cannot be empty.");
                        process::exit(1);
                    }
                    return trimmed.to_string();
                }
                if positional.is_none() {
                    let trimmed = arg.trim();
                    if !trimmed.is_empty() {
                        positional = Some(trimmed.to_string());
                    }
                }
            }
        }
    }

    positional
        .or_else(|| env::var("BINDGEN_TARGET").ok())
        .and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| "thumbv8m.main-none-eabihf".to_string())
}
