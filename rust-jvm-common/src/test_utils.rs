use std::path::PathBuf;

pub fn get_test_resources() -> PathBuf {
    let mut test_resources_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_resources_path.push("../resources/test");
    test_resources_path
}