use std::path::PathBuf;

use stm32_bindings_gen::{Gen, Options};

use clap::Parser;

/// A simple CLI tool
#[derive(Parser, Debug)]
#[command(
    name = "stm32-bindings-gen",
    version = env!("CARGO_PKG_VERSION"),
    about = "Generation of Bindings for STM32 Middlewares"
)]
struct Cli {
    /// Sources Directory
    #[arg(long, default_value = "sources")]
    sources_dir: PathBuf,

    /// Build Directory
    #[arg(long, default_value = "build")]
    build_dir: PathBuf,

    /// If given, only build this module
    #[arg(long, default_value = None)]
    module: Option<String>,

    /// Delete generated static libraries that no `lib_*` feature references.
    ///
    /// Use this before packaging/publishing so the crate stays within
    /// crates.io's upload size limit. Opt-in: local builds keep every library.
    #[arg(long, default_value_t = false)]
    prune_unused_libs: bool,
}

fn main() {
    let args = Cli::parse();

    let opts = Options {
        build_dir: args.build_dir,
        sources_dir: args.sources_dir,
        prune_unused_libs: args.prune_unused_libs,
    };

    Gen::new(opts).run_gen(&args.module);
}
