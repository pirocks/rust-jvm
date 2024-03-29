use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::field_names::FieldName;

use crate::{AllocatedHandle, JavaValue, JavaValueCommon, JString, JVMState, NewJavaValueHandle};
use crate::class_loading::assert_loaded_class;

pub fn intern_safe<'gc>(jvm: &'gc JVMState<'gc>, str_obj: AllocatedHandle<'gc>) -> JString<'gc> {
    let string_class = assert_loaded_class(jvm, CClassName::string().into());
    let char_array_ptr = match str_obj.unwrap_normal_object_ref().get_var(jvm, &string_class, FieldName::field_value()).unwrap_object() {
        None => {
            eprintln!("Weird malformed string encountered. Not interning.");
            return JavaValue::Object(todo!() /*str_obj.into()*/).cast_string().unwrap();
            //fallback to not interning weird strings like this. not sure if compatible with hotspot but idk what else to do. perhaps throwing an exception would be better idk?
        }
        Some(char_array_ptr) => char_array_ptr,
    };
    let char_array = char_array_ptr.unwrap_array();
    let mut native_string_bytes = Vec::with_capacity(char_array.len() as usize);
    for char_ in char_array.array_iterator() {
        native_string_bytes.push(char_.as_njv().unwrap_char_strict());
    }
    let mut guard = jvm.string_internment.write().unwrap();
    match guard.strings.get(&native_string_bytes) {
        None => {
            guard.strings.insert(native_string_bytes, str_obj.duplicate_discouraged());
            NewJavaValueHandle::Object(str_obj.into()).cast_string().unwrap()
        }
        Some(res) => NewJavaValueHandle::Object(res.duplicate_discouraged()).cast_string().unwrap(),
    }
}

