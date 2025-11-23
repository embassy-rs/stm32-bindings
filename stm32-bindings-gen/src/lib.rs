use std::{fs, path::PathBuf};

pub struct Options {
    pub out_dir: PathBuf,
    pub sources_dir: PathBuf,
}

pub struct Gen {
    opts: Options,
}

impl Gen {
    pub fn new(opts: Options) -> Self {
        Self { opts }
    }

    pub fn run_gen(&mut self) {
        fs::create_dir_all(self.opts.out_dir.join("src/bindings")).unwrap();
        fs::create_dir_all(self.opts.out_dir.join("src/lib")).unwrap();

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let bindings = bindgen::Builder::default()
            .clang_arg(format!(
                "-I{}/Middlewares/ST/STM32_WPAN/mac_802_15_4/core/inc",
                self.opts.sources_dir.to_str().unwrap()
            ))
            // The input header we would like to generate
            // bindings for.
            .header("stm32-bindings-gen/inc/wpan-wba.h")
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(self.opts.out_dir.join("src/bindings/wpan-wba.rs"))
            .expect("Couldn't write bindings!");

        // copy misc files
        fs::copy(
            self.opts
                .sources_dir
                .join("Middlewares/ST/STM32_WPAN/mac_802_15_4/lib/wba_mac_lib.a"),
            self.opts.out_dir.join("src/lib/wba_mac_lib.a"),
        )
        .unwrap();
        fs::write(
            self.opts.out_dir.join("README.md"),
            include_bytes!("../res/README.md"),
        )
        .unwrap();
        fs::write(
            self.opts.out_dir.join("build.rs"),
            include_bytes!("../res/build.rs"),
        )
        .unwrap();
        fs::write(
            self.opts.out_dir.join("src/lib.rs"),
            include_bytes!("../res/src/lib.rs"),
        )
        .unwrap();
    }
}
