use std::process::Command;
use std::env::{current_dir, var};


const DEFAULT_LIBJAVA_LOCATION: &'static str = "/home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/jdk/lib/amd64/libjava.so";
const DEFAULT_RT_JAR_LOCATION: &'static str = "/home/francis/Desktop/jdk8u232-b09/jre/lib/rt.jar";

#[test]
fn float_double_arithmetic() {
    run_integration_test("FloatDoubleArithmetic");
}

fn run_integration_test(class_name: &str) {
    let libjava_path = var("LIBJAVAPATH")
        .unwrap_or(DEFAULT_LIBJAVA_LOCATION.to_string());
    let rt_jar_location = var("RTJAR")
        .unwrap_or(DEFAULT_RT_JAR_LOCATION.to_string());
    let mut resources_path = current_dir().unwrap();
    resources_path.push("resources/test");
    //todo presumably theres a better way of getting exe location

    let mut java_process = Command::new("target/debug/java")
        .arg("--main").arg(class_name)
        .arg("--libjava").arg(libjava_path)
        .arg("--args").arg("args_do_not_work_yet_so...")
        .arg("--jars").arg(rt_jar_location)
        .arg("--classpath").arg(resources_path.as_path().to_str().unwrap())
        .spawn().unwrap();
    assert!(java_process.wait().unwrap().success());
}


#[test]
fn method_introspection_reflection_demo() {
    run_integration_test("MethodIntrospectionReflectionDemo");
}

#[test]
fn empty_main() {
    run_integration_test("EmptyMain");
}


#[test]
fn io_examples() {
    run_integration_test("IOExamples");
}