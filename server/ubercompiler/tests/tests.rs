use lgn_data_compiler::{compiler_api::CompilationEnv, compiler_cmd, Locale, Platform, Target};

static UBERCOMPILER_EXE: &str = env!("CARGO_BIN_EXE_compiler-ubercompiler");

#[test]
fn test() {
    // list all compiler info.
    let info_output = {
        let info_cmd = lgn_data_compiler::compiler_cmd::CompilerInfoCmd::default();
        let info_output = info_cmd
            .execute(UBERCOMPILER_EXE)
            .expect("valid output")
            .take();

        assert!(info_output.len() > 1);
        info_output
    };

    let env = CompilationEnv {
        target: Target::Game,
        platform: Platform::Windows,
        locale: Locale::new("en"),
    };

    // get hashes for all compilers
    {
        let all_hash_cmd = compiler_cmd::CompilerHashCmd::new(&env, None);

        let all_hash_output = all_hash_cmd
            .execute(UBERCOMPILER_EXE)
            .expect("valid output");

        assert!(all_hash_output.compiler_hash_list.len() >= info_output.len());
    }

    // get hash of a single selected transform
    {
        let selected_transform = info_output[0].transform;
        let single_hash_cmd = compiler_cmd::CompilerHashCmd::new(&env, Some(selected_transform));
        let single_hash_output = single_hash_cmd
            .execute(UBERCOMPILER_EXE)
            .expect("valid output");

        assert!(!single_hash_output.compiler_hash_list.is_empty());
        for (transform, _) in single_hash_output.compiler_hash_list {
            assert_eq!(transform, selected_transform);
        }
    }
}
