use runtime_common::java_values::JavaValue;

pub fn to_native(j : JavaValue) -> jni::{
    match j {
        JavaValue::Long(_) => {},
        JavaValue::Int(_) => {},
        JavaValue::Short(_) => {},
        JavaValue::Byte(_) => {},
        JavaValue::Boolean(_) => {},
        JavaValue::Char(_) => {},
        JavaValue::Float(_) => {},
        JavaValue::Double(_) => {},
        JavaValue::Array(_) => {},
        JavaValue::Object(_) => {},
        JavaValue::Top => {},
    }
}
