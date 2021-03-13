use std::sync::Arc;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::jint;
use rust_jvm_common::classnames::ClassName;

use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::string::JString;
use crate::java_values::{ArrayObject, JavaValue, Object};
use crate::jvm_state::JVMState;

/*
// unofficial modifier flags, used by HotSpot:
    static final int BRIDGE    = 0x00000040;
    static final int VARARGS   = 0x00000080;
    static final int SYNTHETIC = 0x00001000;
    static final int ANNOTATION= 0x00002000;
    static final int ENUM      = 0x00004000;

    static final int
                MN_IS_METHOD           = 0x00010000, // method (not constructor)
                MN_IS_CONSTRUCTOR      = 0x00020000, // constructor
                MN_IS_FIELD            = 0x00040000, // field
                MN_IS_TYPE             = 0x00080000, // nested type
                MN_CALLER_SENSITIVE    = 0x00100000, // @CallerSensitive annotation detected
                MN_REFERENCE_KIND_SHIFT = 24, // refKind
                MN_REFERENCE_KIND_MASK = 0x0F000000 >> MN_REFERENCE_KIND_SHIFT,
                // The SEARCH_* bits are not for MN.flags but for the matchFlags argument of MHN.getMembers:
                MN_SEARCH_SUPERCLASSES = 0x00100000,
                MN_SEARCH_INTERFACES   = 0x00200000;

         /**
         * Access modifier flags.
         */
        static final char
            ACC_PUBLIC                 = 0x0001,
            ACC_PRIVATE                = 0x0002,
            ACC_PROTECTED              = 0x0004,
            ACC_STATIC                 = 0x0008,
            ACC_FINAL                  = 0x0010,
            ACC_SYNCHRONIZED           = 0x0020,
            ACC_VOLATILE               = 0x0040,
            ACC_TRANSIENT              = 0x0080,
            ACC_NATIVE                 = 0x0100,
            ACC_INTERFACE              = 0x0200,
            ACC_ABSTRACT               = 0x0400,
            ACC_STRICT                 = 0x0800,
            ACC_SYNTHETIC              = 0x1000,
            ACC_ANNOTATION             = 0x2000,
            ACC_ENUM                   = 0x4000,
            // aliases:
            ACC_SUPER                  = ACC_SYNCHRONIZED,
            ACC_BRIDGE                 = ACC_VOLATILE,
            ACC_VARARGS                = ACC_TRANSIENT;
            
            todo do these need to be added on top of access flags?
*/


fn get_modifiers(method_view: &MethodView) -> jint {
    method_view.access_flags() as i32
}


fn get_signature(state: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView) -> Result<JString, WasException> {
    Ok(JString::from_rust(state, int_state, method_view.desc_str())?.intern(state, int_state)?)
}

fn exception_types_table(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView) -> JavaValue {
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const
    let exception_table: Vec<JavaValue> = method_view.code_attribute()
        .map(|x| &x.exception_table)
        .unwrap_or(&vec![])
        .iter()
        .map(|x| x.catch_type)
        .map(|x| if x == 0 {
            ReferenceTypeView::Class(ClassName::throwable())
        } else {
            method_view.classview().constant_pool_view(x as usize).unwrap_class().class_ref_type()
        })
        .map(|x| {
            PTypeView::Ref(x)
        })
        .map(|x| {
            JClass::from_type(jvm, int_state, x).java_value()
        })
        .collect();
    JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
        jvm,
        int_state,
        exception_table,
        class_type,
        jvm.thread_state.new_monitor("".to_string()),
    )))))
}

fn parameters_type_objects(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView) -> JavaValue {
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const
    let mut res = vec![];
    let parsed = method_view.desc();
    for param_type in parsed.parameter_types {
        res.push(JClass::from_type(jvm, int_state, PTypeView::from_ptype(&param_type)).java_value());
    }

    JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
        jvm,
        int_state,
        res,
        class_type,
        jvm.thread_state.new_monitor("".to_string()),
    )))))
}


pub mod method {
    use std::sync::Arc;

    use classfile_view::view::ClassView;
    use classfile_view::view::method_view::MethodView;
    use classfile_view::view::ptype_view::PTypeView;
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;
    use type_safe_proc_macro_utils::getter_gen;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::instructions::ldc::load_class_constant_by_type;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::reflect::{exception_types_table, get_modifiers, get_signature, parameters_type_objects};
    use crate::java::lang::reflect::constructor::Constructor;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    const METHOD_SIGNATURE: &str = "(Ljava/lang/Class;Ljava/lang/String;[Ljava/lang/Class;Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B[B)V";

    pub struct Method {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method(&self) -> Method {
            Method { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Method {
        pub fn method_object_from_method_view(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView) -> Result<Method, WasException> {
            let clazz = {
                let field_class_type = method_view.classview().type_();
                //todo so if we are calling this on int.class that is caught by the unimplemented above.
                load_class_constant_by_type(jvm, int_state, field_class_type);
                int_state.pop_current_operand_stack().cast_class()
            };
            let name = {
                let name = method_view.name();
                if name == "<init>" {
                    return Ok(Constructor::constructor_object_from_method_view(jvm, int_state, method_view)?.java_value().cast_method());
                }
                JString::from_rust(jvm, int_state, name)?.intern(jvm, int_state)?
            };
            let parameter_types = parameters_type_objects(jvm, int_state, &method_view);
            let return_type = {
                let rtype = method_view.desc().return_type;
                JClass::from_type(jvm, int_state, PTypeView::from_ptype(&rtype))
            };
            let exception_types = exception_types_table(jvm, int_state, &method_view);
            let modifiers = get_modifiers(&method_view);
            //todo what does slot do?
            let slot = -1;
            let signature = get_signature(jvm, int_state, &method_view)?;
            let annotations = JavaValue::empty_byte_array(jvm, int_state);
            let parameter_annotations = JavaValue::empty_byte_array(jvm, int_state);
            let annotation_default = JavaValue::empty_byte_array(jvm, int_state);
            Ok(Method::new_method(jvm, int_state, clazz, name, parameter_types, return_type, exception_types, modifiers, slot, signature, annotations, parameter_annotations, annotation_default)?)
        }

        pub fn new_method(jvm: &JVMState,
                          int_state: &mut InterpreterStateGuard,
                          clazz: JClass,
                          name: JString,
                          parameter_types: JavaValue,
                          return_type: JClass,
                          exception_types: JavaValue,
                          modifiers: jint,
                          slot: jint,
                          signature: JString,
                          annotations: JavaValue,
                          parameter_annotations: JavaValue,
                          annotation_default: JavaValue,
        ) -> Result<Method, WasException> {
            let method_class = check_initing_or_inited_class(jvm, int_state, ClassName::method().into()).unwrap();
            push_new_object(jvm, int_state, &method_class);
            let method_object = int_state.pop_current_operand_stack();
            let full_args = vec![method_object.clone(),
                                 clazz.java_value(),
                                 name.java_value(),
                                 parameter_types,
                                 return_type.java_value(),
                                 exception_types,
                                 JavaValue::Int(modifiers),
                                 JavaValue::Int(slot),
                                 signature.java_value(),
                                 annotations,
                                 parameter_annotations,
                                 annotation_default];
            //todo replace with wrapper object
            run_constructor(jvm, int_state, method_class, full_args, METHOD_SIGNATURE.to_string())?;
            Ok(method_object.cast_method())
        }


        pub fn init() -> Self {
            unimplemented!()
        }

        pub fn get_clazz(&self) -> JClass {
            self.normal_object.lookup_field("clazz").cast_class()
        }

        pub fn get_modifiers(&self) -> jint {
            self.normal_object.lookup_field("modifiers").unwrap_int()
        }

        pub fn get_name(&self) -> JString {
            self.normal_object.lookup_field("name").cast_string()
        }

        pub fn parameter_types(&self) -> Vec<JClass> {
            self.normal_object.lookup_field("parameterTypes").unwrap_array().mut_array().iter().map(|value| value.cast_class()).collect()
        }

        getter_gen!(slot,jint,unwrap_int);

        as_object_or_java_value!();
    }
}

pub mod constructor {
    use std::sync::Arc;

    use classfile_view::view::ClassView;
    use classfile_view::view::method_view::MethodView;
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::instructions::ldc::load_class_constant_by_type;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::reflect::{exception_types_table, get_modifiers, get_signature, parameters_type_objects};
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    const CONSTRUCTOR_SIGNATURE: &str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

    pub struct Constructor {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_constructor(&self) -> Constructor {
            Constructor { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Constructor {
        pub fn constructor_object_from_method_view(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView) -> Result<Constructor, WasException> {
            let clazz = {
                let field_class_type = method_view.classview().type_();
                //todo this doesn't cover the full generality of this, b/c we could be calling on int.class or array classes
                load_class_constant_by_type(jvm, int_state, field_class_type);
                int_state.pop_current_operand_stack().cast_class()
            };

            let parameter_types = parameters_type_objects(jvm, int_state, &method_view);
            let exception_types = exception_types_table(jvm, int_state, &method_view);
            let modifiers = get_modifiers(&method_view);
            //todo what does slot do?
            let slot = -1;
            let signature = get_signature(jvm, int_state, &method_view)?;
            Constructor::new_constructor(jvm, int_state, clazz, parameter_types, exception_types, modifiers, slot, signature)
        }


        pub fn new_constructor(
            jvm: &JVMState,
            int_state: &mut InterpreterStateGuard,
            clazz: JClass,
            parameter_types: JavaValue,
            exception_types: JavaValue,
            modifiers: jint,
            slot: jint,
            signature: JString,
        ) -> Result<Constructor, WasException> {
            let constructor_class = check_initing_or_inited_class(jvm, int_state, ClassName::constructor().into()).unwrap();//todo pass the error up
            //todo impl these
            push_new_object(jvm, int_state, &constructor_class);
            let constructor_object = int_state.pop_current_operand_stack();

            let empty_byte_array = JavaValue::empty_byte_array(jvm, int_state);
            let full_args = vec![constructor_object.clone(), clazz.java_value(), parameter_types, exception_types, JavaValue::Int(modifiers), JavaValue::Int(slot), signature.java_value(), empty_byte_array.clone(), empty_byte_array];
            run_constructor(jvm, int_state, constructor_class, full_args, CONSTRUCTOR_SIGNATURE.to_string())?;
            Ok(constructor_object.cast_constructor())
        }

        as_object_or_java_value!();
    }
}

pub mod field {
    use std::sync::Arc;

    use classfile_view::view::ptype_view::PTypeView;
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::string::JString;
    use crate::java_values::{ArrayObject, JavaValue, Object};

    pub struct Field {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_field(&self) -> Field {
            Field { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Field {
        pub fn init(
            jvm: &JVMState,
            int_state: &mut InterpreterStateGuard,
            clazz: JClass,
            name: JString,
            type_: JClass,
            modifiers: jint,
            slot: jint,
            signature: JString,
            annotations: Vec<JavaValue>,
        ) -> Result<Self, WasException> {
            let field_classfile = check_initing_or_inited_class(jvm, int_state, ClassName::field().into()).unwrap();//todo pass the error up
            push_new_object(jvm, int_state, &field_classfile);
            let field_object = int_state.pop_current_operand_stack();


            let modifiers = JavaValue::Int(modifiers);
            let slot = JavaValue::Int(slot);

            //todo impl annotations.
            let annotations = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
                jvm,
                int_state,
                annotations,
                PTypeView::ByteType,
                jvm.thread_state.new_monitor("monitor for annotations array".to_string()),
            )))));

            run_constructor(
                jvm,
                int_state,
                field_classfile,
                vec![field_object.clone(), clazz.java_value(), name.java_value(), type_.java_value(), modifiers, slot, signature.java_value(), annotations],
                "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;IILjava/lang/String;[B)V".to_string(),
            )?;
            Ok(field_object.cast_field())
        }

        pub fn name(&self) -> JString {
            self.normal_object.lookup_field("name").cast_string()
        }

        pub fn clazz(&self) -> JClass {
            self.normal_object.lookup_field("clazz").cast_class()
        }

        as_object_or_java_value!();
    }
}

pub mod constant_pool {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::push_new_object;
    use crate::java::lang::class::JClass;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct ConstantPool {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_constant_pool(&self) -> ConstantPool {
            ConstantPool { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl ConstantPool {
        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: JClass) -> ConstantPool {
            let constant_pool_classfile = check_initing_or_inited_class(jvm, int_state, ClassName::new("java/lang/reflect/ConstantPool").into()).unwrap();//todo pass the error up
            push_new_object(jvm, int_state, &constant_pool_classfile);
            let constant_pool_object = int_state.pop_current_operand_stack();
            let res = constant_pool_object.cast_constant_pool();
            res.set_constant_pool_oop(class);
            res
        }

        pub fn get_constant_pool_oop(&self) -> JClass {
            self.normal_object.lookup_field("constantPoolOop").cast_class()
        }


        pub fn set_constant_pool_oop(&self, jclass: JClass) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("constantPoolOop".to_string(), jclass.java_value());
        }

        as_object_or_java_value!();
    }
}