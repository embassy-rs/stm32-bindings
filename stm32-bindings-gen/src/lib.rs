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
}

pub struct Library {
    pub sources_dir: PathBuf,
    pub target_triple: String,
    pub header: &'static [u8],
    pub module: &'static str,
    pub includes: Vec<PathBuf>,
    pub library: PathBuf,
}

pub struct Gen {
    opts: Options,
    libs: Vec<Library>,
}

impl Gen {
    pub fn new(opts: Options, libs: Vec<Library>) -> Self {
        Self { opts, libs }
    }

    pub fn run_gen(&mut self) {
        let _ = fs::remove_dir_all(self.opts.out_dir.clone());
        fs::create_dir_all(self.opts.out_dir.join("src/bindings")).unwrap();
        fs::create_dir_all(self.opts.out_dir.join("src/lib")).unwrap();

        for lib in &self.libs {
            let sources_dir = self.opts.sources_dir.join(&lib.sources_dir);

            // Create a named temporary file
            let mut header = NamedTempFile::with_suffix(".h").unwrap();

            // Write some data to the first handle
            header.write_all(lib.header).unwrap();

            // The bindgen::Builder is the main entry point
            // to bindgen, and lets you build up options for
            // the resulting bindings.
            let target_flag = format!("--target={}", lib.target_triple);

            let mut builder = bindgen::Builder::default()
                .parse_callbacks(Box::new(UppercaseCallbacks))
                // Force Clang to use the same layout as the selected target.
                .clang_arg(&target_flag);

            for include_arg in &lib.includes {
                builder = builder.clang_arg(&format!(
                    "-I{}",
                    sources_dir.join(include_arg).to_str().unwrap()
                ));
            }

            if lib.target_triple.to_ascii_lowercase().starts_with("thumb") {
                builder = builder.clang_arg("-mthumb");
            }
            let bindings = builder
                // The input header we would like to generate
                // bindings for.
                .header(header.path().to_str().unwrap())
                // Finish the builder and generate the bindings.
                .generate()
                // Unwrap the Result and panic on failure.
                .expect("Unable to generate bindings");

            let out_path = self
                .opts
                .out_dir
                .join("src")
                .join("bindings")
                .join(format!("{}.rs", lib.module));

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
                sources_dir.join(&lib.library),
                self.opts
                    .out_dir
                    .join("src")
                    .join("lib")
                    .join(format!("{}.a", lib.module)),
            )
            .unwrap();
        }

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
