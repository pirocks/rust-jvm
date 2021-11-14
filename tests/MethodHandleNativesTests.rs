use std::env::{current_dir, var};
use std::process::Command;

const DEFAULT_LIBJAVA_LOCATION: &'static str = "/home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/jdk/lib/amd64/libjava.so";
const DEFAULT_RT_JAR_DIR_LOCATION: &'static str = "/home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/images/j2sdk-image/jre/lib";
const DEFAULT_RT_JAR_EXT_DIR_LOCATION: &'static str = "/home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/images/j2sdk-image/jre/lib/ext";

#[test]
fn run_integration_test() {
    let libjava_path = var("LIBJAVAPATH").unwrap_or(DEFAULT_LIBJAVA_LOCATION.to_string());
    let rt_jar_location = var("RTJAR_DIR").unwrap_or(DEFAULT_RT_JAR_DIR_LOCATION.to_string());
    let rt_jar_ext_location =
        var("RTJAR_EXT_DIR").unwrap_or(DEFAULT_RT_JAR_EXT_DIR_LOCATION.to_string());
    let mut resources_path = current_dir().unwrap();
    resources_path.push("resources/test");

    let mut java_process = Command::new("target/debug/java")
        .arg("--unittest-mode")
        .arg("--libjava")
        .arg(libjava_path)
        .arg("--classpath")
        .arg(rt_jar_location)
        .arg(rt_jar_ext_location)
        .arg("--jvmti")
        .arg("--tracing")
        .spawn()
        .unwrap();
    assert!(java_process.wait().unwrap().success());
}
