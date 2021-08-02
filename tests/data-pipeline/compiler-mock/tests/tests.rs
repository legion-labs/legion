//static COMPILER_EXE: &str = env!("CARGO_BIN_EXE_compiler-mock");

//
// A single compiler test is required to make sure compiler binary is generated
// before running the integration tests in the root "/tests" directory.
// Tests under "/tests/data-pipeline" depends on `compiler-mock`.
//
#[test]
fn force_build_bin() {}
