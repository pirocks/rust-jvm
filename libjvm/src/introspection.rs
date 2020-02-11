use jni_bindings::{jobjectArray, jclass, JNIEnv, jobject, jint, jstring, jbyteArray, jboolean, JVM_ExceptionTableEntryType};
use slow_interpreter::rust_jni::native_util::{to_object, get_state, get_frame};
use std::sync::Arc;
use runtime_common::java_values::{Object, ArrayObject, JavaValue};
use std::cell::RefCell;
use rust_jvm_common::unified_types::{ParsedType, ClassWithLoader};
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::interpreter_util::{run_constructor, push_new_object, check_inited_class};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_name};
use classfile_parser::types::{parse_method_descriptor, parse_field_descriptor};
use runtime_common::{StackEntry, InterpreterState};
use std::rc::Rc;
use slow_interpreter::{array_of_type_class, get_or_create_class_object};
use rust_jvm_common::classfile::ACC_PUBLIC;
use std::ops::Deref;
use std::ffi::CStr;
use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
use slow_interpreter::rust_jni::interface::string::new_string_with_string;

pub mod constant_pool {
    use jni_bindings::{JNIEnv, jclass, jobject, jint, jobjectArray, jfloat, jlong, jdouble, jstring};

    #[no_mangle]
    unsafe extern "system" fn JVM_GetClassConstantPool(env: *mut JNIEnv, cls: jclass) -> jobject {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetSize(env: *mut JNIEnv, unused: jobject, jcpool: jobject) -> jint {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetClassAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jclass {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetClassAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jclass {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetMethodAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetMethodAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetFieldAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetFieldAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetMemberRefInfoAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobjectArray {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetIntAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jint {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetLongAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jlong {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetFloatAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jfloat {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetDoubleAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jdouble {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetStringAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jstring {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_ConstantPoolGetUTF8At(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jstring {
        unimplemented!()
    }

}


pub mod is_x{
    use jni_bindings::{jdouble, jboolean, JNIEnv, jclass};
    use rust_jvm_common::classfile::ACC_INTERFACE;
    use rust_jvm_common::classnames::class_name;
    use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;

    #[no_mangle]
    unsafe extern "system" fn JVM_IsNaN(d: jdouble) -> jboolean {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_IsInterface(env: *mut JNIEnv, cls: jclass) -> jboolean {
//    get_frame(env).print_stack_trace();
        (runtime_class_from_object(cls).unwrap().classfile.access_flags & ACC_INTERFACE > 0) as jboolean
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_IsArrayClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
        unimplemented!()
    }

    #[no_mangle]
    /**
        * Determines if the specified {@code Class} object represents a
        * primitive type.
        *
        * <p> There are nine predefined {@code Class} objects to represent
        * the eight primitive types and void.  These are created by the Java
        * Virtual Machine, and have the same names as the primitive types that
        * they represent, namely {@code boolean}, {@code byte},
        * {@code char}, {@code short}, {@code int},
        * {@code long}, {@code float}, and {@code double}.
        *
        * <p> These objects may only be accessed via the following public static
        * final variables, and are the only {@code Class} objects for which
        * this method returns {@code true}.
        *
        * @return true if and only if this class represents a primitive type
        *
        * @see     java.lang.Boolean#TYPE
        * @see     java.lang.Character#TYPE
        * @see     java.lang.Byte#TYPE
        * @see     java.lang.Short#TYPE
        * @see     java.lang.Integer#TYPE
        * @see     java.lang.Long#TYPE
        * @see     java.lang.Float#TYPE
        * @see     java.lang.Double#TYPE
        * @see     java.lang.Void#TYPE
        * @since JDK1.1
        */
    unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
//    get_frame(env).print_stack_trace();
        let class_object = runtime_class_from_object(cls);
        if class_object.is_none() {
            return false as jboolean;
        }
        let name = class_name(&class_object.unwrap().classfile).get_referred_name();
        dbg!(&name);
        let is_primitive = name == "java/lang/Boolean".to_string() ||
            name == "java/lang/Character".to_string() ||
            name == "java/lang/Byte".to_string() ||
            name == "java/lang/Short".to_string() ||
            name == "java/lang/Integer".to_string() ||
            name == "java/lang/Long".to_string() ||
            name == "java/lang/Float".to_string() ||
            name == "java/lang/Double".to_string() ||
            name == "java/lang/Void".to_string();

        is_primitive as jboolean
    }



    #[no_mangle]
    unsafe extern "system" fn JVM_IsConstructorIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "system" fn JVM_IsVMGeneratedMethodIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
        unimplemented!()
    }

}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassInterfaces(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassSigners(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetProtectionDomain(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetComponentType(env: *mut JNIEnv, cls: jclass) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    runtime_class_from_object(cls).unwrap().classfile.access_flags as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaredClasses(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaringClass(env: *mut JNIEnv, ofClass: jclass) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassSignature(env: *mut JNIEnv, cls: jclass) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAnnotations(env: *mut JNIEnv, cls: jclass) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassTypeAnnotations(env: *mut JNIEnv, cls: jclass) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetFieldTypeAnnotations(env: *mut JNIEnv, field: jobject) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodTypeAnnotations(env: *mut JNIEnv, method: jobject) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    unimplemented!()
}

fn field_type_to_class(state: &mut InterpreterState, frame: &Rc<StackEntry>, type_: &ParsedType) -> JavaValue {
    match type_ {
        ParsedType::IntType => {
            load_class_constant_by_name(state, frame, "java/lang/Integer".to_string());
        }
        ParsedType::Class(cl) => {
            load_class_constant_by_name(state, frame, cl.class_name.get_referred_name());
        }
        ParsedType::BooleanType => {
            //todo dup.
            load_class_constant_by_name(state, frame, "java/lang/Boolean".to_string());
        }
        ParsedType::LongType => {
            //todo dup.
            load_class_constant_by_name(state, frame, "java/lang/Long".to_string());
        }
        ParsedType::ArrayReferenceType(sub) => {
            frame.push(JavaValue::Object(array_of_type_class(state, frame.clone(), sub.sub_type.deref()).into()));
        }
        ParsedType::CharType => {
            load_class_constant_by_name(state, frame, "java/lang/Character".to_string());
        }
        _ => {
            dbg!(type_);
            frame.print_stack_trace();
            unimplemented!()
        }
    }
    frame.pop()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredFields(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let frame = get_frame(env);
    let state = get_state(env);
//    frame.print_stack_trace();
    let class_obj = runtime_class_from_object(ofClass);
//    dbg!(&class_obj.clone().unwrap_normal_object().class_pointer);
//    let runtime_object = state.class_object_pool.borrow().get(&class_obj).unwrap();
    let field_classfile = check_inited_class(state, &ClassName::Str("java/lang/reflect/Field".to_string()), frame.clone().into(), frame.class_pointer.loader.clone());
    let mut object_array = vec![];
    &class_obj.clone().unwrap().classfile.fields.iter().enumerate().for_each(|(i, f)| {
        push_new_object(frame.clone(), &field_classfile);
        let field_object = frame.pop();

        object_array.push(field_object.clone());
        let field_class_name = class_name(&class_obj.clone().as_ref().unwrap().classfile).get_referred_name();
        load_class_constant_by_name(state, &frame, field_class_name);
        let parent_runtime_class = frame.pop();
        let field_name = class_obj.clone().unwrap().classfile.constant_pool[f.name_index as usize].extract_string_from_utf8();
        create_string_on_stack(state, &frame, field_name);
        let field_name_string = frame.pop();

        let field_desc_str = class_obj.clone().unwrap().classfile.constant_pool[f.descriptor_index as usize].extract_string_from_utf8();
        let field_type = parse_field_descriptor(&frame.class_pointer.loader, field_desc_str.as_str()).unwrap().field_type;
        let field_type_class = field_type_to_class(state, &frame, &field_type);

        let modifiers = JavaValue::Int(f.access_flags as i32);
        let slot = JavaValue::Int(i as i32);

        create_string_on_stack(state, &frame, field_desc_str);
        let signature_string = frame.pop();

        //todo impl annotations.
        let annotations = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: ParsedType::ByteType }))));

        run_constructor(
            state,
            frame.clone(),
            field_classfile.clone(),
            vec![field_object, parent_runtime_class, field_name_string, field_type_class, modifiers, slot, signature_string, annotations],
            "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;IILjava/lang/String;[B)V".to_string(),
        )
    });

    //first arg: runtime_class
    //second arg: name
    //third arg: type class pointer
    //fourth arg: access_flags
    //fifth: put index here
    //descriptor
    //just put empty byte array??
//    Field(Class<?> var1, String var2, Class<?> var3, int var4, int var5, String var6, byte[] var7) {
//        this.clazz = var1;
//        this.name = var2;
//        this.type = var3;
//        this.modifiers = var4;
//        this.slot = var5;
//        this.signature = var6;
//        this.annotations = var7;
//    }
//    class_obj.unwrap()

    let res = Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(object_array), elem_type: ParsedType::Class(ClassWithLoader { class_name: class_name(&field_classfile.classfile), loader: field_classfile.loader.clone() }) })));
    to_object(res)
}


const CONSTRUCTOR_SIGNATURE: &'static str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let temp = runtime_class_from_object(ofClass).unwrap();
    let target_classfile = &temp.classfile;
    let constructors = target_classfile.lookup_method_name(&"<init>".to_string());
    let state = get_state(env);
    let frame = get_frame(env);
    let class_obj = runtime_class_from_object(ofClass);
    let loader = frame.class_pointer.loader.clone();
    let constructor_class = check_inited_class(state, &ClassName::new("java/lang/reflect/Constructor"), frame.clone().into(), loader.clone());
    let mut object_array = vec![];

    constructors.clone().iter().filter(|(i, m)| {
        if publicOnly > 0 {
            m.access_flags & ACC_PUBLIC > 0
        } else {
            true
        }
    }).for_each(|(i, m)| {
        let class_type = ParsedType::Class(ClassWithLoader { class_name: ClassName::class(), loader: loader.clone() });//todo this should be a global const

        push_new_object(frame.clone(), &constructor_class);
        let constructor_object = frame.pop();

        object_array.push(constructor_object.clone());

        let clazz = {
            let field_class_name = class_name(&class_obj.clone().as_ref().unwrap().classfile).get_referred_name();
            load_class_constant_by_name(state, &frame, field_class_name);
            frame.pop()
        };

        let parameter_types = {
            let mut res = vec![];
            let desc_str = m.descriptor_str(&target_classfile);
            let parsed = parse_method_descriptor(&loader, desc_str.as_str()).unwrap();
            for param_type in parsed.parameter_types {
                res.push(match param_type {
                    ParsedType::Class(c) => {
                        load_class_constant_by_name(state, &frame, c.class_name.get_referred_name());
                        frame.pop()
                    }
                    _ => unimplemented!()
                });
            }

            JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(res), elem_type: class_type.clone() }))))
        };


        let exceptionTypes = {
            //todo not currently supported
            assert!(m.code_attribute().unwrap().exception_table.is_empty());
            JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: class_type.clone() }))))
        };

        let modifiers = JavaValue::Int(constructor_class.classfile.access_flags as i32);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);

        let signature = {
            create_string_on_stack(state, &frame, m.descriptor_str(&target_classfile));
            frame.pop()
        };

        //todo impl these
        let empty_byte_array = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: ParsedType::ByteType }))));

        let full_args = vec![constructor_object, clazz, parameter_types, exceptionTypes, modifiers, slot, signature, empty_byte_array.clone(), empty_byte_array];
        run_constructor(state, frame.clone(), constructor_class.clone(), full_args, CONSTRUCTOR_SIGNATURE.to_string())
    });
    let res = Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(object_array), elem_type: ParsedType::Class(ClassWithLoader { class_name: class_name(&constructor_class.classfile), loader: constructor_class.loader.clone() }) })));
    to_object(res)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAccessFlags(env: *mut JNIEnv, cls: jclass) -> jint {
    runtime_class_from_object(cls).unwrap().classfile.access_flags as i32
}


#[no_mangle]
unsafe extern "system" fn JVM_ClassDepth(env: *mut JNIEnv, name: jstring) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassContext(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassNameUTF(env: *mut JNIEnv, cb: jclass) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPTypes(env: *mut JNIEnv, cb: jclass, types: *mut ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPEntriesCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassFieldsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassMethodsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionsCount(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCode(env: *mut JNIEnv, cb: jclass, method_index: jint, code: *mut ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCodeLength(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableLength(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetFieldIxModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxLocalsCount(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxArgsSize(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxMaxStack(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetCallerClass(env: *mut JNIEnv, depth: ::std::os::raw::c_int) -> jclass {
    /*todo, so this is needed for booting but it is what could best be described as an advanced feature.
    Therefore it only sorta works*/
    let frame = get_frame(env);
    let state = get_state(env);

    load_class_constant_by_name(state, &frame, class_name(&frame.last_call_stack.as_ref().unwrap().class_pointer.classfile).get_referred_name());
    let jclass = frame.pop().unwrap_object();
    to_object(jclass)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int, calledClass: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int, calledClass: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSameClassPackage(env: *mut JNIEnv, class1: jclass, class2: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetEnclosingMethodInfo(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableEntry(
    env: *mut JNIEnv,
    cb: jclass,
    method_index: jint,
    entry_index: jint,
    entry: *mut JVM_ExceptionTableEntryType,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionIndexes(
    env: *mut JNIEnv,
    cb: jclass,
    method_index: jint,
    exceptions: *mut ::std::os::raw::c_ushort,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromCaller(
    env: *mut JNIEnv,
    c_name: *const ::std::os::raw::c_char,
    init: jboolean,
    loader: jobject,
    caller: jclass,
) -> jclass {
    let state = get_state(env);
    let frame = get_frame(env);

    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    to_object(Some(get_or_create_class_object(state, &ClassName::Str(name), frame.clone(), frame.class_pointer.loader.clone())))
}



#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let obj = runtime_class_from_object(cls).unwrap();
    let full_name = class_name(&obj.classfile).get_referred_name().replace("/", ".");
//    use regex::Regex;
//    let rg = Regex::new("/[A-Za-z_][A-Za-z_0-9]*");//todo use a correct regex
//    let class_name = rg.unwrap().captures(full_name.as_str()).unwrap().iter().last().unwrap().unwrap().as_str();
    new_string_with_string(env, full_name)
}

