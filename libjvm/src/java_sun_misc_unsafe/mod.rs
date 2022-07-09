use std::ffi::c_void;
use std::intrinsics::{transmute, volatile_load};
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::{null, null_mut};

use nix::convert_ioctl_res;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jboolean, jbyte, jclass, jint, jlong, JNIEnv, jobject, JVM_CALLER_DEPTH};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::FieldId;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::interpreter_util::new_object;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::runtime_class::static_vars;
use slow_interpreter::rust_jni::interface::get_field::new_field_id;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, from_object_new, get_interpreter_state, get_state, to_object, to_object_new};
use slow_interpreter::utils::throw_npe;

use crate::introspection::JVM_GetCallerClass;

pub mod compare_and_swap;
pub mod defineAnonymousClass;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(env: *mut JNIEnv, cb: jclass) {
    //todo for now register nothing, register later as needed.
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(env: *mut JNIEnv, obj: jobject, cb: jclass) -> jint {
    size_of::<jlong>() as jint
    //unimplemented but can't return nothing.
    //essentially the amount at the beginning of the array which is reserved
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldBase(env: *mut JNIEnv, field: jobject) -> jobject {
    null_mut()
    //unimplemented but can't return nothing.
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(env: *mut JNIEnv, obj: jobject, cb: jclass) -> jint {
    size_of::<jlong>() as jint
    //todo unimplemented but can't return nothing, and need to return a power of 2,1 counts as a power of two. This essentially reprs the size of an elem in java arrays
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize(env: *mut JNIEnv, obj: jobject) -> jint {
    64
    //officially speaking unimplemented but can't return nothing, and should maybe return something reasonable todo
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_allocateInstance(env: *mut JNIEnv, the_unsafe: jobject, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let jclass = from_jclass(jvm, cls);
    let rc = check_initing_or_inited_class(jvm, int_state, jclass.as_type(jvm)).unwrap();
    let obj_handle = new_object(jvm, int_state, &rc);
    to_object_new(Some(obj_handle.as_allocated_obj()))
}

#[no_mangle]
unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass(env: *mut JNIEnv, cb: jclass) -> jclass {
    JVM_GetCallerClass(env, JVM_CALLER_DEPTH)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory(env: *mut JNIEnv, the_unsafe: jobject, src_obj: jobject, offset: jlong, dst_obj: jobject, address: jlong, len: jlong) {
    todo!("update for new offset")
    /*let jvm = get_state(env);
    let nonnull = match from_object(jvm, src_obj) {
        Some(x) => x,
        None => return throw_npe(get_state(env), get_interpreter_state(env)),
    };
    let as_array = nonnull.unwrap_array(); //not defined for non-byte-array objects
    assert_eq!(as_array.elem_type, CPDType::ByteType);
    let array_mut = as_array;
    let src_slice_indices = offset..(offset + len);
    let mut src_buffer: Vec<i8> = vec![];
    for i in src_slice_indices {
        src_buffer.push(array_mut.get_i(jvm, i as i32).unwrap_byte());
    }
    assert_eq!(dst_obj, null_mut());
    libc::memcpy(transmute(address), src_buffer.as_ptr() as *const libc::c_void, len as usize);*/
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__JB(env: *mut JNIEnv, the_unsafe: jobject, address: jlong, byte_: jbyte) {
    let byte_addr: *mut jbyte = transmute(address);
    byte_addr.write(byte_);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__JI(env: *mut JNIEnv, the_unsafe: jobject, address: jlong, int_: jint) {
    let int_addr: *mut jint = transmute(address);
    int_addr.write(int_)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_objectFieldOffset(env: *mut JNIEnv, the_unsafe: jobject, field_obj: jobject) -> jlong {
    let jvm = get_state(env);
    let jfield = NewJavaValueHandle::Object(from_object_new(jvm, field_obj).unwrap()).cast_field();
    let name = FieldName(jvm.string_pool.add_name(jfield.name(jvm).to_rust_string(jvm), false));
    let clazz = jfield.clazz(jvm).gc_lifeify().as_runtime_class(jvm);
    let class_view = clazz.view();
    let field = match class_view.lookup_field(name) {
        Some(x) => x,
        None => todo!(),
    };
    let field_numbers = &clazz.unwrap_class_class().field_numbers;
    let field_number = field_numbers[&name].number;
    let res = field_number.0 as jlong * size_of::<jlong>() as jlong;
    res
    /*class_view.fields().enumerate().for_each(|(i, f)| {
        if f.field_name() == name {
            field_i = Some(i);
        }
    });
    let jvm = get_state(env);
    let field_id = new_field_id(jvm, clazz, field_i.unwrap());
    field_id as jlong*/
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldOffset(env: *mut JNIEnv, the_unsafe: jobject, field_obj: jobject) -> jlong {
    //todo major duplication
    let jvm = get_state(env);
    let jfield = NewJavaValueHandle::Object(from_object_new(jvm, field_obj).unwrap()).cast_field();
    let name = FieldName(jvm.string_pool.add_name(jfield.name(jvm).to_rust_string(jvm), false));
    let clazz = jfield.clazz(jvm).gc_lifeify().as_runtime_class(jvm);
    let class_view = clazz.view();
    let mut field_i = None;
    class_view.fields().enumerate().for_each(|(i, f)| {
        if f.field_name() == name && f.is_static() {
            field_i = Some(i);
        }
    });
    let jvm = get_state(env);
    let field_id = new_field_id(jvm, clazz, field_i.unwrap());
    field_id as jlong
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getIntVolatile(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match from_object_new(jvm, obj) {
        Some(notnull) => {
            return volatile_load(obj.offset(offset as isize) as *const jint);
            /*let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
            let field_name = rc.view().field(field_i as usize).field_name();
            notnull.as_allocated_obj().get_var_top_level(jvm, field_name).as_njv().unwrap_int()*/
        }
        None => {
            //static
            let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
            let field_name = rc.view().field(field_i as usize).field_name();
            let static_vars = static_vars(rc.deref(), jvm);
            static_vars.get(field_name).to_jv().unwrap_int()
        }
    }
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_allocateMemory(env: *mut JNIEnv, the_unsafe: jobject, len: jlong) -> jlong {
    let res: i64 = libc::malloc(len as usize) as i64;
    res
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__JJ(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong, val: jlong) {
    let ptr: *mut i64 = transmute(ptr);
    ptr.write(val);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__J(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) -> jbyte {
    let ptr: *mut i8 = transmute(ptr);
    ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__J(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) -> i32 {
    let ptr: *mut i32 = transmute(ptr);
    ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__J(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) -> i64 {
    let ptr: *mut i64 = transmute(ptr);
    ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getLongVolatile(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jlong {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match from_object_new(jvm, obj) {
        Some(notnull) => {
            return volatile_load(obj.offset(offset as isize) as *const jlong);
            /*let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
            let field_name = rc.view().field(field_i as usize).field_name();
            notnull.as_allocated_obj().get_var_top_level(jvm, field_name).as_njv().unwrap_int()*/
        }
        None => {
            //static
            let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
            let field_name = rc.view().field(field_i as usize).field_name();
            let static_vars = static_vars(rc.deref(), jvm);
            static_vars.get(field_name).to_jv().unwrap_long()
        }
    }
    /*Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J(env, the_unsafe, obj, offset)*/
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jlong {
    Java_sun_misc_Unsafe_getLongVolatile(env, the_unsafe, obj, offset)
    // todo!("update for new offset")
    /*let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    //todo major dupe with getIntVolatile
    match from_object(jvm, obj) {
        Some(notnull) => {
            let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
            let field_name = rc.view().field(field_i as usize).field_name();
            notnull.unwrap_normal_object().get_var(jvm, rc, field_name).unwrap_long()
        }
        None => {
            //static
            let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
            let field_name = rc.view().field(field_i as usize).field_name();
            let static_vars = rc.static_vars(jvm);
            static_vars.get(field_name).to_jv().unwrap_long()
        }
    }*/
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_freeMemory(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) {
    libc::free(transmute(ptr))
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getObjectVolatile(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, field_id_and_array_idx: jlong) -> jobject {
    let jvm = get_state(env);
    match from_object_new(jvm, obj) {
        None => {
            let field_id = field_id_and_array_idx as FieldId;
            let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
            let runtime_class_view = runtime_class.view();
            let field_view = runtime_class_view.field(i as usize);
            assert!(field_view.is_static());
            let name = field_view.field_name();
            let res = static_vars(runtime_class.deref(), jvm).get(name);
            to_object_new(res.as_njv().unwrap_object_alloc())
        }
        Some(object_to_read) => {
            let offseted = object_to_read.ptr().as_ptr().offset(field_id_and_array_idx as isize) as *mut c_void;
            (offseted as *mut jobject).read_volatile()
            /*if !object_to_read.is_array(jvm) {
                todo!()
            }
            let arr = object_to_read.unwrap_array(jvm);
            let array_idx = field_id_and_array_idx;
            let res = arr.get_i(array_idx as i32 as usize);
            to_object_new(res.unwrap_object().as_ref().map(|handle|handle.as_allocated_obj()))*/
        }
    }
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putObjectVolatile(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, to_put: jobject) {
    let jvm = get_state(env);
    match from_object_new(jvm, obj) {
        None => {
            let field_id = offset as FieldId;
            let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
            let runtime_class_view = runtime_class.view();
            let field_view = runtime_class_view.field(i as usize);
            assert!(field_view.is_static());
            let name = field_view.field_name();
            let mut static_vars_guard = static_vars(runtime_class.deref(), jvm);
            let mut res = static_vars_guard.get(name);
            res = NewJavaValueHandle::from_optional_object(from_object_new(jvm, to_put)); //todo dup with get function
            static_vars_guard.set(name, res)
        }
        Some(object_to_read) => {
            let offseted = object_to_read.ptr().as_ptr().offset(offset as isize) as *mut c_void;
            (offseted as *mut jobject).write_volatile(to_put);
            /*if object_to_read.is_array(jvm){
                let array_idx = offset;
                object_to_read.unwrap_array(jvm).set_i(array_idx as i32 as usize, NewJavaValueHandle::from_optional_object(from_object_new(jvm, to_put)).as_njv());
            }else {
                let field_id = offset as FieldId;
                let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
                let runtime_class_view = runtime_class.view();
                let field_view = runtime_class_view.field(i as usize);
                assert!(!field_view.is_static());
                let name = field_view.field_name();
                let top_level_rc = object_to_read.as_allocated_obj().runtime_class(jvm);
                object_to_read.as_allocated_obj().set_var(&top_level_rc,name, NewJavaValueHandle::from_optional_object(from_object_new(jvm, to_put)).as_njv());
            }*/
        }
    }
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
unsafe extern "system" fn Java_sun_misc_Unsafe_setMemory(env: *mut JNIEnv, the_unsafe: jobject, o: jobject, offset: jlong, bytes: jlong, value: jbyte) {
    assert_eq!(o, null_mut());

    for i in offset..(offset + bytes) {
        let address = i as *mut jbyte;
        address.write(value)
    }
}

pub mod defineClass;
pub mod park;