//! Common space for utils used across crates 

/// Non-zero exit code indicates a program error
pub fn exit_program(code: i32) -> ! {
    std::process::exit(code)
}