use crate::{JVMState, StackEntry, JVMTIState};
use crate::runtime_class::{prepare_class, RuntimeClass, RuntimeClassArray};
use crate::runtime_class::initialize_class;
use std::sync::Arc;
use rust_jvm_common::classnames::ClassName;
use classfile_view::loading::LoaderArc;
use crate::java_values::{JavaValue, default_value, Object};
use descriptor_parser::{parse_method_descriptor};
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use classfile_view::view::{HasAccessFlags, ClassView};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use std::ops::Deref;


//todo jni should really live in interpreter state

pub fn push_new_object(jvm: &JVMState, current_frame: &StackEntry, target_classfile: &Arc<RuntimeClass>, class_object_type: Option<Arc<RuntimeClass>>) {
    let loader_arc = &current_frame.class_pointer.loader(jvm).clone();
    let object_pointer = JavaValue::new_object(jvm, target_classfile.clone(), class_object_type);
    let new_obj = JavaValue::Object(object_pointer.clone());
    default_init_fields(jvm, loader_arc.clone(), object_pointer, target_classfile.view(), loader_arc.clone());
    current_frame.push(new_obj);
}

fn default_init_fields(jvm: &JVMState, loader_arc: LoaderArc, object_pointer: Option<Arc<Object>>, view: &ClassView, bl: LoaderArc) {
    if view.super_name().is_some() {
        let super_name = view.super_name();
        let loaded_super = loader_arc.load_class(loader_arc.clone(), &super_name.unwrap(), bl.clone(), jvm.get_live_object_pool_getter()).unwrap();
        default_init_fields(jvm, loader_arc.clone(), object_pointer.clone(), &loaded_super, bl);
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

pub fn run_constructor(state: &JVMState, frame: &StackEntry, target_classfile: Arc<RuntimeClass>, full_args: Vec<JavaValue>, descriptor: String) {
    let method_view = target_classfile.view().method_index().lookup(&"<init>".to_string(), &parse_method_descriptor(descriptor.as_str()).unwrap()).unwrap();
    let md = method_view.desc();
    let this_ptr = full_args[0].clone();
    let actual_args = &full_args[1..];
    frame.push(this_ptr);
    for arg in actual_args {
        frame.push(arg.clone());
    }
    //todo this should be invoke special
    invoke_virtual_method_i(state, md, target_classfile.clone(), method_view.method_i(), &method_view, false);
}




pub fn check_inited_class(
    jvm: &JVMState,
    ptype: &PTypeView,
    loader_arc: LoaderArc,
) -> Arc<RuntimeClass> {
    //todo racy/needs sychronization
    if !jvm.initialized_classes.read().unwrap().contains_key(&ptype) {
        //todo the below is jank
        match ptype.try_unwrap_ref_type() {
            None => {}
            Some(ref_) => {
                match ref_ {
                    ReferenceTypeView::Class(class_name) => {
                        if class_name == &ClassName::raw_byte() {
                            return check_inited_class(jvm, &PTypeView::ByteType, loader_arc);
                        }
                        if class_name == &ClassName::raw_char() {
                            return check_inited_class(jvm, &PTypeView::CharType, loader_arc);
                        }
                        if class_name == &ClassName::raw_double() {
                            return check_inited_class(jvm, &PTypeView::DoubleType, loader_arc);
                        }
                        if class_name == &ClassName::raw_float() {
                            return check_inited_class(jvm, &PTypeView::FloatType, loader_arc);
                        }
                        if class_name == &ClassName::raw_int() {
                            return check_inited_class(jvm, &PTypeView::IntType, loader_arc);
                        }
                        if class_name == &ClassName::raw_long() {
                            return check_inited_class(jvm, &PTypeView::LongType, loader_arc);
                        }
                        if class_name == &ClassName::raw_short() {
                            return check_inited_class(jvm, &PTypeView::ShortType, loader_arc);
                        }
                        if class_name == &ClassName::raw_boolean() {
                            return check_inited_class(jvm, &PTypeView::BooleanType, loader_arc);
                        }
                        if class_name == &ClassName::raw_void() {
                            return check_inited_class(jvm, &PTypeView::VoidType, loader_arc);
                        }
                    }
                    ReferenceTypeView::Array(_) => {}
                }
            }
        }
        &ptype.unwrap_type_to_name().map(|class_name|jvm.tracing.trace_class_loads(&class_name));
        let new_rclass = match &ptype{
            PTypeView::ByteType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Byte)//todo duplication with last line
            },
            PTypeView::CharType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Char)//todo duplication with last line
            },
            PTypeView::DoubleType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Double)
            },
            PTypeView::FloatType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Float)//todo duplication with last line
            },
            PTypeView::IntType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Int)//todo duplication with last line
            },
            PTypeView::LongType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Long)//todo duplication with last line
            },
            PTypeView::Ref(ref_) => match ref_{
                ReferenceTypeView::Class(class_name) => {
                    let new_rclass = check_inited_class_impl(jvm,class_name,loader_arc);
                    new_rclass
                },
                ReferenceTypeView::Array(arr) => {
                    let array_type_class = check_inited_class(jvm,arr.deref(),loader_arc);
                    let new_rclass = Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class: array_type_class }));
                    new_rclass
                },
            },
            PTypeView::ShortType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Short)//todo duplication with last line
            },
            PTypeView::BooleanType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Boolean)//todo duplication with last line
            },
            PTypeView::VoidType => {
                check_inited_class(jvm,&ptype.primitive_to_non_primitive_equiv().into(),loader_arc);
                Arc::new(RuntimeClass::Void)//todo duplication with last line
            },
            PTypeView::TopType | PTypeView::NullType | PTypeView::Uninitialized(_) | PTypeView::UninitializedThis |
            PTypeView::UninitializedThisOrClass(_) => panic!(),
        };
        jvm.initialized_classes.write().unwrap().insert(ptype.clone(),new_rclass);
        // jvm.jvmti_state.built_in_jdwp.class_prepare(jvm, ptype)//todo this should really happen in the function that actually does preparing
    } else {}
    //todo race?
    let res = jvm.initialized_classes.read().unwrap().get(ptype).unwrap().clone();
    res
}

fn check_inited_class_impl(
    jvm: &JVMState,
    class_name: &ClassName,
    loader_arc: LoaderArc,
) -> Arc<RuntimeClass>{
    let bl = jvm.bootstrap_loader.clone();
    let target_classfile = loader_arc.clone().load_class(loader_arc.clone(), &class_name, bl, jvm.get_live_object_pool_getter()).unwrap();
    let ptype = PTypeView::Ref(ReferenceTypeView::Class(class_name.clone()));
    let prepared = Arc::new(prepare_class(jvm, target_classfile.backing_class(), loader_arc.clone()));
    jvm.initialized_classes.write().unwrap().insert(ptype.clone(), prepared.clone());//must be before, otherwise infinite recurse
    let inited_target = initialize_class(prepared, jvm);
    jvm.initialized_classes.write().unwrap().insert(ptype.clone(), inited_target);
    match &jvm.jvmti_state{
        None => {},
        Some(jvmti) => {
            jvmti.built_in_jdwp.class_prepare(jvm,&class_name);
        },
    }
    let res = jvm.initialized_classes.read().unwrap().get(&ptype).unwrap().clone();
    res
}
