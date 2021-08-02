#[cfg(test)]
mod tests {
    use core::slice;
    use std::{
        env,
        fs::File,
        path::{Path, PathBuf},
    };

    use data_compiler::{
        compiled_asset_store::{
            CompiledAssetStore, CompiledAssetStoreAddr, LocalCompiledAssetStore,
        },
        compiler_cmd::{list_compilers, CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd},
        Locale, Platform, Target,
    };
    use resources::{test_resource, ResourceId, ResourceProcessor, RESOURCE_EXT};

    fn target_dir() -> PathBuf {
        env::current_exe()
            .ok()
            .map(|mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            })
            .unwrap_or_else(|| panic!("cannot find test directory"))
    }

    fn compiler_exe(name: &str) -> PathBuf {
        target_dir().join(format!("compiler-{}{}", name, env::consts::EXE_SUFFIX))
    }

    #[test]
    fn find_compiler() {
        let exe_path = compiler_exe("test");
        assert!(exe_path.exists());

        let compilers = list_compilers(slice::from_ref(&target_dir()));
        assert_ne!(compilers.len(), 0);
    }

    #[test]
    fn info_command() {
        let exe_path = compiler_exe("test");
        assert!(exe_path.exists());

        let command = CompilerInfoCmd::default();
        let _info = command.execute(&exe_path).expect("info output");
    }

    #[test]
    fn compiler_hash_command() {
        let exe_path = compiler_exe("test");
        assert!(exe_path.exists());

        let command = CompilerHashCmd::new(Target::Game, Platform::Windows, &Locale::new("en"));
        let _hashes = command.execute(&exe_path).expect("hash list");
    }

    fn create_test_resource(id: ResourceId, dir: &Path, content: &str) {
        let path = dir.join(format!("{:x}.{}", id, RESOURCE_EXT));
        let mut file = File::create(path).expect("new file");

        let mut proc = test_resource::TestResourceProc {};
        let mut resource = proc.new_resource();

        resource
            .as_any_mut()
            .downcast_mut::<test_resource::TestResource>()
            .unwrap()
            .content = String::from(content);
        proc.write_resource(resource.as_ref(), &mut file)
            .expect("write to file");
    }

    #[test]
    fn compile_command() {
        let work_dir = tempfile::tempdir().unwrap();
        let resource_dir = work_dir.path();

        let content = "test content";

        let source = ResourceId::generate_new(test_resource::TYPE_ID);
        create_test_resource(source, resource_dir, content);

        let exe_path = compiler_exe("test");
        assert!(exe_path.exists());

        let cas_addr = CompiledAssetStoreAddr::from(resource_dir.to_owned());

        let mut command = CompilerCompileCmd::new(
            source,
            &[],
            &cas_addr,
            &resource_dir,
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        );

        let result = command
            .execute(&exe_path, resource_dir)
            .expect("compile result");
        println!("{:?}", result);

        assert_eq!(result.compiled_assets.len(), 1);

        let asset_checksum = result.compiled_assets[0].checksum;

        let cas = LocalCompiledAssetStore::new(cas_addr).expect("valid cas");
        assert!(cas.exists(asset_checksum));

        let asset_content = cas.read(asset_checksum).expect("asset content");
        let mut reversed = content.as_bytes().to_owned();
        reversed.reverse();
        assert_eq!(&asset_content[..], &reversed[..]);
    }
}
