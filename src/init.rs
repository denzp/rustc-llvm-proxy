use super::SHARED_LIB;

use std::io::{BufRead, BufReader, Result};
use std::process::Command;

const POSSIBLE_BACKENDS: &[&str] = &[
    "AArch64", "AMDGPU", "ARM", "BPF", "Hexagon", "Lanai", "Mips", "MSP430", "NVPTX", "PowerPC",
    "Sparc", "SystemZ", "X86", "XCore",
];

fn get_native_arch() -> Result<String> {
    let output = Command::new("rustc").args(&["--print", "cfg"]).output()?;
    let buf = BufReader::new(output.stdout.as_slice());
    for line in buf.lines() {
        let line = line?;
        if !line.starts_with("target_arch") {
            continue;
        }
        // line should be like: target_arch="x86_64"
        return Ok(line.split('"').nth(1).unwrap().into());
    }
    unreachable!("`rustc --print cfg` result is wrong");
}

fn arch2backend(arch: &str) -> String {
    match arch {
        "x86_64" => "X86".into(),
        _ => panic!("Unknown backend: {}", arch), // FIXME
    }
}

fn get_native_backend() -> String {
    let arch = get_native_arch().expect("Fail to get native arch");
    arch2backend(&arch)
}

#[no_mangle]
pub unsafe extern "C" fn LLVM_InitializeAllTargets() {
    for backend in POSSIBLE_BACKENDS {
        let name = format!("LLVMInitialize{}Target", backend);
        if let Ok(entrypoint) = SHARED_LIB.get::<unsafe extern "C" fn()>(name.as_bytes()) {
            entrypoint();
        }
    }
}
