use wtf8::Wtf8Buf;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;

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

fn get_signature(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, method_view: &MethodView) -> Result<JString<'gc_life>, WasException> {
    Ok(JString::from_rust(jvm, int_state, Wtf8Buf::from_string(method_view.desc_str().to_str(&jvm.string_pool)))?.intern(jvm, int_state)?)
}

fn exception_types_table(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, method_view: &MethodView) -> Result<JavaValue<'gc_life>, WasException> {
    let class_type: CPDType = CClassName::class().into();
    let empty_vec = vec![];
    let types_iter = method_view
        .code_attribute()
        .map(|x| &x.exception_table)
        .unwrap_or(&empty_vec)
        .iter()
        .map(|x| x.catch_type)
        .map(|x| match x {
            None => CPRefType::Class(CClassName::throwable()),
            Some(x) => CPRefType::Class(x),
        })
        .map(|x| CPDType::Ref(x));

    let mut exception_table: Vec<JavaValue<'gc_life>> = vec![]; //types_iter
    for ptype in types_iter {
        exception_table.push(JClass::from_type(jvm, int_state, ptype)?.java_value())
    }
    Ok(JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(jvm, int_state, exception_table, class_type, jvm.thread_state.new_monitor("".to_string()))?)))))
}

fn parameters_type_objects(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, method_view: &MethodView) -> Result<JavaValue<'gc_life>, WasException> {
    let class_type: CPDType = CClassName::class().into();
    let mut res = vec![];
    let parsed = method_view.desc();
    for param_type in &parsed.arg_types {
        res.push(JClass::from_type(jvm, int_state, param_type.clone())?.java_value());
    }

    Ok(JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(jvm, int_state, res, class_type, jvm.thread_state.new_monitor("".to_string()))?)))))
}

pub mod method {
    use wtf8::Wtf8Buf;

    use classfile_view::view::ClassView;
    use classfile_view::view::method_view::MethodView;
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::instructions::ldc::load_class_constant_by_type;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::reflect::{exception_types_table, get_modifiers, get_signature, parameters_type_objects};
    use crate::java::lang::string::JString;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    const METHOD_SIGNATURE: &str = "(Ljava/lang/Class;Ljava/lang/String;[Ljava/lang/Class;Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B[B)V";

    pub struct Method<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_method(&self) -> Method<'gc_life> {
            Method { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Method<'gc_life> {
        pub fn method_object_from_method_view(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, method_view: &MethodView) -> Result<Method<'gc_life>, WasException> {
            let clazz = {
                let field_class_type = method_view.classview().type_();
                //todo so if we are calling this on int.class that is caught by the unimplemented above.
                load_class_constant_by_type(jvm, int_state, &field_class_type)?.cast_class().unwrap()
            };
            let name = {
                let name = method_view.name();
                if name == MethodName::constructor_init() {
                    todo!()
                    // return Ok(Constructor::constructor_object_from_method_view(jvm, int_state, method_view)?.java_value().cast_method());
                }
                JString::from_rust(jvm, int_state, Wtf8Buf::from_string(name.0.to_str(&jvm.string_pool)))?.intern(jvm, int_state)?
            };
            let parameter_types = parameters_type_objects(jvm, int_state, &method_view)?;
            let return_type = {
                let cpdtype = method_view.desc().return_type.clone(); //todo this is a spurious clone
                JClass::from_type(jvm, int_state, cpdtype)?
            };
            let exception_types = exception_types_table(jvm, int_state, &method_view)?;
            let modifiers = get_modifiers(&method_view);
            //todo what does slot do?
            let slot = -1;
            let signature = get_signature(jvm, int_state, &method_view)?;
            let annotations = JavaValue::empty_byte_array(jvm, int_state)?;
            let parameter_annotations = JavaValue::empty_byte_array(jvm, int_state)?;
            let annotation_default = JavaValue::empty_byte_array(jvm, int_state)?;
            Ok(Method::new_method(jvm, int_state, clazz, name, parameter_types, return_type, exception_types, modifiers, slot, signature, annotations, parameter_annotations, annotation_default)?)
        }

        pub fn new_method(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, clazz: JClass<'gc_life>, name: JString<'gc_life>, parameter_types: JavaValue<'gc_life>, return_type: JClass<'gc_life>, exception_types: JavaValue<'gc_life>, modifiers: jint, slot: jint, signature: JString<'gc_life>, annotations: JavaValue<'gc_life>, parameter_annotations: JavaValue<'gc_life>, annotation_default: JavaValue<'gc_life>) -> Result<Method<'gc_life>, WasException> {
            let method_class = check_initing_or_inited_class(jvm, int_state, CClassName::method().into()).unwrap();
            let method_object = new_object(jvm, int_state, &method_class);
            let full_args = vec![method_object.clone(), clazz.java_value(), name.java_value(), parameter_types, return_type.java_value(), exception_types, JavaValue::Int(modifiers), JavaValue::Int(slot), signature.java_value(), annotations, parameter_annotations, annotation_default];
            //todo replace with wrapper object
            let c_method_descriptor = CMethodDescriptor::void_return(vec![
                CClassName::class().into(),
                CClassName::string().into(),
                CPDType::array(CClassName::class().into()),
                CClassName::class().into(),
                CPDType::array(CClassName::class().into()),
                CPDType::IntType,
                CPDType::IntType,
                CClassName::string().into(),
                CPDType::array(CPDType::ByteType),
                CPDType::array(CPDType::ByteType),
                CPDType::array(CPDType::ByteType),
            ]);
            run_constructor(jvm, int_state, method_class, full_args, &c_method_descriptor)?;
            Ok(method_object.cast_method())
        }

        pub fn get_clazz(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_clazz()).cast_class().unwrap()
            //todo this unwrap
        }

        pub fn get_modifiers(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
            self.normal_object.lookup_field(jvm, FieldName::field_modifiers()).unwrap_int()
        }

        pub fn get_name(&self, jvm: &'gc_life JVMState<'gc_life>) -> JString<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_name()).cast_string().expect("methods must have names")
        }

        pub fn parameter_types(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<JClass<'gc_life>> {
            self.normal_object.lookup_field(jvm, FieldName::field_parameterTypes()).unwrap_array().array_iterator(jvm).map(|value| value.cast_class().unwrap()).collect()
            //todo unwrap
        }

        pub fn get_slot_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<jint> {
            let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_slot());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.unwrap_int().into()
                } else {
                    None
                }
            } else {
                maybe_null.unwrap_int().into()
            }
        }
        pub fn get_slot(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
            self.get_slot_or_null(jvm).unwrap()
        }
        pub fn get_return_type_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<JClass<'gc_life>> {
            let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_returnType());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.cast_class().into()
                } else {
                    None
                }
            } else {
                maybe_null.cast_class().into()
            }
        }
        pub fn get_return_type(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.get_return_type_or_null(jvm).unwrap()
        }

        as_object_or_java_value!();
    }
}

pub mod constructor {
    use classfile_view::view::ClassView;
    use classfile_view::view::method_view::MethodView;
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::instructions::ldc::load_class_constant_by_type;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::reflect::{exception_types_table, get_modifiers, get_signature, parameters_type_objects};
    use crate::java::lang::string::JString;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    const CONSTRUCTOR_SIGNATURE: &str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

    pub struct Constructor<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_constructor(&self) -> Constructor<'gc_life> {
            Constructor { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Constructor<'gc_life> {
        pub fn constructor_object_from_method_view(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, method_view: &MethodView) -> Result<Constructor<'gc_life>, WasException> {
            let clazz = {
                let field_class_type = method_view.classview().type_();
                //todo this doesn't cover the full generality of this, b/c we could be calling on int.class or array classes
                load_class_constant_by_type(jvm, int_state, &field_class_type)?.cast_class().unwrap()
            };

            let parameter_types = parameters_type_objects(jvm, int_state, &method_view)?;
            let exception_types = exception_types_table(jvm, int_state, &method_view)?;
            let modifiers = get_modifiers(&method_view);
            //todo what does slot do?
            let slot = -1;
            let signature = get_signature(jvm, int_state, &method_view)?;
            Constructor::new_constructor(jvm, int_state, clazz, parameter_types, exception_types, modifiers, slot, signature)
        }

        pub fn new_constructor(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, clazz: JClass<'gc_life>, parameter_types: JavaValue<'gc_life>, exception_types: JavaValue<'gc_life>, modifiers: jint, slot: jint, signature: JString<'gc_life>) -> Result<Constructor<'gc_life>, WasException> {
            let constructor_class = check_initing_or_inited_class(jvm, int_state, CClassName::constructor().into())?;
            let constructor_object = new_object(jvm, int_state, &constructor_class);

            //todo impl annotations
            let empty_byte_array = JavaValue::empty_byte_array(jvm, int_state)?;
            let full_args = vec![constructor_object.clone(), clazz.java_value(), parameter_types, exception_types, JavaValue::Int(modifiers), JavaValue::Int(slot), signature.java_value(), empty_byte_array.clone(), empty_byte_array];
            let c_method_descriptor = CMethodDescriptor::void_return(vec![CClassName::class().into(), CPDType::array(CClassName::class().into()), CPDType::array(CClassName::class().into()), CPDType::IntType, CPDType::IntType, CClassName::string().into(), CPDType::array(CPDType::ByteType), CPDType::array(CPDType::ByteType)]);
            run_constructor(jvm, int_state, constructor_class, full_args, &c_method_descriptor)?;
            Ok(constructor_object.cast_constructor())
        }

        pub fn get_clazz(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_clazz()).cast_class().unwrap()
            //todo this unwrap
        }

        pub fn get_modifiers(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
            self.normal_object.lookup_field(jvm, FieldName::field_modifiers()).unwrap_int()
        }

        pub fn get_name(&self, jvm: &'gc_life JVMState<'gc_life>) -> JString<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_name()).cast_string().expect("methods must have names")
        }

        pub fn parameter_types(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<JClass<'gc_life>> {
            self.normal_object.lookup_field(jvm, FieldName::field_parameterTypes()).unwrap_array().array_iterator(jvm).map(|value| value.cast_class().unwrap()).collect()
            //todo unwrap
        }

        pub fn get_slot_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<jint> {
            let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_slot());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.unwrap_int().into()
                } else {
                    None
                }
            } else {
                maybe_null.unwrap_int().into()
            }
        }
        pub fn get_slot(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
            self.get_slot_or_null(jvm).unwrap()
        }
        pub fn get_return_type_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<JClass<'gc_life>> {
            let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_returnType());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.cast_class().into()
                } else {
                    None
                }
            } else {
                maybe_null.cast_class().into()
            }
        }
        pub fn get_return_type(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.get_return_type_or_null(jvm).unwrap()
        }

        as_object_or_java_value!();
    }
}

pub mod field {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::string::JString;
    use crate::java_values::{ArrayObject, GcManagedObject, JavaValue, Object};

    pub struct Field<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_field(&self) -> Field<'gc_life> {
            Field { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Field<'gc_life> {
        pub fn init(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, clazz: JClass<'gc_life>, name: JString<'gc_life>, type_: JClass<'gc_life>, modifiers: jint, slot: jint, signature: JString<'gc_life>, annotations: Vec<JavaValue<'gc_life>>) -> Result<Self, WasException> {
            let field_classfile = check_initing_or_inited_class(jvm, int_state, CClassName::field().into())?;
            let field_object = new_object(jvm, int_state, &field_classfile);

            let modifiers = JavaValue::Int(modifiers);
            let slot = JavaValue::Int(slot);

            //todo impl annotations.
            let annotations = JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(jvm, int_state, annotations, CPDType::ByteType, jvm.thread_state.new_monitor("monitor for annotations array".to_string()))?))));

            run_constructor(
                jvm,
                int_state,
                field_classfile,
                vec![field_object.clone(), clazz.java_value(), name.java_value(), type_.java_value(), modifiers, slot, signature.java_value(), annotations],
                &CMethodDescriptor::void_return(vec![CClassName::class().into(), CClassName::string().into(), CClassName::class().into(), CPDType::IntType, CPDType::IntType, CClassName::string().into(), CPDType::array(CPDType::ByteType)]),
            )?;
            Ok(field_object.cast_field())
        }

        pub fn name(&self, jvm: &'gc_life JVMState<'gc_life>) -> JString<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_name()).cast_string().expect("fields must have names")
        }

        pub fn clazz(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_clazz()).cast_class().expect("todo")
        }

        as_object_or_java_value!();
    }
}

pub mod constant_pool {
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::new_object;
    use crate::java::lang::class::JClass;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct ConstantPool<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_constant_pool(&self) -> ConstantPool<'gc_life> {
            ConstantPool { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> ConstantPool<'gc_life> {
        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>, class: JClass<'gc_life>) -> Result<ConstantPool<'gc_life>, WasException> {
            let constant_pool_classfile = check_initing_or_inited_class(jvm, int_state, CClassName::constant_pool().into())?;
            let constant_pool_object = new_object(jvm, int_state, &constant_pool_classfile);
            let res = constant_pool_object.cast_constant_pool();
            res.set_constant_pool_oop(class);
            Ok(res)
        }

        pub fn get_constant_pool_oop(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_constantPoolOop()).cast_class().unwrap()
        }

        pub fn set_constant_pool_oop(&self, jclass: JClass<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level(FieldName::field_constantPoolOop(), jclass.java_value());
        }

        as_object_or_java_value!();
    }
}