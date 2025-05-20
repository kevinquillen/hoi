use std::env;
use std::path::Path;

pub fn copy_fixture(name: &str, target_dir: &Path, target_filename: &str) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = Path::new(&manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(name);

    let contents = std::fs::read_to_string(&src_path).expect("Failed to read fixture");

    #[cfg(windows)]
    let contents = contents
        .replace("$ENV_VAR", "%ENV_VAR%")
        .replace("$LOCAL_VAR", "%LOCAL_VAR%")
        .replace("$OVERRIDE_VAR", "%OVERRIDE_VAR%");

    std::fs::write(target_dir.join(target_filename), contents)
        .expect("Failed to write test config");
}