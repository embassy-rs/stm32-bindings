use std::path::{Path, PathBuf};
use std::{env, fs, io};

fn add_dir(src: &Path) -> io::Result<()> {
    println!("cargo:rustc-link-search=native={}", src.to_str().unwrap());

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            add_dir(&path)?;
        }
    }
    Ok(())
}

fn main() {
    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = crate_dir.join("src").join("lib");

    add_dir(&lib_dir).unwrap();

    env::vars()
        .filter_map(|(a, _)| a.strip_prefix("CARGO_FEATURE_LIB_").map(|a| a.to_string()))
        .map(|a| a.to_ascii_lowercase())
        .for_each(|a| println!("cargo:rustc-link-lib=static={}", a));
}
