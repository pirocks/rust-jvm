use std::mem::{size_of};
use std::ptr::null_mut;

use libc::c_void;

use array_memory_layout::accessor::Accessor;
use array_memory_layout::layout::ArrayMemoryLayout;
use better_nonnull::BetterNonNull;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jshort};
use runtime_class_stuff::field_numbers::FieldNameAndClass;
use runtime_class_stuff::object_layout::FieldAccessor;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::global_consts::ADDRESS_SIZE;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_state};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object_new};

use crate::double_register_addressing::calc_address;

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(_env: *mut JNIEnv, _cb: jclass) {
    //todo for now register nothing, register later as needed.
}

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

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getIntVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jint {
    calc_address(obj, offset).cast::<jint>().read()
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getBooleanVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jboolean {
    calc_address(obj, offset).cast::<jboolean>().read()
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getCharVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jchar {
    calc_address(obj, offset).cast::<jchar>().read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getByteVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jbyte {
    calc_address(obj, offset).cast::<jbyte>().read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getShortVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jshort {
    calc_address(obj, offset).cast::<jshort>().read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getFloatVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jfloat {
    calc_address(obj, offset).cast::<jfloat>().read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jint {
    Java_sun_misc_Unsafe_getIntVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jfloat {
    Java_sun_misc_Unsafe_getFloatVolatile(env, the_unsafe, obj, offset)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__Ljava_lang_Object_2JJ(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, long_: jlong) {
    Java_sun_misc_Unsafe_putLongVolatile(env, the_unsafe, obj, offset, long_)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putObject(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, to_put: jobject) {
    Java_sun_misc_Unsafe_putObjectVolatile(env, the_unsafe, obj, offset, to_put)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedObject(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, to_put: jobject) {
    Java_sun_misc_Unsafe_putObjectVolatile(env, the_unsafe, obj, offset, to_put)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getLongVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jlong {
    calc_address(obj, offset).cast::<jlong>().read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jlong {
    Java_sun_misc_Unsafe_getLongVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__Ljava_lang_Object_2JI(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jint) {
    Java_sun_misc_Unsafe_putIntVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__Ljava_lang_Object_2JB(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jbyte) {
    Java_sun_misc_Unsafe_putByteVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__Ljava_lang_Object_2JB(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jshort) {
    Java_sun_misc_Unsafe_putShortVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putFloat__Ljava_lang_Object_2JF(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jfloat) {
    Java_sun_misc_Unsafe_putFloatVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getDouble__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jdouble) {
    Java_sun_misc_Unsafe_putDoubleVolatile(env, the_unsafe, obj, offset, val)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jshort) {
    Java_sun_misc_Unsafe_putShortVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jbyte {
    Java_sun_misc_Unsafe_getByteVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putIntVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jint) {
    obj.cast::<c_void>().offset(offset as isize).cast::<jint>().write(val)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putByteVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jbyte) {
    calc_address(obj, offset).cast::<jbyte>().write(val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putLongVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jlong) {
    calc_address(obj, offset).cast::<jlong>().write(val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putFloatVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jfloat) {
    calc_address(obj, offset).cast::<jfloat>().write(val)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putDoubleVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jdouble) {
    calc_address(obj, offset).cast::<jdouble>().write(val)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putShortVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jshort) {
    calc_address(obj, offset).cast::<jshort>().write(val)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getObjectVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jobject {
    calc_address(obj, offset).cast::<jobject>().read()
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putObjectVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj_to_write: jobject, offset: jlong, to_put: jobject) {
    let field_address = BetterNonNull::new(obj_to_write as *mut c_void).unwrap().offset(offset as isize).unwrap().0;
    FieldAccessor::new(field_address, CPDType::object()).write_object(to_put)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getObject(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jobject {
    Java_sun_misc_Unsafe_getObjectVolatile(env, the_unsafe, obj, offset)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getBoolean(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jboolean {
    Java_sun_misc_Unsafe_getBooleanVolatile(env, the_unsafe, obj, offset)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedInt(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jint) {
    Java_sun_misc_Unsafe_putIntVolatile(env, the_unsafe, obj, offset, val)
}
