use bindgen::callbacks::{ItemInfo, ItemKind, ParseCallbacks};
use std::io::Write;
use std::{fs, path::PathBuf};
use tempfile::NamedTempFile;

#[derive(Debug)]
struct UppercaseCallbacks;

impl ParseCallbacks for UppercaseCallbacks {
    fn item_name(&self, item: ItemInfo<'_>) -> Option<String> {
        if matches!(item.kind, ItemKind::Var) {
            Some(item.name.to_ascii_uppercase())
        } else {
            None
        }
    }
}

pub struct Options {
    pub out_dir: PathBuf,
    pub sources_dir: PathBuf,
    pub target_triple: String,
}

pub struct Gen {
    opts: Options,
}

impl Gen {
    pub fn new(opts: Options) -> Self {
        Self { opts }
    }

    pub fn run_gen(&mut self) {
        let _ = fs::remove_dir_all(self.opts.out_dir.clone());
        fs::create_dir_all(self.opts.out_dir.join("src/bindings")).unwrap();
        fs::create_dir_all(self.opts.out_dir.join("src/lib")).unwrap();

        // Create a named temporary file
        let mut header = NamedTempFile::new().unwrap();

        // Write some data to the first handle
        header
            .write_all(include_bytes!("../inc/wpan-wba.h"))
            .unwrap();

        header.reopen().unwrap();

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let target_flag = format!("--target={}", self.opts.target_triple);
        let include_arg = format!(
            "-I{}/Middlewares/ST/STM32_WPAN/mac_802_15_4/core/inc",
            self.opts.sources_dir.to_str().unwrap()
        );
        let mut builder = bindgen::Builder::default()
            .parse_callbacks(Box::new(UppercaseCallbacks))
            // Force Clang to use the same layout as the selected target.
            .clang_arg(&target_flag)
            .clang_arg(&include_arg);
        if self
            .opts
            .target_triple
            .to_ascii_lowercase()
            .starts_with("thumb")
        {
            builder = builder.clang_arg("-mthumb");
        }
        let bindings = builder
            // The input header we would like to generate
            // bindings for.
            .header("stm32-bindings-gen/inc/wpan-wba.h")
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        let out_path = self.opts.out_dir.join("src/bindings/wpan_wba.rs");

        bindings
            .write_to_file(&out_path)
            .expect("Couldn't write bindings!");

        let mut file_contents = fs::read_to_string(&out_path).unwrap();
        file_contents = file_contents
            .replace("::std::mem::", "::core::mem::")
            .replace("::std::os::raw::", "::core::ffi::")
            .replace("::std::option::", "::core::option::");

        file_contents = file_contents
            .lines()
            .map(|line| {
                if let Some(rest) = line.strip_prefix("pub const ") {
                    if let Some((name, tail)) = rest.split_once(':') {
                        let upper = name.trim().to_ascii_uppercase();
                        return format!("pub const {}:{}", upper, tail);
                    }
                }
                line.to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n");

        if !file_contents.ends_with('\n') {
            file_contents.push('\n');
        }

        fs::write(&out_path, file_contents).unwrap();

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
            self.opts.out_dir.join("Cargo.toml"),
            include_bytes!("../res/Cargo.toml"),
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

        fs::write(
            self.opts.out_dir.join("src/bindings/mod.rs"),
            include_bytes!("../res/src/bindings/mod.rs"),
        )
        .unwrap();
    }
}
