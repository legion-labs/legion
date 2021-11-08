use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    process::Command,
};

static GRAPHICS_SANDBOX_CLI_EXE: &str = env!("CARGO_BIN_EXE_graphics-sandbox");
static GRAPHICS_SANDBOX_TEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn generate_image(wd: &Path, setup_name: &str) {
    let args = &["--snapshot", "--setup-name", setup_name];
    println!("{} {}", GRAPHICS_SANDBOX_CLI_EXE, args.join(" "));
    let status = Command::new(GRAPHICS_SANDBOX_CLI_EXE)
        .current_dir(wd)
        .args(args)
        .envs(std::env::vars())
        .status()
        .expect("failed to execute command");
    assert!(status.success());
}

fn init_test_dir(test_name: &str) -> PathBuf {
    let path = Path::new(GRAPHICS_SANDBOX_CLI_EXE)
        .parent()
        .unwrap()
        .join("graphics-tests-scratch")
        .join(test_name);
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}

#[derive(Debug, PartialEq)]
struct SnapshotData {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

/// Load the image using `png`
fn load_image(path: &Path) -> io::Result<SnapshotData> {
    use png::ColorType::*;
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info()?;
    let mut img_data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut img_data)?;

    match info.color_type {
        Rgba => Ok(SnapshotData {
            data: img_data,
            width: info.width,
            height: info.height,
        }),
        _ => unreachable!("uncovered color type"),
    }
}

#[ignore = "GPU"]
#[test]
fn triangle_render() {
    let test_name = "triangle_render";
    let setup_name = "triangle_render";
    let wd = init_test_dir(test_name);
    generate_image(&wd, setup_name);
    let snapshot = load_image(&wd.join(&format!("{}.png", setup_name))).unwrap();
    let ref_path = Path::new(GRAPHICS_SANDBOX_TEST_DIR)
        .join("tests")
        .join("refs")
        .join(&format!("{}_{}.png", test_name, setup_name));
    let ref_snapshot = load_image(&ref_path).unwrap();
    assert_eq!(snapshot, ref_snapshot);
}
