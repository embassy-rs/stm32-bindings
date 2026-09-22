use bindgen::callbacks::{ItemInfo, ItemKind, ParseCallbacks};
use lazy_regex::regex;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};
use syn::fold::Fold;
use syn::{ForeignItem, ForeignItemFn, ItemForeignMod};

const NEWLIB_SHARED_OPAQUES: &[&str] = &["_reent", "__sFILE", "__sFILE64"];

#[derive(Debug, Clone, Copy)]
enum Directory {
    Sources(&'static str),
    #[allow(dead_code)]
    Build(&'static str),
    Vendored(&'static str),
}

#[derive(Debug, Clone, Copy)]
struct BindingSpec {
    module: &'static str,
    feature: Option<&'static str>,
    header: &'static str,
    root: Directory,
    target_triple: &'static str,
    include_dirs: &'static [&'static str],
    clang_args: &'static [&'static str],
    allowlist: &'static [&'static str],
    aliases: &'static [&'static str],
    library_artifacts: &'static [LibraryArtifact],
}

#[derive(Debug, Clone, Copy)]
struct LibraryArtifact {
    source: &'static str,
    destination: &'static str,
}

const BINDING_SPECS: &[BindingSpec] = &[
    BindingSpec {
        module: "wba_link_layer",
        feature: Some("wba_wpan"),
        header: "stm32-bindings-gen/inc/link_layer.h",
        root: Directory::Sources("STM32CubeWBA"),
        target_triple: "thumbv8m.main-none-eabihf",
        include_dirs: &[
            "Middlewares/ST/STM32_WPAN",
            "Middlewares/ST/STM32_WPAN/mac_802_15_4/core/inc",
            "Middlewares/ST/STM32_WPAN/mac_802_15_4/mac_utilities/inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_sys/inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc/_40nm_reg_files",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc/ot_inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/config",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/config/ieee_15_4_basic",
            "Drivers/CMSIS/Core/Include",
        ],
        clang_args: &[
            "-DSUPPORT_MAC=1",
            "-DSUPPORT_BLE=1",
            "-DMAC=1",
            "-DBLE=1",
            "-DBLE_LL=1",
            "-DMAC_LAYER=1",
            "-DSUPPORT_MAC=1",
            "-DSUPPORT_CONFIG_LIB=1",
            "-DSUPPORT_OPENTHREAD_1_2=1",
            "-DSUPPORT_ANT_DIV=1",
            "-DEXT_ADDRESS_LENGTH=8",
        ],
        allowlist: &[],
        aliases: &[],
        library_artifacts: &[LibraryArtifact {
            source: "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/lib",
            destination: "src/lib/link_layer",
        }],
    },
    BindingSpec {
        module: "wba_wpan_mac",
        feature: Some("wba_wpan_mac"),
        header: "stm32-bindings-gen/inc/wba_wpan_mac.h",
        root: Directory::Sources("STM32CubeWBA"),
        target_triple: "thumbv8m.main-none-eabihf",
        include_dirs: &[
            "Middlewares/ST/STM32_WPAN",
            "Middlewares/ST/STM32_WPAN/mac_802_15_4/core/inc",
            "Middlewares/ST/STM32_WPAN/mac_802_15_4/mac_utilities/inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_sys/inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc/_40nm_reg_files",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc/ot_inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/config",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/config/ieee_15_4_basic",
            "Drivers/CMSIS/Core/Include",
        ],
        clang_args: &["-DSUPPORT_MAC=1", "-DMAC=1", "-DMAC_LAYER=1"],
        allowlist: &[],
        aliases: &["mac", "mac_802_15_4", "wpan_wba"],
        library_artifacts: &[
            LibraryArtifact {
                source: "Middlewares/ST/STM32_WPAN/mac_802_15_4/lib",
                destination: "src/lib/wba_wpan_mac",
            },
            LibraryArtifact {
                source: "Middlewares/ST/STM32_WPAN/mac_802_15_4/lib/wba_mac_lib.a",
                destination: "src/lib/wba_mac_lib.a",
            },
        ],
    },
    BindingSpec {
        module: "wba_ble_stack",
        feature: Some("wba_wpan_ble"),
        header: "stm32-bindings-gen/inc/wba_ble.h",
        root: Directory::Sources("STM32CubeWBA"),
        target_triple: "thumbv8m.main-none-eabihf",
        include_dirs: &[
            "Middlewares/ST/STM32_WPAN",
            "Middlewares/ST/STM32_WPAN/ble/stack/include",
            "Middlewares/ST/STM32_WPAN/ble/stack/include/auto",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_sys/inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc/_40nm_reg_files",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/inc/ot_inc",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/config",
            "Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib/config/ble_basic_plus",
            "Middlewares/ST/STM32_WPAN/ble/audio/Inc",
            "Middlewares/ST/STM32_WPAN/ble/codec/codec_manager/Inc",
            "Middlewares/ST/STM32_WPAN/ble/codec/lc3/Inc",
            "Drivers/CMSIS/Core/Include",
        ],
        clang_args: &[
            "-DBLE=1",
            "-DBLE_LL=1",
            "-DSUPPORT_BLE=1",
            "-DMAC=1",
            "-DMAC_LAYER=1",
            "-DSUPPORT_MAC=1",
            "-DSUPPORT_CONFIG_LIB=1",
            "-DSUPPORT_OPENTHREAD_1_2=1",
            "-DSUPPORT_ANT_DIV=1",
            "-DEXT_ADDRESS_LENGTH=8",
        ],
        allowlist: &[],
        aliases: &["ble", "ble_wba"],
        library_artifacts: &[
            LibraryArtifact {
                source: "Middlewares/ST/STM32_WPAN/ble/stack/lib",
                destination: "src/lib/ble/stack",
            },
            LibraryArtifact {
                source: "Middlewares/ST/STM32_WPAN/ble/audio/lib",
                destination: "src/lib/ble/audio",
            },
            LibraryArtifact {
                source: "Middlewares/ST/STM32_WPAN/ble/codec/codec_manager/Lib",
                destination: "src/lib/ble/codec_manager",
            },
            LibraryArtifact {
                source: "Middlewares/ST/STM32_WPAN/ble/codec/lc3/Lib",
                destination: "src/lib/ble/lc3",
            },
        ],
    },
    BindingSpec {
        module: "wba_ble_uuid",
        feature: Some("wba_wpan_ble_uuid"),
        header: "stm32-bindings-gen/inc/wba_ble_uuid.h",
        root: Directory::Sources("STM32CubeWBA"),
        target_triple: "thumbv8m.main-none-eabihf",
        include_dirs: &[
            "Middlewares/ST/STM32_WPAN",
            "Middlewares/ST/STM32_WPAN/ble/svc/Inc",
            "Drivers/CMSIS/Core/Include",
        ],
        clang_args: &[],
        allowlist: &[],
        aliases: &["ble_uuid"],
        library_artifacts: &[],
    },
    BindingSpec {
        module: "wba_ble_svc",
        feature: Some("wba_wpan_ble_svc"),
        header: "stm32-bindings-gen/inc/wba_ble_svc.h",
        root: Directory::Sources("STM32CubeWBA"),
        target_triple: "thumbv8m.main-none-eabihf",
        include_dirs: &[
            "Middlewares/ST/STM32_WPAN",
            "Middlewares/ST/STM32_WPAN/ble/svc/Inc",
            "Drivers/CMSIS/Core/Include",
        ],
        clang_args: &[],
        allowlist: &[],
        aliases: &["ble_svc"],
        library_artifacts: &[],
    },
    BindingSpec {
        module: "wba_openthread",
        feature: Some("wba_wpan_openthread"),
        header: "stm32-bindings-gen/inc/wba_openthread.h",
        root: Directory::Sources("STM32CubeWBA"),
        target_triple: "thumbv8m.main-none-eabihf",
        include_dirs: &[
            "Middlewares/ST/STM32_WPAN",
            "Middlewares/ST/STM32_WPAN/thread/openthread/stack/include",
            "Middlewares/ST/STM32_WPAN/thread/openthread/common",
            "Middlewares/ST/STM32_WPAN/thread/openthread/config",
            "Drivers/CMSIS/Core/Include",
        ],
        clang_args: &[
            "-DOPENTHREAD_RADIO_INTERFACE_VERSION=100",
            "-DOPENTHREAD_CONFIG_ENABLE_ALL_OPTIONAL_ARGS=1",
        ],
        allowlist: &[],
        aliases: &["openthread"],
        library_artifacts: &[LibraryArtifact {
            source: "Middlewares/ST/STM32_WPAN/thread/openthread/openthread_lib",
            destination: "src/lib/openthread",
        }],
    },
    BindingSpec {
        module: "wb_openthread",
        feature: Some("wb_wpan_openthread"),
        header: "stm32-bindings-gen/inc/wb_openthread.h",
        root: Directory::Build("thread"),
        target_triple: "thumbv7em.main-none-eabihf",
        include_dirs: &[
            "build/include",
            "build/include/openthread",
            "build/include/openthread/platform",
            "build/core/openthread_api",
            "build/core/openthread_config",
            "build/include/Include",
            "build/include/Inc",
        ],
        clang_args: &[
            "-DOPENTHREAD_RADIO_INTERFACE_VERSION=100",
            "-DOPENTHREAD_CONFIG_ENABLE_ALL_OPTIONAL_ARGS=1",
        ],
        allowlist: &[],
        aliases: &["openthread"],
        library_artifacts: &[LibraryArtifact {
            source: "build/libopenthread.a",
            destination: "src/lib/openthread/stm32wb_ot_mtd_lib.a",
        }],
    },
    BindingSpec {
        module: "nema_gfx",
        feature: Some("nema_gfx"),
        header: "stm32-bindings-gen/inc/nema_gfx.h",
        root: Directory::Vendored("nema_gfx"),
        target_triple: "thumbv8m.main-none-eabihf",
        include_dirs: &["include"],
        clang_args: &["-mcpu=cortex-m55"],
        allowlist: &[],
        aliases: &["nemagfx"],
        library_artifacts: &[LibraryArtifact {
            source: "lib",
            destination: "src/lib/nema_gfx",
        }],
    },
];

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
    pub build_dir: PathBuf,
    pub sources_dir: PathBuf,
    /// Delete generated static libraries that no `lib_*` feature references.
    ///
    /// This keeps the packaged crate within crates.io's upload limit. The
    /// default (`false`) keeps every copied library, which is what local
    /// development and generated-crate checks want.
    pub prune_unused_libs: bool,
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

/// Removes matching functions
struct ForeignFnsRemover<'a> {
    regex: &'a regex::Regex,
}

impl<'a> Fold for ForeignFnsRemover<'a> {
    fn fold_item_foreign_mod(&mut self, mut node: ItemForeignMod) -> ItemForeignMod {
        // First, recursively fold subnodes
        node = syn::fold::fold_item_foreign_mod(self, node);

        // Then filter out matching functions
        node.items.retain(|item| {
            if let ForeignItem::Fn(ForeignItemFn { sig, .. }) = item {
                let name = sig.ident.to_string();
                !self.regex.is_match(&name)
            } else {
                true
            }
        });

        node
    }
}

/// Transforms the functions to uppercase
struct LinkNameAttrAdder<'a> {
    regex: &'a regex::Regex,
}

impl<'a> Fold for LinkNameAttrAdder<'a> {
    fn fold_foreign_item_fn(&mut self, mut func: ForeignItemFn) -> ForeignItemFn {
        let fn_name = func.sig.ident.to_string();
        // Only add if not already present
        if !func.attrs.iter().any(|a| a.path().is_ident("link_name"))
            && self.regex.is_match(&fn_name)
        {
            let link_name_value = fn_name.to_uppercase();

            func.attrs.push(syn::parse_quote!(
                #[link_name = #link_name_value]
            ));
        }

        // Recurse into the function body (nested functions in blocks)
        syn::fold::fold_foreign_item_fn(self, func)
    }
}

pub struct Gen {
    opts: Options,
}

impl Gen {
    pub fn new(opts: Options) -> Self {
        Self { opts }
    }

    pub fn run_gen(&mut self, module: &Option<String>) {
        println!("Generating bindings into {}", self.opts.build_dir.display(),);

        self.prepare_build_dir();
        self.write_static_files();

        let mut modules = Vec::new();
        let mut aliases = Vec::new();

        for spec in BINDING_SPECS {
            if module.as_ref().is_some_and(|module| module != spec.module) {
                continue;
            }

            println!("  -> generating `{}` bindings", spec.module);
            let sources_dir = match spec.root {
                Directory::Build(dir) => self.opts.build_dir.join(dir),
                Directory::Sources(dir) => self.opts.sources_dir.join(dir),
                Directory::Vendored(dir) => Path::new(env!("CARGO_MANIFEST_DIR")).join(dir),
            };

            self.generate_bindings_for_spec(spec, &sources_dir);
            self.copy_artifacts_for_spec(spec, &sources_dir);

            modules.push((spec.module.to_owned(), spec.feature.map(str::to_owned)));
            for alias in spec.aliases {
                aliases.push((
                    spec.module.to_owned(),
                    alias.to_string(),
                    spec.feature.map(str::to_owned),
                ));
            }
        }

        self.write_bindings_mod(&modules, &aliases);

        if self.opts.prune_unused_libs {
            self.prune_unused_libs();
        }
    }

    fn build_dir(&self) -> PathBuf {
        self.opts.build_dir.join("stm32-bindings")
    }

    fn prepare_build_dir(&self) {
        let _ = fs::remove_dir_all(&self.build_dir());
        self.create_dir(self.build_dir().join("src/bindings"));
        self.create_dir(self.build_dir().join("src/lib"));
    }

    fn write_static_files(&self) {
        self.write_bytes("README.md", include_bytes!("../res/README.md"));
        self.write_bytes(
            "Cargo.toml",
            str::from_utf8(include_bytes!("../res/Cargo.toml"))
                .unwrap()
                .replace("$VERSION$", env!("CARGO_PKG_VERSION"))
                .as_bytes(),
        );
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

    fn generate_bindings_for_spec(&self, spec: &BindingSpec, sources_dir: &Path) {
        let mut builder = bindgen::Builder::default()
            .parse_callbacks(Box::new(UppercaseCallbacks))
            .header(spec.header)
            .clang_arg(format!("--target={}", spec.target_triple));

        println!("sources dir: {:?}", sources_dir);

        for arg in host_isystem_args() {
            builder = builder.clang_arg(arg);
        }

        let crate_inc = Path::new(env!("CARGO_MANIFEST_DIR")).join("inc");
        builder = builder.clang_arg(format!("-iquote{}", crate_inc.display()));
        builder = builder.clang_arg(format!("-I{}", crate_inc.display()));

        if Self::is_thumb_target(&spec.target_triple) {
            builder = builder.clang_arg("-mthumb");
        }

        for dir in spec.include_dirs {
            let include_path = Path::new(dir);
            let resolved = if include_path.is_absolute() {
                include_path.to_path_buf()
            } else {
                sources_dir.join(include_path)
            };
            builder = builder.clang_arg(format!("-I{}", resolved.display()));
        }

        for arg in spec.clang_args {
            builder = builder.clang_arg(*arg);
        }

        for ty in NEWLIB_SHARED_OPAQUES {
            builder = builder.opaque_type(ty);
        }

        for arg in arm_sysroot_args() {
            builder = builder.clang_arg(arg);
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
            .use_core()
            .enable_function_attribute_detection()
            .derive_debug(false)
            .derive_default(true)
            .layout_tests(false)
            .generate()
            .unwrap_or_else(|err| panic!("Unable to generate bindings for {}: {err}", spec.module));

        let out_path = self
            .build_dir()
            .join("src/bindings")
            .join(format!("{}.rs", spec.module));

        let file = syn::parse_file(&bindings.to_string()).unwrap();
        let file = LinkNameAttrAdder {
            regex: &*regex!("^(aci|hal|hci)_.*"),
        }
        .fold_file(file);
        let file = ForeignFnsRemover {
            regex: &*regex!("^(ACI|HAL|HCI)_.*"),
        }
        .fold_file(file);

        self.write_string_path(
            &out_path,
            self.format_tokens(file.into_token_stream()).unwrap(),
        );
    }

    fn copy_artifacts_for_spec(&self, spec: &BindingSpec, sources_dir: &Path) {
        for artifact in spec.library_artifacts {
            let src = sources_dir.join(artifact.source);
            let dst = self.build_dir().join(artifact.destination);

            if src.is_file() {
                self.copy_lib(&src, &dst)
                    .unwrap_or_else(|err| panic!("Failed to copy file {}: {err}", src.display()));
            } else if src.is_dir() {
                self.copy_lib_dir(&src, &dst)
                    .unwrap_or_else(|err| panic!("Failed to copy dir {}: {err}", src.display()));
            } else {
                panic!(
                    "Artifact source {} is neither file nor directory",
                    src.display()
                );
            }
        }
    }

    fn format_tokens(&self, tokens: TokenStream) -> io::Result<String> {
        // Convert AST back to a token stream
        let tokens = tokens.to_string();

        // Format using rustfmt
        let mut rustfmt = Command::new("rustfmt")
            .arg("--emit")
            .arg("stdout")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let stdin = rustfmt
                .stdin
                .as_mut()
                .ok_or(io::Error::other("Failed to open rustfmt stdin"))?;
            stdin.write_all(tokens.as_bytes())?;
        }

        let output = rustfmt.wait_with_output()?;
        if !output.status.success() {
            return Err(io::Error::other("rustfmt failed"));
        }

        String::from_utf8(output.stdout).map_err(|e| io::Error::other(e))
    }

    fn write_bytes(&self, relative: &str, bytes: &[u8]) {
        let path = self.build_dir().join(relative);
        if let Some(parent) = path.parent() {
            self.create_dir(parent);
        }
        fs::write(path, bytes).expect("Unable to write bytes");
    }

    fn write_string(&self, relative: &str, contents: String) {
        let path = self.build_dir().join(relative);
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

    fn copy_lib(&self, src: &Path, dst: &Path) -> io::Result<()> {
        if !(src.extension().is_some_and(|ext| ext == OsStr::new("a"))) {
            return Ok(());
        }

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }

        let dst = if dst
            .file_name()
            .is_some_and(|file_name| file_name.to_string_lossy().starts_with("lib"))
        {
            dst.to_path_buf()
        } else {
            let file_name = "lib".to_string()
                + dst
                    .file_name()
                    .ok_or(io::Error::new(io::ErrorKind::InvalidFilename, ""))?
                    .to_str()
                    .ok_or(io::Error::new(io::ErrorKind::InvalidFilename, ""))?;

            dst.parent()
                .unwrap_or(&Path::new(""))
                .join(file_name.to_ascii_lowercase())
        };

        fs::copy(src, dst)?;
        Ok(())
    }

    fn copy_lib_dir(&self, src: &Path, dst: &Path) -> io::Result<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let target = dst.join(entry.file_name());
            if path.is_dir() {
                self.copy_lib_dir(&path, &target)?;
            } else {
                self.copy_lib(&path, &target)?;
            }
        }
        Ok(())
    }

    fn is_thumb_target(triple: &str) -> bool {
        triple.trim().to_ascii_lowercase().starts_with("thumb")
    }

    /// Deletes copied static libraries that no `lib_*` feature references.
    ///
    /// `res/build.rs` turns every `lib_<name>` feature into
    /// `-l static=<name>` (lowercased), so a feature `lib_foo` requires an
    /// archive named `libfoo.a`. Anything else in `src/lib` can never be
    /// selected by a user and only bloats the published crate.
    fn prune_unused_libs(&self) {
        let manifest_path = self.build_dir().join("Cargo.toml");
        let manifest = fs::read_to_string(&manifest_path)
            .unwrap_or_else(|err| panic!("Unable to read {}: {err}", manifest_path.display()));

        let expected = expected_lib_filenames(&manifest);
        if expected.is_empty() {
            panic!(
                "Refusing to prune libraries: no `lib_*` features found in {}",
                manifest_path.display()
            );
        }

        let lib_dir = self.build_dir().join("src/lib");
        let mut kept = 0usize;
        let mut removed = 0usize;
        let mut removed_bytes = 0u64;

        for path in collect_archives(&lib_dir) {
            let file_name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();

            if expected.contains(&file_name) {
                kept += 1;
                continue;
            }

            removed_bytes += fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
            fs::remove_file(&path)
                .unwrap_or_else(|err| panic!("Unable to remove {}: {err}", path.display()));
            removed += 1;
        }

        remove_empty_dirs(&lib_dir);

        println!(
            "  -> pruned {removed} unused static libraries ({removed_bytes} bytes), kept {kept}"
        );

        if kept == 0 {
            panic!("No static libraries remain after pruning; refusing to produce an empty crate");
        }
    }
}

/// Returns the archive names (lowercase `lib*.a`) required by the `lib_*`
/// features declared in the generated `Cargo.toml`.
fn expected_lib_filenames(manifest: &str) -> BTreeSet<String> {
    let mut expected = BTreeSet::new();

    for line in manifest.lines() {
        let Some(rest) = line.trim().strip_prefix("lib_") else {
            continue;
        };
        let Some((name, _)) = rest.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty() {
            expected.insert(format!("lib{}.a", name.to_ascii_lowercase()));
        }
    }

    expected
}

/// Recursively collects every `.a` archive below `dir`.
fn collect_archives(dir: &Path) -> Vec<PathBuf> {
    let mut archives = Vec::new();

    let Ok(entries) = fs::read_dir(dir) else {
        return archives;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            archives.extend(collect_archives(&path));
        } else if path.extension().is_some_and(|ext| ext == OsStr::new("a")) {
            archives.push(path);
        }
    }

    archives
}

/// Removes directories that became empty after pruning. `remove_dir` fails on
/// non-empty directories, which is the behaviour we want, so errors are ignored.
fn remove_empty_dirs(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            remove_empty_dirs(&path);
            let _ = fs::remove_dir(&path);
        }
    }
}

fn arm_sysroot_args() -> Vec<String> {
    let mut args = Vec::new();
    let mut system_include_paths = BTreeSet::new();

    let mut push_sysroot = |path: &Path| {
        system_include_paths.insert(path.join("include"));
        system_include_paths.insert(path.join("include-fixed"));
        system_include_paths.insert(path.join("usr/include"));
        system_include_paths.insert(path.join("usr/include/newlib"));
        system_include_paths.insert(path.join("arm-none-eabi/include"));

        let arg = format!("--sysroot={}", path.display());
        if !args.iter().any(|existing| existing == &arg) {
            args.push(arg);
        }
    };

    if let Some(sysroot_os) = env::var_os("ARM_NONE_EABI_SYSROOT") {
        let sysroot_path = PathBuf::from(&sysroot_os);
        if sysroot_path.exists() {
            push_sysroot(sysroot_path.as_path());
        }
    }

    if let Some(sysroot) = gcc_query(&["-print-sysroot"]) {
        let sysroot = sysroot.trim();
        if !sysroot.is_empty() {
            push_sysroot(Path::new(sysroot));
        }
    }

    if let Some(include_dir) = gcc_query(&["-print-file-name=include"]) {
        let include_dir = include_dir.trim();
        if !include_dir.is_empty() && include_dir != "include" {
            system_include_paths.insert(PathBuf::from(include_dir));
        }
    }

    if let Some(libgcc) = gcc_query(&["-print-libgcc-file-name"]) {
        let libgcc_path = Path::new(libgcc.trim());
        if let Some(version_dir) = libgcc_path.parent() {
            system_include_paths.insert(version_dir.join("include"));
            system_include_paths.insert(version_dir.join("include-fixed"));

            if let Some(toolchain_root) = version_dir.parent() {
                if let Some(version) = version_dir.file_name().and_then(|name| name.to_str()) {
                    system_include_paths
                        .insert(toolchain_root.join("include").join("c++").join(version));
                    system_include_paths.insert(
                        toolchain_root
                            .join("include")
                            .join("c++")
                            .join(version)
                            .join("arm-none-eabi"),
                    );
                }
            }
        }
    }

    for path in gcc_include_search_paths() {
        system_include_paths.insert(path);
    }

    if let Some(extra) = env::var_os("ARM_NONE_EABI_INCLUDE") {
        for path in env::split_paths(&extra) {
            system_include_paths.insert(path);
        }
    }

    for path in system_include_paths {
        if path.exists() {
            let flag = format!("-isystem{}", path.display());
            if !args.contains(&flag) {
                args.push(flag);
            }
        }
    }

    args
}

fn gcc_include_search_paths() -> Vec<PathBuf> {
    let mut command = Command::new("arm-none-eabi-gcc");
    command.args(["-xc", "-E", "-Wp,-v", "-"]);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => return Vec::new(),
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let stderr = match String::from_utf8(output.stderr) {
        Ok(text) => text,
        Err(_) => return Vec::new(),
    };

    let mut paths = Vec::new();
    let mut capture = false;

    for line in stderr.lines() {
        if line.contains("#include <...> search starts here:") {
            capture = true;
            continue;
        }
        if capture {
            if line.contains("End of search list.") {
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let trimmed = trimmed.trim_start_matches("(framework directory) ");
            let trimmed = trimmed.trim_end_matches(" (framework directory)");
            if trimmed.is_empty() {
                continue;
            }
            let candidate = PathBuf::from(trimmed);
            if candidate.is_relative() {
                continue;
            }
            paths.push(candidate);
        }
    }

    paths
}

fn gcc_query(args: &[&str]) -> Option<String> {
    let mut command = Command::new("arm-none-eabi-gcc");
    for arg in args {
        command.arg(arg);
    }
    command.output().ok().and_then(|output| {
        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_lib_filenames_maps_features_to_archives() {
        let manifest = "\
[features]
default = [\"pac\"]
pac = []
lib_linklayer_ble_full_v3_0 = []
lib_lc3 = []
lib_stm32wb_ot_mtd_lib = []
wba_wpan = []
";
        let expected = expected_lib_filenames(manifest);

        assert!(expected.contains("liblinklayer_ble_full_v3_0.a"));
        assert!(expected.contains("liblc3.a"));
        assert!(expected.contains("libstm32wb_ot_mtd_lib.a"));
        assert_eq!(expected.len(), 3);
    }

    #[test]
    fn expected_lib_filenames_ignores_non_lib_features() {
        let manifest = "\
[features]
default = [\"pac\"]
pac = []
wba_wpan = []
wba_wpan_openthread = []
metadata = []
defmt = [\"dep:defmt\"]
";
        assert!(expected_lib_filenames(manifest).is_empty());
    }

    #[test]
    fn prune_unused_libs_removes_only_unreferenced_archives() {
        let tmp = tempfile::tempdir().unwrap();
        let build_dir = tmp.path();
        let crate_dir = build_dir.join("stm32-bindings");
        let link_layer = crate_dir.join("src/lib/link_layer");
        let openthread = crate_dir.join("src/lib/openthread");

        std::fs::create_dir_all(&link_layer).unwrap();
        std::fs::create_dir_all(&openthread).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            "[features]\nlib_linklayer_ble_full_v3_0 = []\n",
        )
        .unwrap();
        std::fs::write(link_layer.join("liblinklayer_ble_full_v3_0.a"), b"keep").unwrap();
        std::fs::write(link_layer.join("libstm32wba_ot_ftd_lib.a"), b"drop").unwrap();
        std::fs::write(openthread.join("libstm32wba_ot_mtd_lib.a"), b"drop").unwrap();

        let generator = Gen::new(Options {
            build_dir: build_dir.to_path_buf(),
            sources_dir: build_dir.to_path_buf(),
            prune_unused_libs: true,
        });
        generator.prune_unused_libs();

        assert!(link_layer.join("liblinklayer_ble_full_v3_0.a").exists());
        assert!(!link_layer.join("libstm32wba_ot_ftd_lib.a").exists());
        assert!(!openthread.join("libstm32wba_ot_mtd_lib.a").exists());
        assert!(!openthread.exists(), "empty directories should be removed");
    }
}
