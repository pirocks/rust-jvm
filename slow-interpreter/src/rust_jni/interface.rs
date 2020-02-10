use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use jni_bindings::{JNINativeInterface_, JNIEnv, jobject, jmethodID, jthrowable, jint, jclass, __va_list_tag, jchar, jsize, jstring, jfieldID, jboolean, jbyteArray, jarray, jbyte, JavaVM, JNIInvokeInterface_, jlong};
use std::mem::transmute;
use std::ffi::{c_void, CStr, VaList};
use crate::rust_jni::{exception_check, register_natives, release_string_utfchars, get_method_id, MethodId};
use crate::rust_jni::native_util::{get_object_class, get_frame, get_state, to_object, from_object};
use crate::rust_jni::string::{release_string_chars, new_string_utf, get_string_utfchars, new_string_with_string};
use crate::instructions::invoke::{invoke_static_impl, invoke_virtual_method_i, invoke_special, invoke_special_impl};
use rust_jvm_common::classfile::ACC_STATIC;
use classfile_parser::types::{parse_method_descriptor, MethodDescriptor};
use rust_jvm_common::unified_types::ParsedType;
use runtime_common::java_values::{JavaValue, Object, ArrayObject};
use log::trace;
use crate::instructions::ldc::load_class_constant_by_name;
use std::sync::Arc;
use runtime_common::runtime_class::RuntimeClass;
use crate::interpreter_util::{check_inited_class, push_new_object};
use std::ops::{Deref, DerefMut};
use std::cell::RefCell;

//GetFieldID
pub fn get_interface(state: &InterpreterState, frame: Rc<StackEntry>) -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: {
            let boxed = Box::new(frame);
            Box::into_raw(boxed) as *mut c_void
        },
        reserved2: std::ptr::null_mut(),
        reserved3: std::ptr::null_mut(),
        GetVersion: None,
        DefineClass: None,
        FindClass: Some(find_class),
        FromReflectedMethod: None,
        FromReflectedField: None,
        ToReflectedMethod: None,
        GetSuperclass: Some(get_superclass),
        IsAssignableFrom: Some(is_assignable_from),
        ToReflectedField: None,
        Throw: None,
        ThrowNew: None,
        ExceptionOccurred: Some(exception_occured),
        ExceptionDescribe: None,
        ExceptionClear: None,
        FatalError: None,
        PushLocalFrame: None,
        PopLocalFrame: None,
        NewGlobalRef: Some(new_global_ref),
        DeleteGlobalRef: None,
        DeleteLocalRef: Some(delete_local_ref),
        IsSameObject: None,
        NewLocalRef: None,
        EnsureLocalCapacity: Some(ensure_local_capacity),
        AllocObject: None,
        NewObject: Some(unsafe { transmute::<_, unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jobject>(new_object as *mut c_void) }),
        NewObjectV: None,
        NewObjectA: None,
        GetObjectClass: Some(get_object_class),
        IsInstanceOf: None,
        GetMethodID: Some(get_method_id),
        CallObjectMethod: Some(unsafe { transmute::<_, unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jobject>(call_object_method as *mut c_void) }),
        CallObjectMethodV: None,
        CallObjectMethodA: None,
        CallBooleanMethod: None,
        CallBooleanMethodV: None,
        CallBooleanMethodA: None,
        CallByteMethod: None,
        CallByteMethodV: None,
        CallByteMethodA: None,
        CallCharMethod: None,
        CallCharMethodV: None,
        CallCharMethodA: None,
        CallShortMethod: None,
        CallShortMethodV: None,
        CallShortMethodA: None,
        CallIntMethod: None,
        CallIntMethodV: None,
        CallIntMethodA: None,
        CallLongMethod: None,
        CallLongMethodV: None,
        CallLongMethodA: None,
        CallFloatMethod: None,
        CallFloatMethodV: None,
        CallFloatMethodA: None,
        CallDoubleMethod: None,
        CallDoubleMethodV: None,
        CallDoubleMethodA: None,
        CallVoidMethod: None,
        CallVoidMethodV: None,
        CallVoidMethodA: None,
        CallNonvirtualObjectMethod: None,
        CallNonvirtualObjectMethodV: None,
        CallNonvirtualObjectMethodA: None,
        CallNonvirtualBooleanMethod: None,
        CallNonvirtualBooleanMethodV: None,
        CallNonvirtualBooleanMethodA: None,
        CallNonvirtualByteMethod: None,
        CallNonvirtualByteMethodV: None,
        CallNonvirtualByteMethodA: None,
        CallNonvirtualCharMethod: None,
        CallNonvirtualCharMethodV: None,
        CallNonvirtualCharMethodA: None,
        CallNonvirtualShortMethod: None,
        CallNonvirtualShortMethodV: None,
        CallNonvirtualShortMethodA: None,
        CallNonvirtualIntMethod: None,
        CallNonvirtualIntMethodV: None,
        CallNonvirtualIntMethodA: None,
        CallNonvirtualLongMethod: None,
        CallNonvirtualLongMethodV: None,
        CallNonvirtualLongMethodA: None,
        CallNonvirtualFloatMethod: None,
        CallNonvirtualFloatMethodV: None,
        CallNonvirtualFloatMethodA: None,
        CallNonvirtualDoubleMethod: None,
        CallNonvirtualDoubleMethodV: None,
        CallNonvirtualDoubleMethodA: None,
        CallNonvirtualVoidMethod: None,
        CallNonvirtualVoidMethodV: None,
        CallNonvirtualVoidMethodA: None,
        GetFieldID: Some(get_field_id),
        GetObjectField: Some(get_object_field),
        GetBooleanField: None,
        GetByteField: None,
        GetCharField: None,
        GetShortField: None,
        GetIntField: None,
        GetLongField: None,
        GetFloatField: None,
        GetDoubleField: None,
        SetObjectField: None,
        SetBooleanField: Some(set_boolean_field),
        SetByteField: None,
        SetCharField: None,
        SetShortField: None,
        SetIntField: Some(set_int_field),
        SetLongField: Some(set_long_field),
        SetFloatField: None,
        SetDoubleField: None,
        GetStaticMethodID: Some(get_static_method_id),
        CallStaticObjectMethod: None,
        CallStaticObjectMethodV: Some(unsafe { transmute::<_, unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut __va_list_tag) -> jobject>(call_static_object_method_v as *mut c_void) }),
        CallStaticObjectMethodA: None,
        CallStaticBooleanMethod: None,
        CallStaticBooleanMethodV: Some(unsafe { transmute::<_, unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut __va_list_tag) -> jboolean>(call_static_boolean_method_v as *mut c_void) }),
        CallStaticBooleanMethodA: None,
        CallStaticByteMethod: None,
        CallStaticByteMethodV: None,
        CallStaticByteMethodA: None,
        CallStaticCharMethod: None,
        CallStaticCharMethodV: None,
        CallStaticCharMethodA: None,
        CallStaticShortMethod: None,
        CallStaticShortMethodV: None,
        CallStaticShortMethodA: None,
        CallStaticIntMethod: None,
        CallStaticIntMethodV: None,
        CallStaticIntMethodA: None,
        CallStaticLongMethod: None,
        CallStaticLongMethodV: None,
        CallStaticLongMethodA: None,
        CallStaticFloatMethod: None,
        CallStaticFloatMethodV: None,
        CallStaticFloatMethodA: None,
        CallStaticDoubleMethod: None,
        CallStaticDoubleMethodV: None,
        CallStaticDoubleMethodA: None,
        CallStaticVoidMethod: None,
        CallStaticVoidMethodV: None,
        CallStaticVoidMethodA: None,
        GetStaticFieldID: Some(get_static_field_id),
        GetStaticObjectField: None,
        GetStaticBooleanField: None,
        GetStaticByteField: None,
        GetStaticCharField: None,
        GetStaticShortField: None,
        GetStaticIntField: None,
        GetStaticLongField: None,
        GetStaticFloatField: None,
        GetStaticDoubleField: None,
        SetStaticObjectField: Some(set_static_object_field),
        SetStaticBooleanField: None,
        SetStaticByteField: None,
        SetStaticCharField: None,
        SetStaticShortField: None,
        SetStaticIntField: None,
        SetStaticLongField: None,
        SetStaticFloatField: None,
        SetStaticDoubleField: None,
        NewString: Some(new_string),
        GetStringLength: Some(get_string_utflength),
        GetStringChars: None,
        ReleaseStringChars: Some(release_string_chars),
        NewStringUTF: Some(new_string_utf),
        GetStringUTFLength: Some(get_string_utflength),
        GetStringUTFChars: Some(get_string_utfchars),
        ReleaseStringUTFChars: Some(release_string_utfchars),
        GetArrayLength: Some(get_array_length),
        NewObjectArray: None,
        GetObjectArrayElement: None,
        SetObjectArrayElement: None,
        NewBooleanArray: None,
        NewByteArray: Some(new_byte_array),
        NewCharArray: None,
        NewShortArray: None,
        NewIntArray: None,
        NewLongArray: None,
        NewFloatArray: None,
        NewDoubleArray: None,
        GetBooleanArrayElements: None,
        GetByteArrayElements: None,
        GetCharArrayElements: None,
        GetShortArrayElements: None,
        GetIntArrayElements: None,
        GetLongArrayElements: None,
        GetFloatArrayElements: None,
        GetDoubleArrayElements: None,
        ReleaseBooleanArrayElements: None,
        ReleaseByteArrayElements: None,
        ReleaseCharArrayElements: None,
        ReleaseShortArrayElements: None,
        ReleaseIntArrayElements: None,
        ReleaseLongArrayElements: None,
        ReleaseFloatArrayElements: None,
        ReleaseDoubleArrayElements: None,
        GetBooleanArrayRegion: None,
        GetByteArrayRegion: Some(get_byte_array_region),
        GetCharArrayRegion: None,
        GetShortArrayRegion: None,
        GetIntArrayRegion: None,
        GetLongArrayRegion: None,
        GetFloatArrayRegion: None,
        GetDoubleArrayRegion: None,
        SetBooleanArrayRegion: None,
        SetByteArrayRegion: Some(set_byte_array_region),
        SetCharArrayRegion: None,
        SetShortArrayRegion: None,
        SetIntArrayRegion: None,
        SetLongArrayRegion: None,
        SetFloatArrayRegion: None,
        SetDoubleArrayRegion: None,
        RegisterNatives: Some(register_natives),
        UnregisterNatives: None,
        MonitorEnter: None,
        MonitorExit: None,
        GetJavaVM: Some(get_java_vm),
        GetStringRegion: Some(get_string_region),
        GetStringUTFRegion: Some(get_string_utfregion),
        GetPrimitiveArrayCritical: None,
        ReleasePrimitiveArrayCritical: None,
        GetStringCritical: None,
        ReleaseStringCritical: None,
        NewWeakGlobalRef: None,
        DeleteWeakGlobalRef: None,
        ExceptionCheck: Some(exception_check),
        NewDirectByteBuffer: None,
        GetDirectBufferAddress: None,
        GetDirectBufferCapacity: None,
        GetObjectRefType: None,
    }
}

#[no_mangle]
pub unsafe extern "C" fn call_object_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jobject {
    let method_id = (method_id as *mut MethodId).as_ref().unwrap();
    let classfile = method_id.class.classfile.clone();
    let method = &classfile.methods[method_id.method_i];
    if method.access_flags & ACC_STATIC > 0 {
        unimplemented!()
    }
    let state = get_state(env);
    let frame = get_frame(env);
    let exp_descriptor_str = method.descriptor_str(&classfile);
    let parsed = parse_method_descriptor(&method_id.class.loader, exp_descriptor_str.as_str()).unwrap();

    frame.push(JavaValue::Object(from_object(obj)));
    for type_ in &parsed.parameter_types {
        match type_ {
            ParsedType::ByteType => unimplemented!(),
            ParsedType::CharType => unimplemented!(),
            ParsedType::DoubleType => unimplemented!(),
            ParsedType::FloatType => unimplemented!(),
            ParsedType::IntType => unimplemented!(),
            ParsedType::LongType => unimplemented!(),
            ParsedType::Class(_) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            ParsedType::ShortType => unimplemented!(),
            ParsedType::BooleanType => unimplemented!(),
            ParsedType::ArrayReferenceType(_) => unimplemented!(),
            ParsedType::VoidType => unimplemented!(),
            ParsedType::TopType => unimplemented!(),
            ParsedType::NullType => unimplemented!(),
            ParsedType::Uninitialized(_) => unimplemented!(),
            ParsedType::UninitializedThis => unimplemented!(),
            ParsedType::UninitializedThisOrClass(_) => panic!(),
        }
    }
    //todo add params into operand stack;
    trace!("----NATIVE EXIT ----");
    invoke_virtual_method_i(state, frame.clone(), parsed, method_id.class.clone(), method_id.method_i, method);
    trace!("----NATIVE ENTER ----");
    let res = frame.pop().unwrap_object();
    to_object(res)
}

unsafe extern "C" fn exception_occured(_env: *mut JNIEnv) -> jthrowable {
    //exceptions don't happen yet todo
    std::ptr::null_mut()
}


unsafe extern "C" fn delete_local_ref(_env: *mut JNIEnv, _obj: jobject) {
    //todo no gc, just leak
}

unsafe extern "C" fn ensure_local_capacity(_env: *mut JNIEnv, _capacity: jint) -> jint {
    //we always have ram. todo
    0 as jint
}

unsafe extern "C" fn find_class(env: *mut JNIEnv, c_name: *const ::std::os::raw::c_char) -> jclass {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let state = get_state(env);
    let frame = get_frame(env);
    load_class_constant_by_name(state, &frame, name);
    let obj = frame.pop().unwrap_object();
    to_object(obj)
}

unsafe extern "C" fn new_global_ref(_env: *mut JNIEnv, lobj: jobject) -> jobject {
    let obj = from_object(lobj);
    match &obj {
        None => {}
        Some(o) => {
            Box::leak(Box::new(o.clone()));
        }
    }
    to_object(obj)
}

unsafe extern "C" fn get_static_method_id(
    _env: *mut JNIEnv,
    clazz: jclass,
    name: *const ::std::os::raw::c_char,
    sig: *const ::std::os::raw::c_char,
) -> jmethodID {
    let method_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let method_descriptor_str = CStr::from_ptr(sig).to_str().unwrap().to_string();
    let class_obj_o = from_object(clazz).unwrap();
    let class_obj = class_obj_o.unwrap_normal_object();
    //todo dup
    let runtime_class = class_obj.object_class_object_pointer.borrow().as_ref().unwrap().clone();
    let classfile = &runtime_class.classfile;
    let (method_i, method) = classfile.lookup_method(method_name, method_descriptor_str).unwrap();
    assert!(method.is_static());
    let res = Box::into_raw(Box::new(MethodId { class: runtime_class.clone(), method_i }));
    transmute(res)
}

unsafe extern "C" fn call_static_object_method_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: VaList) -> jobject {
    let frame = call_static_method_v(env, jmethod_id, &mut l);
    let res = frame.pop().unwrap_object();
    to_object(res)
}

unsafe fn call_static_method_v(env: *mut *const JNINativeInterface_, jmethod_id: jmethodID, l: &mut VaList) -> Rc<StackEntry> {
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame = get_frame(env);
    let classfile = &method_id.class.classfile;
    let method = &classfile.methods[method_id.method_i];
    let method_descriptor_str = method.descriptor_str(classfile);
    let _name = method.method_name(classfile);
    let parsed = parse_method_descriptor(&method_id.class.loader, method_descriptor_str.as_str()).unwrap();
//todo dup
    push_params_onto_frame(l, &frame, &parsed);
    trace!("----NATIVE EXIT ----");
    invoke_static_impl(state, frame.clone(), parsed, method_id.class.clone(), method_id.method_i, method);
    trace!("----NATIVE ENTER----");
    frame
}

unsafe fn push_params_onto_frame(l: &mut VaList, frame: &Rc<StackEntry>, parsed: &MethodDescriptor) {
    for type_ in &parsed.parameter_types {
        match type_ {
            ParsedType::ByteType => unimplemented!(),
            ParsedType::CharType => unimplemented!(),
            ParsedType::DoubleType => unimplemented!(),
            ParsedType::FloatType => unimplemented!(),
            ParsedType::IntType => unimplemented!(),
            ParsedType::LongType => unimplemented!(),
            ParsedType::Class(_) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            ParsedType::ShortType => unimplemented!(),
            ParsedType::BooleanType => unimplemented!(),
            ParsedType::ArrayReferenceType(a) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
                //todo dupe.
            },
            ParsedType::VoidType => unimplemented!(),
            ParsedType::TopType => unimplemented!(),
            ParsedType::NullType => unimplemented!(),
            ParsedType::Uninitialized(_) => unimplemented!(),
            ParsedType::UninitializedThis => unimplemented!(),
            ParsedType::UninitializedThisOrClass(_) => panic!()
        }
    }
}

unsafe extern "C" fn new_string(env: *mut JNIEnv, unicode: *const jchar, len: jsize) -> jstring {
    let mut str = String::with_capacity(len as usize);
    for i in 0..len {
        str.push(unicode.offset(i as isize).read() as u8 as char)
    }
    let res = new_string_with_string(env, str);
    assert_ne!(res, std::ptr::null_mut());
    res
}

pub struct FieldID {
    pub class: Arc<RuntimeClass>,
    pub field_i: usize,
}

unsafe extern "C" fn get_field_id(_env: *mut JNIEnv, clazz: jclass, c_name: *const ::std::os::raw::c_char, _sig: *const ::std::os::raw::c_char) -> jfieldID {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let runtime_class = runtime_class_from_object(clazz).unwrap();
    let fields = &runtime_class.classfile.fields;
    for field_i in 0..fields.len() {
        //todo check descriptor
        if fields[field_i].name(&runtime_class.classfile) == name {
            return Box::into_raw(Box::new(FieldID { class: runtime_class.clone(), field_i })) as jfieldID;
        }
    }
    panic!()
}


unsafe extern "C" fn get_string_utflength(_env: *mut JNIEnv, str: jstring) -> jsize {
    let str_obj = from_object(str).unwrap();
    //todo use length function.
    let str_fields = str_obj.unwrap_normal_object().fields.borrow();
    let char_object = str_fields.get("value").unwrap().unwrap_object().unwrap();
    let chars = char_object.unwrap_array();
    let borrowed_elems = chars.elems.borrow();
    borrowed_elems.len() as i32
}


pub unsafe extern "C" fn get_string_utfregion(_env: *mut JNIEnv, str: jstring, start: jsize, len: jsize, buf: *mut ::std::os::raw::c_char) {
    let str_obj = from_object(str).unwrap();
    let str_fields = str_obj.unwrap_normal_object().fields.borrow();
    let char_object = str_fields.get("value").unwrap().unwrap_object().unwrap();
    let chars = char_object.unwrap_array();
    let borrowed_elems = chars.elems.borrow();
    for i in start..(start + len) {
        let char_ = (&borrowed_elems[i as usize]).unwrap_char() as i8 as u8 as char;
        buf.offset(i as isize).write(char_ as i8);
    }
    buf.offset((start + len) as isize).write('\0' as i8);
}


pub unsafe fn runtime_class_from_object(cls: jclass) -> Option<Arc<RuntimeClass>> {
    let object_non_null = from_object(cls).unwrap().clone();
    let object_class = object_non_null.unwrap_normal_object().object_class_object_pointer.borrow();
    object_class.clone()
}


unsafe extern "C" fn get_superclass(env: *mut JNIEnv, sub: jclass) -> jclass {
    let super_name = match runtime_class_from_object(sub).unwrap().classfile.super_class_name() {
        None => { return to_object(None); }
        Some(n) => n,
    };
    let frame = get_frame(env);
    let state = get_state(env);
//    frame.print_stack_trace();
    let _inited_class = check_inited_class(state, &super_name, frame.clone().into(), frame.class_pointer.loader.clone());
    load_class_constant_by_name(state, &frame, super_name.get_referred_name());
    to_object(frame.pop().unwrap_object())
}


unsafe extern "C" fn is_assignable_from(_env: *mut JNIEnv, _sub: jclass, _sup: jclass) -> jboolean {
    //todo impl later
    true as jboolean
}

unsafe extern "C" fn get_static_field_id(env: *mut JNIEnv, clazz: jclass, name: *const ::std::os::raw::c_char, sig: *const ::std::os::raw::c_char) -> jfieldID {
//    get_frame(env).print_stack_trace();
    //todo should have its own impl
    get_field_id(env, clazz, name, sig)
}

unsafe extern "C" fn set_static_object_field(_env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jobject) {
//Box::into_raw(Box::new(FieldID { class: runtime_class.clone(), field_i })) as jfieldID;
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));//todo leak
    let value = from_object(value);
    let classfile = &field_id.class.classfile;
    let field_name = classfile.constant_pool[classfile.fields[field_id.field_i].name_index as usize].extract_string_from_utf8();
    let static_class = runtime_class_from_object(clazz).unwrap();
    static_class.static_vars.borrow_mut().insert(field_name, JavaValue::Object(value));
}


unsafe extern "C" fn new_byte_array(_env: *mut JNIEnv, len: jsize) -> jbyteArray {
    let mut the_vec = vec![];
    for _ in 0..len {
        the_vec.push(JavaValue::Byte(0))
    }
    to_object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(the_vec), elem_type: ParsedType::ByteType }))))
}

unsafe extern "C" fn get_string_region(_env: *mut JNIEnv, str: jstring, start: jsize, len: jsize, buf: *mut jchar) {
    let temp = from_object(str).unwrap().unwrap_normal_object().fields.borrow().get("value").unwrap().unwrap_object().unwrap();
    let char_array = &temp.unwrap_array().elems.borrow();
    let mut str_ = Vec::new();
    for char_ in char_array.iter() {
        str_.push(char_.unwrap_char())
    }
    for i in start..(start + len) {
        buf.offset(i as isize).write(str_[i as usize] as jchar);
    }
}

unsafe extern "C" fn call_static_boolean_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jboolean {
    call_static_method_v(env, method_id, &mut l);
    let res = get_frame(env).pop();
    res.unwrap_int() as jboolean
}


unsafe extern "C" fn get_array_length(_env: *mut JNIEnv, array: jarray) -> jsize {
    let non_null_array: &Object = &from_object(array).unwrap();
    let len = match non_null_array {
        Object::Array(a) => {
            a.elems.borrow().len()
        }
        Object::Object(_o) => {
            unimplemented!()
        }
    };
    len as jsize
}


unsafe extern "C" fn get_byte_array_region(_env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *mut jbyte) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().elems.borrow();
    let array = array_ref.deref();
    for i in start..(start + len) {
        buf.offset(i as isize).write(array[i as usize].unwrap_int() as jbyte)
    }
}


unsafe extern "C" fn get_object_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jobject {
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let nonnull = from_object(obj).unwrap();
    let field_borrow = nonnull.unwrap_normal_object().fields.borrow();
    let fields = field_borrow.deref();
    let classfile = &field_id.class.classfile;
    let field_name = classfile.fields[field_id.field_i].name(classfile);
    to_object(fields.get(&field_name).unwrap().unwrap_object())
}


unsafe extern "C" fn set_byte_array_region(_env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *const jbyte) {
    for i in start..(start + len) {
        from_object(array)
            .unwrap()
            .unwrap_array()
            .elems
            .borrow_mut()
            .insert(i as usize, JavaValue::Byte(buf.offset(i as isize).read() as i8));
    }
}


unsafe extern "C" fn new_object(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ...) -> jobject {
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame = get_frame(env);
    let classfile = &method_id.class.classfile;
    let method = &classfile.methods[method_id.method_i];
    let method_descriptor_str = method.descriptor_str(classfile);
    let _name = method.method_name(classfile);
    let parsed = parse_method_descriptor(&method_id.class.loader, method_descriptor_str.as_str()).unwrap();
    push_new_object(frame.clone(), &method_id.class);
    let obj = frame.pop();
    frame.push(obj.clone());
    for type_ in &parsed.parameter_types {
        match type_ {
            ParsedType::ByteType => unimplemented!(),
            ParsedType::CharType => unimplemented!(),
            ParsedType::DoubleType => unimplemented!(),
            ParsedType::FloatType => unimplemented!(),
            ParsedType::IntType => unimplemented!(),
            ParsedType::LongType => unimplemented!(),
            ParsedType::Class(_) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            ParsedType::ShortType => unimplemented!(),
            ParsedType::BooleanType => unimplemented!(),
            ParsedType::ArrayReferenceType(a) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
                //todo dupe.
            },
            ParsedType::VoidType => unimplemented!(),
            ParsedType::TopType => unimplemented!(),
            ParsedType::NullType => unimplemented!(),
            ParsedType::Uninitialized(_) => unimplemented!(),
            ParsedType::UninitializedThis => unimplemented!(),
            ParsedType::UninitializedThisOrClass(_) => panic!()
        }
    }
    invoke_special_impl(
        state,
        &frame,
        &parsed,
        method_id.method_i,
        method_id.class.clone(),
        &classfile.methods[method_id.method_i]
    );
    to_object(obj.unwrap_object())
}


unsafe extern "C" fn get_java_vm(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint{
    *vm = Box::into_raw(Box::new(Box::leak(Box::new(JNIInvokeInterface_ {
        reserved0: std::ptr::null_mut(),
        reserved1: std::ptr::null_mut(),
        reserved2: std::ptr::null_mut(),
        DestroyJavaVM: None,
        AttachCurrentThread: None,
        DetachCurrentThread: None,
        GetEnv: None,
        AttachCurrentThreadAsDaemon: None
    }))));
    0 as jint
}

unsafe extern "C" fn set_int_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jint){
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let classfile = &field_id.class.classfile;
    let name = classfile.fields[field_id.field_i as usize].name(classfile);
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow_mut().deref_mut().insert(name,JavaValue::Int(val));
}

unsafe extern "C" fn set_long_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jlong){
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let classfile = &field_id.class.classfile;
    let name = classfile.fields[field_id.field_i as usize].name(classfile);
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow_mut().deref_mut().insert(name,JavaValue::Long(val));
}


unsafe extern "C" fn set_boolean_field(env : * mut JNIEnv, obj : jobject, field_id_raw : jfieldID, val : jboolean ){
    let field_id:& FieldID  = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let classfile = &field_id.class.classfile;
    let name = classfile.fields[field_id.field_i as usize].name(classfile);
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow_mut().deref_mut().insert(name,JavaValue::Boolean(val != 0));
}