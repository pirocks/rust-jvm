use jvmti_jni_bindings::{jfloat, jint};

pub fn fcmpl(value2: jfloat, value1: jfloat) -> jint{
    if value1.is_nan() || value2.is_nan() {
        return -1;
    }
    fcmp_common(value2, value1)
}

pub fn fcmpg(value2: jfloat, value1: jfloat) -> jint {
    if value1.is_nan() || value2.is_nan() {
        return 1;
    }
    fcmp_common(value2, value1)
}

fn fcmp_common(value2: f32, value1: f32) -> jint {
    if value1.to_bits() == value2.to_bits() {
        return 0
    } else if value1 > value2 {
        return 1
    } else if value1 < value2 {
        return -1
    } else { panic!() }
}
