use std::ops::Deref;
use std::sync::Arc;

use classfile_view::loading::{ClassLoadingError, LoaderArc};
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::parse_method_descriptor;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::invoke::special::invoke_special_impl;
use crate::java::lang::class_not_found_exception;
use crate::java::lang::string::JString;
use crate::java_values::{default_value, JavaValue, Object};
use crate::runtime_class::{prepare_class, RuntimeClass, RuntimeClassArray};
use crate::runtime_class::initialize_class;

//todo jni should really live in interpreter state

pub fn push_new_object(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    target_classfile: &Arc<RuntimeClass>,
    class_object_type: Option<Arc<RuntimeClass>>,
) {
    let loader_arc = target_classfile.loader(jvm);//&int_state.current_frame().class_pointer().loader(jvm).clone();//todo fix loaders.
    let object_pointer = JavaValue::new_object(jvm, target_classfile.clone(), class_object_type);
    let new_obj = JavaValue::Object(object_pointer.clone());
    default_init_fields(jvm, int_state, loader_arc.clone(), object_pointer, target_classfile.view(), loader_arc.clone());
    int_state.current_frame_mut().push(new_obj);
}

fn default_init_fields(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    loader_arc: LoaderArc,
    object_pointer: Option<Arc<Object>>,
    view: &ClassView,
    bl: LoaderArc,
) {
    if view.super_name().is_some() {
        let super_name = view.super_name();
        let loaded_super = loader_arc.load_class(loader_arc.clone(), &super_name.unwrap(), bl.clone(), jvm.get_live_object_pool_getter()).unwrap();
        default_init_fields(jvm, int_state, loader_arc.clone(), object_pointer.clone(), &loaded_super, bl);
    }
    for field in view.fields() {
        if !field.is_static() {
            //todo should I look for constant val attributes?
            let _value_i = match field.constant_value_attribute() {
                None => {}
                Some(_i) => unimplemented!(),
            };
            let name = field.field_name();
            let type_ = field.field_type();
            let val = default_value(type_);
            {
                object_pointer.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert(name, val);
            }
        }
    }
}

pub fn run_constructor(
    state: &JVMState,
    int_state: &mut InterpreterStateGuard,
    target_classfile: Arc<RuntimeClass>,
    full_args: Vec<JavaValue>,
    descriptor: String,
) {
    let method_view = target_classfile.view().lookup_method(&"<init>".to_string(), &parse_method_descriptor(descriptor.as_str()).unwrap()).unwrap();
    let md = method_view.desc();
    let this_ptr = full_args[0].clone();
    let actual_args = &full_args[1..];
    int_state.push_current_operand_stack(this_ptr);
    for arg in actual_args {
        int_state.push_current_operand_stack(arg.clone());
    }
    // dbg!(int_state.current_frame().local_vars());
    // dbg!(int_state.current_frame().operand_stack());
    invoke_special_impl(state, int_state, &md, method_view.method_i(), target_classfile.clone(), &method_view);
}


pub fn check_inited_class(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    ptype: &PTypeView,
    loader_arc: LoaderArc,
) -> Result<Arc<RuntimeClass>, ClassLoadingError> {
    //todo racy/needs sychronization
    let before = int_state.int_state.as_ref().unwrap().call_stack.len();
    if !jvm.classes.initialized_classes.read().unwrap().contains_key(&ptype) && !jvm.classes.initializing_classes.read().unwrap().contains_key(&ptype) {
        //todo the below is jank
        match ptype.try_unwrap_ref_type() {
            None => {}
            Some(ref_) => {
                match ref_ {
                    ReferenceTypeView::Class(class_name) => {
                        if class_name == &ClassName::raw_byte() {
                            return check_inited_class(jvm, int_state, &PTypeView::ByteType, loader_arc);
                        }
                        if class_name == &ClassName::raw_char() {
                            return check_inited_class(jvm, int_state, &PTypeView::CharType, loader_arc);
                        }
                        if class_name == &ClassName::raw_double() {
                            return check_inited_class(jvm, int_state, &PTypeView::DoubleType, loader_arc);
                        }
                        if class_name == &ClassName::raw_float() {
                            return check_inited_class(jvm, int_state, &PTypeView::FloatType, loader_arc);
                        }
                        if class_name == &ClassName::raw_int() {
                            return check_inited_class(jvm, int_state, &PTypeView::IntType, loader_arc);
                        }
                        if class_name == &ClassName::raw_long() {
                            return check_inited_class(jvm, int_state, &PTypeView::LongType, loader_arc);
                        }
                        if class_name == &ClassName::raw_short() {
                            return check_inited_class(jvm, int_state, &PTypeView::ShortType, loader_arc);
                        }
                        if class_name == &ClassName::raw_boolean() {
                            return check_inited_class(jvm, int_state, &PTypeView::BooleanType, loader_arc);
                        }
                        if class_name == &ClassName::raw_void() {
                            return check_inited_class(jvm, int_state, &PTypeView::VoidType, loader_arc);
                        }
                    }
                    ReferenceTypeView::Array(_) => {}
                }
            }
        }
        if let Some(class_name) = ptype.unwrap_type_to_name() {
            jvm.tracing.trace_class_loads(&class_name)
        }
        let new_rclass = match &ptype {
            PTypeView::ByteType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Byte)//todo duplication with last line
            }
            PTypeView::CharType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Char)//todo duplication with last line
            }
            PTypeView::DoubleType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Double)
            }
            PTypeView::FloatType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Float)//todo duplication with last line
            }
            PTypeView::IntType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Int)//todo duplication with last line
            }
            PTypeView::LongType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Long)//todo duplication with last line
            }
            PTypeView::Ref(ref_) => match ref_ {
                ReferenceTypeView::Class(class_name) => {
                    check_inited_class_impl(jvm, int_state, class_name, loader_arc)?
                }
                ReferenceTypeView::Array(arr) => {
                    let array_type_class = check_inited_class(jvm, int_state, arr.deref(), loader_arc)?;
                    Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class: array_type_class }))
                }
            },
            PTypeView::ShortType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Short)//todo duplication with last line
            }
            PTypeView::BooleanType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Boolean)//todo duplication with last line
            }
            PTypeView::VoidType => {
                check_inited_class(jvm, int_state, &ptype.primitive_to_non_primitive_equiv().into(), loader_arc)?;
                Arc::new(RuntimeClass::Void)//todo duplication with last line
            }
            PTypeView::TopType | PTypeView::NullType | PTypeView::Uninitialized(_) | PTypeView::UninitializedThis |
            PTypeView::UninitializedThisOrClass(_) => panic!(),
        };
        jvm.classes.initialized_classes.write().unwrap().insert(ptype.clone(), new_rclass);
        // jvm.jvmti_state.built_in_jdwp.class_prepare(jvm, ptype)//todo this should really happen in the function that actually does preparing
    } else {}
    //todo race?
    let res = match jvm.classes.initialized_classes.read().unwrap().get(ptype) {
        None => {
            jvm.classes.initializing_classes.read().unwrap().get(ptype).unwrap().clone()
        }
        Some(class) => class.clone(),
    };
    let after = int_state.int_state.as_ref().unwrap().call_stack.len();
    assert_eq!(after, before);
    Ok(res)
}

fn check_inited_class_impl(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    class_name: &ClassName,
    loader_arc: LoaderArc,
) -> Result<Arc<RuntimeClass>, ClassLoadingError> {
    let bl = jvm.bootstrap_loader.clone();
    let target_classfile = loader_arc.clone().load_class(loader_arc.clone(), &class_name, bl, jvm.get_live_object_pool_getter()).map_err(|err| {
        int_state.print_stack_trace();
        if class_name == &ClassName::new("sun/dc/DuctusRenderingEngine") {
            let jclass_name = JString::from_rust(jvm, int_state, class_name.get_referred_name().to_string());
            let class_not_found_exception = class_not_found_exception::ClassNotFoundException::new(jvm, int_state, jclass_name);
            int_state.set_throw(class_not_found_exception.object().into());
            err
        } else {
            dbg!(class_name);
            int_state.print_stack_trace();
            panic!()
        }
    })?;
    let ptype = PTypeView::Ref(ReferenceTypeView::Class(class_name.clone()));
    let prepared = Arc::new(prepare_class(jvm, target_classfile.backing_class(), loader_arc.clone()));
    jvm.classes.prepared_classes.write().unwrap().insert(ptype.clone(), prepared.clone());
    jvm.classes.initializing_classes.write().unwrap().insert(ptype.clone(), prepared.clone());
    match prepared.view().super_name() {
        None => {}
        Some(super_name) => { check_inited_class(jvm, int_state, &super_name.into(), loader_arc)?; }
    };
    match &jvm.jvmti_state {
        None => {}
        Some(jvmti) => {
            // if class_name == &ClassName::Str("java/lang/Thread".to_string()) {
            //     println!("here")
            // }
            jvmti.built_in_jdwp.class_prepare(jvm, &class_name, int_state);
        }
    }

    let inited_target = initialize_class(prepared, jvm, int_state);
    if inited_target.is_none() {
        return Err(ClassLoadingError::ClassNotFoundException);
    }
    jvm.classes.initialized_classes.write().unwrap().insert(ptype.clone(), inited_target.unwrap());
    let res = jvm.classes.initialized_classes.read().unwrap().get(&ptype).unwrap().clone();
    Ok(res)
}
