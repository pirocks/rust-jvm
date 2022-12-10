use std::mem::size_of;
use std::ptr::null_mut;
use array_memory_layout::layout::ArrayMemoryLayout;
use jvmti_jni_bindings::{jclass, jint, jlong, JNIEnv, jobject};
use runtime_class_stuff::field_numbers::FieldNameAndClass;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::global_consts::ADDRESS_SIZE;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::get_state;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object_new};

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(env: *mut JNIEnv, _obj: jobject, cb: jclass) -> jint {
    let jvm = get_state(env);
    let runtime_class = from_jclass(jvm, cb).as_runtime_class(jvm);
    assert!(runtime_class.cpdtype().is_array());
    let memory_layout = ArrayMemoryLayout::from_cpdtype(runtime_class.cpdtype().unwrap_array_type());
    memory_layout.elem_0_entry_offset() as jint
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldBase(_env: *mut JNIEnv, _field: jobject) -> jobject {
    null_mut()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(env: *mut JNIEnv, _obj: jobject, cb: jclass) -> jint {
    let jvm = get_state(env);
    let runtime_class = from_jclass(jvm, cb).as_runtime_class(jvm);
    assert!(runtime_class.cpdtype().is_array());
    let memory_layout = ArrayMemoryLayout::from_cpdtype(runtime_class.cpdtype().unwrap_array_type());
    memory_layout.elem_size().get() as jint
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize(_env: *mut JNIEnv, _obj: jobject) -> jint {
    ADDRESS_SIZE
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_objectFieldOffset(env: *mut JNIEnv, _the_unsafe: jobject, field_obj: jobject) -> jlong {
    let jvm = get_state(env);
    let jfield = NewJavaValueHandle::Object(from_object_new(jvm, field_obj).unwrap()).cast_field();
    let field_name = FieldName(jvm.string_pool.add_name(jfield.name(jvm).to_rust_string(jvm), false));
    let clazz = jfield.clazz(jvm).gc_lifeify().as_runtime_class(jvm);
    let class_view = clazz.view();
    let field_numbers = &clazz.unwrap_class_class().object_layout.field_numbers;
    let class_name = class_view.name().unwrap_name();
    let field_number = field_numbers[&FieldNameAndClass { field_name, class_name }].number;
    let res = field_number.0 as jlong * size_of::<jlong>() as jlong;
    res
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldOffset(env: *mut JNIEnv, _the_unsafe: jobject, field_obj: jobject) -> jlong {
    //todo major duplication
    let jvm = get_state(env);
    let jfield = NewJavaValueHandle::Object(from_object_new(jvm, field_obj).unwrap()).cast_field();
    let name = FieldName(jvm.string_pool.add_name(jfield.name(jvm).to_rust_string(jvm), false));
    let clazz = jfield.clazz(jvm);
    let class_name = clazz.as_type(jvm).unwrap_class_type();
    let static_field = jvm.all_the_static_fields.get(FieldNameAndClass { field_name: name, class_name });
    static_field.raw_address().as_ptr() as jlong
}
