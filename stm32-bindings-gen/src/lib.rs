use bindgen::callbacks::{ItemInfo, ItemKind, ParseCallbacks};
use std::io::Write;
use std::{
    fs,
    path::{Path, PathBuf},
};
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

fn host_isystem_args() -> Vec<String> {
    let mut args = Vec::new();
    if cfg!(target_os = "macos") {
        if let Ok(output) = Command::new("xcrun").arg("--show-sdk-path").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let trimmed = path.trim();
                    if !trimmed.is_empty() {
                        args.push(format!("-isystem{}/usr/include", trimmed));
                    }
                }
            }
        }
    }
    args
}

pub struct Gen {
    opts: Options,
}

impl Gen {
    pub fn new(opts: Options) -> Self {
        Self { opts }
    }

    pub fn run_gen(&mut self) {
        println!(
            "Generating bindings into {} for target {}",
            self.opts.out_dir.display(),
            self.opts.target_triple
        );

        self.prepare_out_dir();
        self.write_static_files();

        let mut modules = Vec::new();
        let mut aliases = Vec::new();

        for spec in BINDING_SPECS {
            println!("  -> generating `{}` bindings", spec.module);
            self.generate_bindings_for_spec(spec);
            self.copy_artifacts_for_spec(spec);

            // The bindgen::Builder is the main entry point
            // to bindgen, and lets you build up options for
            // the resulting bindings.
            let target_flag = format!("--target={}", lib.target_triple);

            let mut builder = bindgen::Builder::default()
                .parse_callbacks(Box::new(UppercaseCallbacks))
                // Force Clang to use the same layout as the selected target.
                .clang_arg(&target_flag);

            let crate_inc = Path::new(env!("CARGO_MANIFEST_DIR")).join("inc");
            builder = builder.clang_arg(&format!("-iquote{}", crate_inc.display()));
            builder = builder.clang_arg(&format!("-I{}", crate_inc.display()));

            for include_arg in &lib.includes {
                builder = builder.clang_arg(&format!(
                    "-I{}",
                    sources_dir.join(include_arg).to_str().unwrap()
                ));
            }
        }

        self.write_bindings_mod(&modules, &aliases);
    }

    fn prepare_out_dir(&self) {
        let _ = fs::remove_dir_all(&self.opts.out_dir);
        self.create_dir(self.opts.out_dir.join("src/bindings"));
        self.create_dir(self.opts.out_dir.join("src/lib"));
    }

    fn write_static_files(&self) {
        self.write_bytes("README.md", include_bytes!("../res/README.md"));
        self.write_bytes("Cargo.toml", include_bytes!("../res/Cargo.toml"));
        self.write_bytes("build.rs", include_bytes!("../res/build.rs"));
        self.write_bytes("src/lib.rs", include_bytes!("../res/src/lib.rs"));
    }

    fn write_bindings_mod(
        &self,
        modules: &[(String, Option<String>)],
        aliases: &[(String, String, Option<String>)],
    ) {
        let mut body = String::new();
        for (module, feature) in modules {
            if let Some(feature) = feature {
                body.push_str(&format!("#[cfg(feature = \"{feature}\")]\n"));
            }
            body.push_str("pub mod ");
            body.push_str(module);
            body.push_str(";\n");
        }
        if !aliases.is_empty() {
            body.push('\n');
            for (module, alias, feature) in aliases {
                if let Some(feature) = feature {
                    body.push_str(&format!("#[cfg(feature = \"{feature}\")]\n"));
                }
                body.push_str("pub use self::");
                body.push_str(module);
                body.push_str(" as ");
                body.push_str(alias);
                body.push_str(";\n");
            }
        }
        self.write_string("src/bindings/mod.rs", body);
    }

    fn generate_bindings_for_spec(&self, spec: &BindingSpec) {
        let mut builder = bindgen::Builder::default()
            .parse_callbacks(Box::new(UppercaseCallbacks))
            .header(spec.header)
            .clang_arg(format!("--target={}", self.opts.target_triple));

        for arg in host_isystem_args() {
            builder = builder.clang_arg(arg);
        }

        let crate_inc = Path::new(env!("CARGO_MANIFEST_DIR")).join("inc");
        builder = builder.clang_arg(format!("-iquote{}", crate_inc.display()));

        if Self::is_thumb_target(&self.opts.target_triple) {
            builder = builder.clang_arg("-mthumb");
        }

        for dir in spec.include_dirs {
            let include_path = Path::new(dir);
            let resolved = if include_path.is_absolute() {
                include_path.to_path_buf()
            } else {
                self.opts.sources_dir.join(include_path)
            };
            builder = builder.clang_arg(format!("-I{}", resolved.display()));
        }

        for arg in spec.clang_args {
            builder = builder.clang_arg(*arg);
        }

        if !spec.allowlist.is_empty() {
            for pattern in spec.allowlist {
                builder = builder
                    .allowlist_type(pattern)
                    .allowlist_var(pattern)
                    .allowlist_function(pattern);
            }
        }

        let bindings = builder
            .generate()
            .unwrap_or_else(|err| panic!("Unable to generate bindings for {}: {err}", spec.module));

        let mut file_contents = bindings.to_string();
        file_contents = Self::normalize_bindings(file_contents);

        let out_path = self
            .opts
            .out_dir
            .join("src/bindings")
            .join(format!("{}.rs", spec.module));

        self.write_string_path(&out_path, file_contents);
    }

    fn copy_artifacts_for_spec(&self, spec: &BindingSpec) {
        for artifact in spec.library_artifacts {
            let src = self.opts.sources_dir.join(artifact.source);
            let dst = self.opts.out_dir.join(artifact.destination);

            if src.is_file() {
                self.copy_file(&src, &dst)
                    .unwrap_or_else(|err| panic!("Failed to copy file {}: {err}", src.display()));
            } else if src.is_dir() {
                self.copy_dir(&src, &dst)
                    .unwrap_or_else(|err| panic!("Failed to copy dir {}: {err}", src.display()));
            } else {
                panic!(
                    "Artifact source {} is neither file nor directory",
                    src.display()
                );
            }
        }
    }

    fn write_bytes(&self, relative: &str, bytes: &[u8]) {
        let path = self.opts.out_dir.join(relative);
        if let Some(parent) = path.parent() {
            self.create_dir(parent);
        }
        fs::write(path, bytes).expect("Unable to write bytes");
    }

    fn write_string(&self, relative: &str, contents: String) {
        let path = self.opts.out_dir.join(relative);
        self.write_string_path(&path, contents);
    }

    fn write_string_path(&self, path: &Path, mut contents: String) {
        if !contents.ends_with('\n') {
            contents.push('\n');
        }
        if let Some(parent) = path.parent() {
            self.create_dir(parent);
        }
        fs::write(path, contents).expect("Unable to write string");
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            fs::create_dir_all(path_ref).expect("Unable to create directory");
        }
    }

    fn copy_file(&self, src: &Path, dst: &Path) -> io::Result<()> {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        Ok(())
    }

    fn copy_dir(&self, src: &Path, dst: &Path) -> io::Result<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let target = dst.join(entry.file_name());
            if path.is_dir() {
                self.copy_dir(&path, &target)?;
            } else {
                self.copy_file(&path, &target)?;
            }
        }
        Ok(())
    }

    fn normalize_bindings(mut contents: String) -> String {
        for (from, to) in STD_TO_CORE_REPLACEMENTS {
            contents = contents.replace(from, to);
        }

        contents
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
            .join("\n")
    }

    fn is_thumb_target(triple: &str) -> bool {
        triple.trim().to_ascii_lowercase().starts_with("thumb")
    }
}
