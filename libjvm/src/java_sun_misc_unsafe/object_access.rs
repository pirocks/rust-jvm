use std::intrinsics::volatile_load;
use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};

use libc::{c_void, initgroups};
use better_nonnull::BetterNonNull;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jclass, jint, jlong, JNIEnv, jobject};
use runtime_class_stuff::accessor::Accessor;
use runtime_class_stuff::array_layout::ArrayMemoryLayout;
use runtime_class_stuff::field_numbers::FieldNameAndClass;
use runtime_class_stuff::object_layout::{FieldAccessor, ObjectLayout};
use rust_jvm_common::{FieldId};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::global_consts::ADDRESS_SIZE;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::allocated_objects::AllocatedHandle;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object_new, to_object_new};
use slow_interpreter::static_vars::static_vars;
use slow_interpreter::utils::new_field_id;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(env: *mut JNIEnv, cb: jclass) {
    //todo for now register nothing, register later as needed.
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(env: *mut JNIEnv, obj: jobject, cb: jclass) -> jint {
    let jvm = get_state(env);
    let runtime_class = from_jclass(jvm, cb).as_runtime_class(jvm);
    assert!(runtime_class.cpdtype().is_array());
    let memory_layout = ArrayMemoryLayout::from_cpdtype(runtime_class.cpdtype().unwrap_array_type());
    memory_layout.elem_0_entry_offset() as jint
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldBase(env: *mut JNIEnv, field: jobject) -> jobject {
    null_mut()
    //unimplemented but can't return nothing.
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(env: *mut JNIEnv, obj: jobject, cb: jclass) -> jint {
    let jvm = get_state(env);
    let runtime_class = from_jclass(jvm, cb).as_runtime_class(jvm);
    assert!(runtime_class.cpdtype().is_array());
    let memory_layout = ArrayMemoryLayout::from_cpdtype(runtime_class.cpdtype().unwrap_array_type());
    memory_layout.elem_size() as jint
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize(env: *mut JNIEnv, obj: jobject) -> jint {
    ADDRESS_SIZE
    //officially speaking unimplemented but can't return nothing, and should maybe return something reasonable todo
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_objectFieldOffset(env: *mut JNIEnv, the_unsafe: jobject, field_obj: jobject) -> jlong {
    let jvm = get_state(env);
    let jfield = NewJavaValueHandle::Object(from_object_new(jvm, field_obj).unwrap()).cast_field();
    let field_name = FieldName(jvm.string_pool.add_name(jfield.name(jvm).to_rust_string(jvm), false));
    let clazz = jfield.clazz(jvm).gc_lifeify().as_runtime_class(jvm);
    let class_view = clazz.view();
    let field = match class_view.lookup_field(field_name) {
        Some(x) => x,
        None => {
            dbg!(field_name.0.to_str(&jvm.string_pool));
            get_interpreter_state(env).debug_print_stack_trace(jvm);
            todo!()
        }
    };
    let field_numbers = &clazz.unwrap_class_class().object_layout.field_numbers;
    let class_name = class_view.name().unwrap_name();
    let field_number = field_numbers[&FieldNameAndClass { field_name, class_name }].number;
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
            return volatile_load((obj as *const c_void).offset(offset as isize) as *const jint);
        }
        None => {
            //static
            let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
            let field_name = rc.view().field(field_i as usize).field_name();
            let static_vars = static_vars(rc.deref(), jvm);
            static_vars.get(field_name, CPDType::IntType).unwrap_int()
        }
    }
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jint {
    Java_sun_misc_Unsafe_getIntVolatile(env, the_unsafe, obj, offset)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__Ljava_lang_Object_2JJ(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, long_: jlong) {
    let jvm = get_state(env);
    let obj_option = from_object_new(jvm, obj);
    todo!("should be intrinsic")
    /*putVolatileImpl(offset, NativeJavaValue { long: long_ }, jvm, obj_option);*/
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
            static_vars.get(field_name, CPDType::LongType).to_jv().unwrap_long()
        }
    }
    /*Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J(env, the_unsafe, obj, offset)*/
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
unsafe extern "system" fn Java_sun_misc_Unsafe_putIntVolatile(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jint) {
    let jvm = get_state(env);
    let obj_option = from_object_new(jvm, obj);
    todo!("should be intrinsic")
    /*putVolatileImpl(offset, NativeJavaValue { int: val }, jvm, obj_option);*/
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getObjectVolatile(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jobject {
    let jvm = get_state(env);
    match from_object_new(jvm, obj) {
        None => {
            let field_id = offset as FieldId;
            let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
            let runtime_class_view = runtime_class.view();
            let field_view = runtime_class_view.field(i as usize);
            assert!(field_view.is_static());
            let name = field_view.field_name();
            let res = static_vars(runtime_class.deref(), jvm).get(name, CPDType::object());
            to_object_new(res.as_njv().unwrap_object_alloc())
        }
        Some(object_to_read) => {
            let field_address = BetterNonNull::new(obj as *mut c_void).unwrap().offset(offset as isize).unwrap().0;
            FieldAccessor::new(field_address, CPDType::object()).read_object()
            // let offseted = object_to_read.ptr().as_ptr().offset(field_id_and_array_idx as isize) as *mut c_void;
            // (offseted as *mut jobject).read_volatile()
        }
    }
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putObjectVolatile(env: *mut JNIEnv, the_unsafe: jobject, obj_to_write: jobject, offset: jlong, to_put: jobject) {
    let jvm = get_state(env);
    let field_address = BetterNonNull::new(obj_to_write as *mut c_void).unwrap().offset(offset as isize).unwrap().0;
    FieldAccessor::new(field_address, CPDType::object()).write_object(to_put)

    // let obj_option = from_object_new(jvm, obj);
    // todo!("this should be a intrinsic anyway")
    // putVolatileImpl(offset, /*NativeJavaValue { object: to_put as *mut c_void }*/, jvm, obj_option);
}
//
// unsafe fn putVolatileImpl<'gc>(offset: jlong, to_put: NativeJavaValue<'gc>, jvm: &'gc JVMState<'gc>, obj_option: Option<AllocatedHandle<'gc>>) {
//     match obj_option {
//         None => {
//             let field_id = offset as FieldId;
//             let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
//             let runtime_class_view = runtime_class.view();
//             let field_view = runtime_class_view.field(i as usize);
//             assert!(field_view.is_static());
//             let name = field_view.field_name();
//             let mut static_vars_guard = static_vars(runtime_class.deref(), jvm);
//             // static_vars_guard.set_raw(name, to_put).unwrap();
//             todo!("this should be an intrinsic anyway")
//         }
//         Some(object_to_read) => {
//             let offseted = object_to_read.ptr().as_ptr().offset(offset as isize) as *mut c_void;
//             (offseted as *mut NativeJavaValue<'gc>).write_volatile(to_put);
//         }
//     }
// }
