use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::loading::LoaderName;
use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};

use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, NormalObject, Object};
use crate::jvm_state::{ClassStatus, JVMState};
use crate::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassArray, RuntimeClassClass};

//todo only use where spec says
pub fn check_initing_or_inited_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    let class = check_loaded_class(jvm, int_state, ptype.clone());
    match class.status() {
        ClassStatus::UNPREPARED => {
            prepare_class(jvm, class.view().backing_class(), &mut *class.static_vars());
            class.set_status(ClassStatus::PREPARED);
            check_initing_or_inited_class(jvm, int_state, ptype)
        }
        ClassStatus::PREPARED => {
            if let Some(super_name) = class.view().super_name() {
                check_initing_or_inited_class(jvm, int_state, super_name.into());
            }
            for interface in class.view().interfaces() {
                check_initing_or_inited_class(jvm, int_state, interface.interface_name().into());
            }
            class.set_status(ClassStatus::INITIALIZING);
            let res = initialize_class(class, jvm, int_state).unwrap();
            res.set_status(ClassStatus::INITIALIZED);
            res
        },
        ClassStatus::INITIALIZING => class,
        ClassStatus::INITIALIZED => class,
    }
}

pub fn assert_loaded_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    match jvm.classes.read().unwrap().initiating_loaders.get(&ptype) {
        None => panic!(),
        Some((_, res)) => res.clone()
    }
}

pub fn check_loaded_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    let guard = jvm.classes.read().unwrap();
    match guard.initiating_loaders.get(&ptype) {
        None => {
            let loader = int_state.current_loader();
            let res = match loader {
                LoaderName::UserDefinedLoader(loader_idx) => {
                    let loader_obj = jvm.class_loaders.write().unwrap().get_by_left(&loader_idx).unwrap().clone().0;
                    let class_loader: ClassLoader = JavaValue::Object(loader_obj.into()).cast_class_loader();
                    match ptype.clone() {
                        PTypeView::ByteType => todo!(),
                        PTypeView::CharType => todo!(),
                        PTypeView::DoubleType => todo!(),
                        PTypeView::FloatType => todo!(),
                        PTypeView::IntType => todo!(),
                        PTypeView::LongType => todo!(),
                        PTypeView::Ref(ref_) => {
                            match ref_ {
                                ReferenceTypeView::Class(class_name) => {
                                    let java_string = JString::from_rust(jvm, int_state, class_name.get_referred_name().clone());
                                    class_loader.load_class(jvm, int_state, java_string).as_runtime_class(jvm)
                                }
                                ReferenceTypeView::Array(sub_type) => {
                                    let sub_class = check_loaded_class(jvm, int_state, ptype);
                                    Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class }))
                                }
                            }
                        }
                        PTypeView::ShortType => todo!(),
                        PTypeView::BooleanType => todo!(),
                        PTypeView::VoidType => todo!(),
                        _ => todo!(),
                    }
                }
                LoaderName::BootstrapLoader => {
                    drop(guard);
                    bootstrap_load(jvm, int_state, ptype)
                }
            };
            let mut guard = jvm.classes.write().unwrap();
            guard.initiating_loaders.entry(res.ptypeview()).or_insert((loader, res.clone()));
            guard.loaded_classes_by_type.entry(loader).or_insert(HashMap::new()).insert(res.ptypeview(), res.clone());
            res
        }
        Some((_, res)) => res.clone()
    }
}

pub fn bootstrap_load(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    match ptype {
        PTypeView::ByteType => todo!(),
        PTypeView::CharType => todo!(),
        PTypeView::DoubleType => todo!(),
        PTypeView::FloatType => todo!(),
        PTypeView::IntType => todo!(),
        PTypeView::LongType => todo!(),
        PTypeView::Ref(ref_) => match ref_ {
            ReferenceTypeView::Class(class_name) => {
                let classfile = jvm.classpath.lookup(&class_name).unwrap();
                let class_view = Arc::new(ClassView::from(classfile.clone()));
                let res = Arc::new(RuntimeClass::Object(RuntimeClassClass {
                    class_view: class_view.clone(),
                    static_vars: Default::default(),
                    status: ClassStatus::UNPREPARED.into(),
                }));
                let class_object = Arc::new(Object::Object(NormalObject {
                    monitor: jvm.thread_state.new_monitor("class object monitor".to_string()),
                    fields: UnsafeCell::new(Default::default()),
                    class_pointer: jvm.classes.read().unwrap().class_class.clone(),
                }));
                jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(class_object), ByAddress(res.clone()));
                if let Some(super_name) = class_view.super_name() {
                    check_loaded_class(jvm, int_state, super_name.into());
                }
                res
            }
            ReferenceTypeView::Array(sub_type) => {
                let sub_class = check_initing_or_inited_class(jvm, int_state, sub_type.deref().clone());
                Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class }))
            }
        },
        PTypeView::ShortType => todo!(),
        PTypeView::BooleanType => todo!(),
        PTypeView::VoidType => todo!(),
        _ => todo!()
    }
}


pub fn check_resolved_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    check_loaded_class(jvm, int_state, ptype)
}

pub fn assert_inited_or_initing_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    let class: Arc<RuntimeClass> = assert_loaded_class(jvm, int_state, ptype.clone());
    match class.status() {
        ClassStatus::UNPREPARED => panic!(),
        ClassStatus::PREPARED => panic!(),
        ClassStatus::INITIALIZING => class,
        ClassStatus::INITIALIZED => class,
    }
}