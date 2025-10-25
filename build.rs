use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=binaries");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let bin_dir = manifest_dir.join("binaries");

    let mut out = String::new();

    for entry in fs::read_dir(&bin_dir).unwrap() {
        let e = entry.unwrap();
        let path = e.path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let const_name = name.replace(['-', '.', '/'], "_");

            out.push_str(&format!(
                "pub const {}: &[u8] = include_bytes!(r#\"{}\"#);\n",
                const_name.to_uppercase(),
                path.display()
            ));
        }
    }

    println!("cargo:rerun-if-changed=strategies");
    let strategy_dir = manifest_dir.join("strategies");
    out.push_str("\nuse std::collections::HashMap;\n");
    out.push_str("pub fn get_strategies() -> HashMap<&'static str, &'static str> {\n");
    out.push_str("    let mut map = HashMap::new();\n");

    if strategy_dir.exists() {
        for entry in fs::read_dir(&strategy_dir).unwrap() {
            let e = entry.unwrap();
            let path = e.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy();
                out.push_str(&format!(
                    "    map.insert(\"{}\", include_str!(r#\"{}\"#).trim());\n",
                    name,
                    path.display()
                ));
            }
        }
    }

    out.push_str("    map\n}\n");

    fs::write(out_dir.join("embedded_bins.rs"), out).unwrap();
}
