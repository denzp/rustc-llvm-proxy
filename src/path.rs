use std::env;
use std::path::PathBuf;
use std::process::Command;

use failure::Error;

pub fn find_lib_path() -> Result<PathBuf, Error> {
    let paths = collect_possible_paths()?;

    if paths.is_empty() {
        bail!("Unable to find possible LLVM shared lib locations.");
    }

    for path in &paths {
        if path.join("librustc_codegen_llvm-llvm.so").exists() {
            return Ok(path.join("librustc_codegen_llvm-llvm.so"));
        }

        if path.join("librustc_codegen_llvm-llvm.dylib").exists() {
            return Ok(path.join("librustc_codegen_llvm-llvm.dylib"));
        }

        if path.join("rustc_codegen_llvm-llvm.dll").exists() {
            return Ok(path.join("rustc_codegen_llvm-llvm.dll"));
        }
    }

    bail!(
        "Unable to find LLVM shared lib in possible locations:\n- {}",
        paths
            .into_iter()
            .map(|item| item.to_str().unwrap().to_owned())
            .collect::<Vec<_>>()
            .join("\n- ")
    );
}

fn collect_possible_paths() -> Result<Vec<PathBuf>, Error> {
    let mut paths = vec![];

    // Special case: find the location for Rust built from sources.
    if let Ok(env_path) = env::var("PATH") {
        for item in env_path.split(':') {
            let mut rustc_path = PathBuf::from(item);

            rustc_path.pop();
            paths.push(rustc_path.join("codegen-backends"));
        }
    }

    if let Ok(rustup_home) = env::var("RUSTUP_HOME") {
        let rustup_home = PathBuf::from(rustup_home);
        let rustup_toolchain = env::var("RUSTUP_TOOLCHAIN")?;
        let rustup_arch = extract_arch(&rustup_toolchain);

        paths.push(
            rustup_home
                .join("toolchains")
                .join(&rustup_toolchain)
                .join("lib")
                .join("rustlib")
                .join(rustup_arch)
                .join("codegen-backends"),
        );
    }

    if let Ok(lib_paths) = env::var("LD_LIBRARY_PATH") {
        for item in lib_paths.split(':') {
            let mut possible_path = PathBuf::from(item);
            possible_path.pop();

            if let Some(possible_toolchain) = possible_path.file_name() {
                let possible_arch = extract_arch(possible_toolchain.to_str().unwrap());

                paths.push(
                    possible_path
                        .join("lib")
                        .join("rustlib")
                        .join(possible_arch)
                        .join("codegen-backends"),
                );
            }
        }
    }

    if let Ok(cargo) = env::var("CARGO") {
        let mut cargo_path = PathBuf::from(cargo);
        cargo_path.pop();
        cargo_path.pop();

        if let Some(toolchain) = cargo_path.file_name() {
            let arch = extract_arch(toolchain.to_str().unwrap());

            paths.push(
                cargo_path
                    .join("lib")
                    .join("rustlib")
                    .join(arch)
                    .join("codegen-backends"),
            );
        }
    }

    if let Ok(output) = Command::new("rustup").args(&["which", "rustc"]).output() {
        let mut rustc_path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
        rustc_path.pop();
        rustc_path.pop();

        if let Some(toolchain) = rustc_path.file_name() {
            let arch = extract_arch(toolchain.to_str().unwrap());

            paths.push(
                rustc_path
                    .join("lib")
                    .join("rustlib")
                    .join(arch)
                    .join("codegen-backends"),
            );
        }
    }

    Ok(paths)
}

fn extract_arch(toolchain: &str) -> String {
    toolchain
        .split('-')
        // Skip `nightly` rust version prefix.
        .skip(1)
        // Also skip rust version specification if exists.
        .skip_while(|item| match item.chars().next() {
            None | Some('0'...'9') => true,
            _ => false,
        })
        .collect::<Vec<_>>()
        .join("-")
}
