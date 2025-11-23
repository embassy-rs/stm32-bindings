use std::path::PathBuf;

use stm32_bindings_gen::{Gen, Options};

fn main() {
    let out_dir = PathBuf::from("build/stm32-bindings");
    let sources_dir = PathBuf::from("sources");

    // let args: Vec<String> = args().collect();

    let opts = Options {
        out_dir,
        sources_dir,
    };
    Gen::new(opts).run_gen();
}
