use std::cell::UnsafeCell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use by_address::ByAddress;

use classfile_parser::parse_validation::AttributeEnclosingType::Class;
use classfile_view::loading::LoaderName;
use classfile_view::loading::LoaderName::BootstrapLoader;
use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classnames::ClassName;

use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, NormalObject, Object};
use crate::jvm_state::{ClassStatus, JVMState};
use crate::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassArray, RuntimeClassClass};

//todo only use where spec says
pub fn check_initing_or_inited_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    let class = check_loaded_class(jvm, int_state, ptype.clone());
    match class.deref() {
        RuntimeClass::Byte => {
            check_initing_or_inited_class(jvm, int_state, ClassName::byte().into());
            return class;
        }
        RuntimeClass::Boolean => {
            check_initing_or_inited_class(jvm, int_state, ClassName::boolean().into());
            return class;
        }
        RuntimeClass::Short => {
            check_initing_or_inited_class(jvm, int_state, ClassName::short().into());
            return class;
        }
        RuntimeClass::Char => {
            check_initing_or_inited_class(jvm, int_state, ClassName::character().into());
            return class;
        }
        RuntimeClass::Int => {
            check_initing_or_inited_class(jvm, int_state, ClassName::int().into());
            return class;
        }
        RuntimeClass::Long => {
            check_initing_or_inited_class(jvm, int_state, ClassName::long().into());
            return class;
        }
        RuntimeClass::Float => {
            check_initing_or_inited_class(jvm, int_state, ClassName::float().into());
            return class;
        }
        RuntimeClass::Double => {
            check_initing_or_inited_class(jvm, int_state, ClassName::double().into());
            return class;
        }
        RuntimeClass::Void => {
            check_initing_or_inited_class(jvm, int_state, ClassName::void().into());
            return class;
        }
        RuntimeClass::Array(a) => {
            check_initing_or_inited_class(jvm, int_state, a.sub_class.ptypeview());
        }
        _ => {}
    }
    match class.status() {
        ClassStatus::UNPREPARED => {
            prepare_class(jvm, class.view().backing_class(), &mut *class.static_vars());
            class.set_status(ClassStatus::PREPARED);
            check_initing_or_inited_class(jvm, int_state, ptype)
        }
        ClassStatus::PREPARED => {
            class.set_status(ClassStatus::INITIALIZING);
            if let Some(super_name) = class.view().super_name() {
                check_initing_or_inited_class(jvm, int_state, super_name.into());
            }
            for interface in class.view().interfaces() {
                check_initing_or_inited_class(jvm, int_state, interface.interface_name().into());
            }
            assert!(int_state.throw().is_none());
            let res = initialize_class(class, jvm, int_state).unwrap();
            res.set_status(ClassStatus::INITIALIZED);
            res
        }
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
    // todo cleanup how these guards work
    let guard = jvm.classes.write().unwrap();
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
                                    let sub_class = check_loaded_class(jvm, int_state, sub_type.deref().clone());
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
    let (class_object, runtime_class) = match ptype.clone() {
        PTypeView::ByteType => (create_class_object(jvm, int_state, ClassName::new("byte").into(), BootstrapLoader), Arc::new(RuntimeClass::Byte)),
        PTypeView::CharType => (create_class_object(jvm, int_state, ClassName::new("char").into(), BootstrapLoader), Arc::new(RuntimeClass::Char)),
        PTypeView::DoubleType => (create_class_object(jvm, int_state, ClassName::new("double").into(), BootstrapLoader), Arc::new(RuntimeClass::Double)),
        PTypeView::FloatType => (create_class_object(jvm, int_state, ClassName::new("float").into(), BootstrapLoader), Arc::new(RuntimeClass::Float)),
        PTypeView::IntType => (create_class_object(jvm, int_state, ClassName::new("int").into(), BootstrapLoader), Arc::new(RuntimeClass::Int)),
        PTypeView::LongType => (create_class_object(jvm, int_state, ClassName::new("long").into(), BootstrapLoader), Arc::new(RuntimeClass::Long)),
        PTypeView::Ref(ref_) => match ref_ {
            ReferenceTypeView::Class(class_name) => {
                let classfile = jvm.classpath.lookup(&class_name).unwrap();
                let class_view = Arc::new(ClassView::from(classfile.clone()));
                let res = Arc::new(RuntimeClass::Object(RuntimeClassClass {
                    class_view: class_view.clone(),
                    static_vars: Default::default(),
                    status: ClassStatus::UNPREPARED.into(),
                }));
                jvm.classes.write().unwrap().initiating_loaders.entry(ptype.clone()).or_insert((BootstrapLoader, res.clone()));
                let class_object = create_class_object(jvm, int_state, class_name.into(), BootstrapLoader);
                jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(class_object.clone()), ByAddress(res.clone()));
                if let Some(super_name) = class_view.super_name() {
                    check_loaded_class(jvm, int_state, super_name.into());
                }
                for interface in class_view.interfaces() {
                    check_loaded_class(jvm, int_state, interface.interface_name().into());
                }
                (class_object, res)
            }
            ReferenceTypeView::Array(sub_type) => {
                let sub_class = check_resolved_class(jvm, int_state, sub_type.deref().clone());
                //todo handle class objects for arraus
                (create_class_object(jvm, int_state, None, BootstrapLoader), Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class })))
            }
        },
        PTypeView::ShortType => (create_class_object(jvm, int_state, ClassName::new("short").into(), BootstrapLoader), Arc::new(RuntimeClass::Short)),
        PTypeView::BooleanType => (create_class_object(jvm, int_state, ClassName::new("boolean").into(), BootstrapLoader), Arc::new(RuntimeClass::Boolean)),
        PTypeView::VoidType => (create_class_object(jvm, int_state, ClassName::new("void").into(), BootstrapLoader), Arc::new(RuntimeClass::Void)),
        _ => todo!()
    };
    jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(class_object), ByAddress(runtime_class.clone()));
    runtime_class
}

pub fn create_class_object(jvm: &JVMState, int_state: &mut InterpreterStateGuard, name: Option<ClassName>, loader: LoaderName) -> Arc<Object> {
    let loader_object = match loader {
        LoaderName::UserDefinedLoader(idx) => {
            JavaValue::Object(jvm.class_loaders.read().unwrap().get_by_left(&idx).unwrap().clone().0.into())
        }
        LoaderName::BootstrapLoader => {
            JavaValue::Object(None)
        }
    };
    if name == ClassName::new("java/lang/Object").into() {
        let mut fields: HashMap<String, JavaValue, RandomState> = Default::default();
        fields.insert("name".to_string(), JavaValue::Object(None));
        fields.insert("classLoader".to_string(), JavaValue::Object(None));
        return Arc::new(Object::Object(NormalObject {
            monitor: jvm.thread_state.new_monitor("object class object monitor".to_string()),
            fields: UnsafeCell::new(fields),
            class_pointer: jvm.classes.read().unwrap().class_class.clone(),
        }));
    }
    let class_object = match loader {
        LoaderName::UserDefinedLoader(idx) => {
            JClass::new(jvm, int_state, loader_object.cast_class_loader())
        }
        BootstrapLoader => {
            JClass::new_bootstrap_loader(jvm, int_state)
        }
    };
    // match name {
    //     None => {}
    //     Some(name) => {
    //         if ((|| { Some(jvm.classes.read().unwrap().loaded_classes_by_type.get(&BootstrapLoader)?.get(&ClassName::string().into())?.status()) })()) == ClassStatus::INITIALIZED.into() {
    //             class_object.set_name_(JString::from_rust(jvm, int_state, name.get_referred_name().to_string()))
    //         }
    //     }
    // }
    class_object.object()
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