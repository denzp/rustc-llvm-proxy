
use super::SHARED_LIB;

const POSSIBLE_BACKENDS: &[&str] = &[
    "AArch64",
    "AMDGPU",
    "ARM",
    "BPF",
    "Hexagon",
    "Lanai",
    "Mips",
    "MSP430",
    "NVPTX",
    "PowerPC",
    "Sparc",
    "SystemZ",
    "X86",
    "XCore",
];

#[no_mangle]
pub unsafe extern "C" fn LLVM_InitializeAllTargets() {
    for backend in POSSIBLE_BACKENDS {
        let name = format!("LLVMInitialize{}Target", backend);
        if let Ok(entrypoint) = SHARED_LIB.get::<unsafe extern "C" fn()>(name.as_bytes()) {
            entrypoint();
        }
    }
}
