use std::cell::UnsafeCell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use by_address::ByAddress;

use classfile_view::loading::{LivePoolGetter, LoaderName};
use classfile_view::loading::LoaderName::BootstrapLoader;
use classfile_view::view::{ClassBackedView, ClassView};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;
use verification::{ClassFileGetter, VerifierContext, verify};

use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::class_not_found_exception::ClassNotFoundException;
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, NormalObject, Object};
use crate::jvm_state::{ClassStatus, JVMState};
use crate::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassArray, RuntimeClassClass};

//todo only use where spec says
pub fn check_initing_or_inited_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Result<Arc<RuntimeClass>, WasException> {
    let class = check_loaded_class(jvm, int_state, ptype.clone())?;
    match class.deref() {
        RuntimeClass::Byte => {
            check_initing_or_inited_class(jvm, int_state, ClassName::byte().into())?;
            return Ok(class);
        }
        RuntimeClass::Boolean => {
            check_initing_or_inited_class(jvm, int_state, ClassName::boolean().into())?;
            return Ok(class);
        }
        RuntimeClass::Short => {
            check_initing_or_inited_class(jvm, int_state, ClassName::short().into())?;
            return Ok(class);
        }
        RuntimeClass::Char => {
            check_initing_or_inited_class(jvm, int_state, ClassName::character().into())?;
            return Ok(class);
        }
        RuntimeClass::Int => {
            check_initing_or_inited_class(jvm, int_state, ClassName::int().into())?;
            return Ok(class);
        }
        RuntimeClass::Long => {
            check_initing_or_inited_class(jvm, int_state, ClassName::long().into())?;
            return Ok(class);
        }
        RuntimeClass::Float => {
            check_initing_or_inited_class(jvm, int_state, ClassName::float().into())?;
            return Ok(class);
        }
        RuntimeClass::Double => {
            check_initing_or_inited_class(jvm, int_state, ClassName::double().into())?;
            return Ok(class);
        }
        RuntimeClass::Void => {
            check_initing_or_inited_class(jvm, int_state, ClassName::void().into())?;
            return Ok(class);
        }
        RuntimeClass::Array(a) => {
            check_initing_or_inited_class(jvm, int_state, a.sub_class.ptypeview())?;
        }
        _ => {}
    }
    match class.status() {
        ClassStatus::UNPREPARED => {
            prepare_class(jvm, int_state, class.view(), &mut *class.static_vars());
            class.set_status(ClassStatus::PREPARED);
            check_initing_or_inited_class(jvm, int_state, ptype)
        }
        ClassStatus::PREPARED => {
            class.set_status(ClassStatus::INITIALIZING);
            if let Some(super_name) = class.view().super_name() {
                check_initing_or_inited_class(jvm, int_state, super_name.into())?;
            }
            for interface in class.view().interfaces() {
                check_initing_or_inited_class(jvm, int_state, interface.interface_name().into())?;
            }
            assert!(int_state.throw().is_none());
            let res = initialize_class(class, jvm, int_state)?;
            res.set_status(ClassStatus::INITIALIZED);
            Ok(res)
        }
        ClassStatus::INITIALIZING => Ok(class),
        ClassStatus::INITIALIZED => Ok(class),
    }
}

pub fn assert_loaded_class(jvm: &JVMState, _int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    match jvm.classes.read().unwrap().initiating_loaders.get(&ptype) {
        None => panic!(),
        Some((_, res)) => res.clone()
    }
}

pub fn check_loaded_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Result<Arc<RuntimeClass>, WasException> {
    let loader = int_state.current_loader();
    check_loaded_class_force_loader(jvm, int_state, &ptype, loader)
}

pub(crate) fn check_loaded_class_force_loader(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: &PTypeView, loader: LoaderName) -> Result<Arc<RuntimeClass>, WasException> {
// todo cleanup how these guards work
    let guard = jvm.classes.write().unwrap();
    match guard.initiating_loaders.get(&ptype) {
        None => {
            let res = match loader {
                LoaderName::UserDefinedLoader(loader_idx) => {
                    let loader_obj = jvm.class_loaders.write().unwrap().get_by_left(&loader_idx).unwrap().clone().0;
                    let class_loader: ClassLoader = JavaValue::Object(loader_obj.into()).cast_class_loader();
                    match ptype.clone() {
                        PTypeView::ByteType => Arc::new(RuntimeClass::Byte),
                        PTypeView::CharType => Arc::new(RuntimeClass::Char),
                        PTypeView::DoubleType => Arc::new(RuntimeClass::Double),
                        PTypeView::FloatType => Arc::new(RuntimeClass::Float),
                        PTypeView::IntType => Arc::new(RuntimeClass::Int),
                        PTypeView::LongType => Arc::new(RuntimeClass::Long),
                        PTypeView::Ref(ref_) => {
                            match ref_ {
                                ReferenceTypeView::Class(class_name) => {
                                    drop(guard);
                                    let java_string = JString::from_rust(jvm, int_state, class_name.get_referred_name().replace("/", ".").clone())?;
                                    class_loader.load_class(jvm, int_state, java_string)?.as_runtime_class(jvm)
                                }
                                ReferenceTypeView::Array(sub_type) => {
                                    drop(guard);
                                    let sub_class = check_loaded_class(jvm, int_state, sub_type.deref().clone())?;
                                    let res = Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class }));
                                    let obj = create_class_object(jvm, int_state, None, loader)?;
                                    jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(obj), ByAddress(res.clone()));
                                    res
                                }
                            }
                        }
                        PTypeView::ShortType => Arc::new(RuntimeClass::Short),
                        PTypeView::BooleanType => Arc::new(RuntimeClass::Boolean),
                        PTypeView::VoidType => Arc::new(RuntimeClass::Void),
                        _ => panic!(),
                    }
                }
                LoaderName::BootstrapLoader => {
                    drop(guard);
                    bootstrap_load(jvm, int_state, ptype.clone())?
                }
            };
            let mut guard = jvm.classes.write().unwrap();
            guard.initiating_loaders.entry(res.ptypeview()).insert((loader, res.clone()));
            guard.loaded_classes_by_type.entry(loader).or_insert(HashMap::new()).insert(res.ptypeview(), res.clone());
            Ok(res)
        }
        Some((_, res)) => Ok(res.clone())
    }
}

pub struct DefaultClassfileGetter<'l> {
    jvm: &'l JVMState,
}

impl ClassFileGetter for DefaultClassfileGetter<'_> {
    fn get_classfile(&self, _loader: LoaderName, class: ClassName) -> Arc<Classfile> {
        //todo verification needs to be better hooked in
        self.jvm.classpath.lookup(&class).unwrap()
    }
}

pub struct DefaultLivePoolGetter {}

impl LivePoolGetter for DefaultLivePoolGetter {
    fn elem_type(&self, _idx: usize) -> ReferenceTypeView {
        todo!()
    }
}

pub fn bootstrap_load(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Result<Arc<RuntimeClass>, WasException> {
    let (class_object, runtime_class) = match ptype.clone() {
        PTypeView::ByteType => (create_class_object(jvm, int_state, ClassName::new("byte").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Byte)),
        PTypeView::CharType => (create_class_object(jvm, int_state, ClassName::new("char").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Char)),
        PTypeView::DoubleType => (create_class_object(jvm, int_state, ClassName::new("double").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Double)),
        PTypeView::FloatType => (create_class_object(jvm, int_state, ClassName::new("float").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Float)),
        PTypeView::IntType => (create_class_object(jvm, int_state, ClassName::new("int").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Int)),
        PTypeView::LongType => (create_class_object(jvm, int_state, ClassName::new("long").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Long)),
        PTypeView::Ref(ref_) => match ref_ {
            ReferenceTypeView::Class(class_name) => {
                let classfile = match jvm.classpath.lookup(&class_name) {
                    Ok(x) => x,
                    Err(_) => {
                        let class_name_string = JString::from_rust(jvm, int_state, class_name.get_referred_name().clone())?;
                        let exception = ClassNotFoundException::new(jvm, int_state, class_name_string)?.object();
                        int_state.set_throw(exception.into());
                        return Err(WasException);
                    }
                };
                let class_view = Arc::new(ClassBackedView::from(classfile.clone()));
                let mut verifier_context = VerifierContext {
                    live_pool_getter: Arc::new(DefaultLivePoolGetter {}) as Arc<dyn LivePoolGetter>,
                    classfile_getter: Arc::new(DefaultClassfileGetter {
                        jvm
                    }) as Arc<dyn ClassFileGetter>,
                    current_loader: LoaderName::BootstrapLoader,
                    verification_types: Default::default(),
                    debug: class_name == ClassName::string()
                };
                verify(&mut verifier_context, class_view.deref(), LoaderName::BootstrapLoader).unwrap();
                let res = Arc::new(RuntimeClass::Object(RuntimeClassClass {
                    class_view: class_view.clone(),
                    static_vars: Default::default(),
                    status: ClassStatus::UNPREPARED.into(),
                }));
                let verification_types = verifier_context.verification_types;
                let mut method_table = jvm.method_table.write().unwrap();
                for (method_i, verification_types) in verification_types {
                    let method_id = method_table.get_method_id(res.clone(), method_i);
                    jvm.function_frame_type_data.write().unwrap().insert(method_id, verification_types);
                }
                drop(method_table);
                jvm.classes.write().unwrap().initiating_loaders.entry(ptype.clone()).or_insert((BootstrapLoader, res.clone()));
                let class_object = create_class_object(jvm, int_state, class_name.into(), BootstrapLoader)?;
                jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(class_object.clone()), ByAddress(res.clone()));
                if let Some(super_name) = class_view.super_name() {
                    check_loaded_class(jvm, int_state, super_name.into())?;
                }
                for interface in class_view.interfaces() {
                    check_loaded_class(jvm, int_state, interface.interface_name().into())?;
                }
                (class_object, res)
            }
            ReferenceTypeView::Array(sub_type) => {
                let sub_class = check_resolved_class(jvm, int_state, sub_type.deref().clone())?;
                //todo handle class objects for arraus
                (create_class_object(jvm, int_state, None, BootstrapLoader)?, Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class })))
            }
        },
        PTypeView::ShortType => (create_class_object(jvm, int_state, ClassName::new("short").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Short)),
        PTypeView::BooleanType => (create_class_object(jvm, int_state, ClassName::new("boolean").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Boolean)),
        PTypeView::VoidType => (create_class_object(jvm, int_state, ClassName::new("void").into(), BootstrapLoader)?, Arc::new(RuntimeClass::Void)),
        _ => todo!()
    };
    jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(class_object), ByAddress(runtime_class.clone()));
    Ok(runtime_class)
}

pub fn create_class_object(jvm: &JVMState, int_state: &mut InterpreterStateGuard, name: Option<ClassName>, loader: LoaderName) -> Result<Arc<Object>, WasException> {
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
        fields.insert("reflectionData".to_string(), JavaValue::Object(None));
        fields.insert("genericInfo".to_string(), JavaValue::Object(None));
        fields.insert("classRedefinedCount".to_string(), JavaValue::Int(0));
        return Ok(Arc::new(Object::Object(NormalObject {
            monitor: jvm.thread_state.new_monitor("object class object monitor".to_string()),
            fields: UnsafeCell::new(fields),
            class_pointer: jvm.classes.read().unwrap().class_class.clone(),
        })));
    }
    let class_object = match loader {
        LoaderName::UserDefinedLoader(_idx) => {
            JClass::new(jvm, int_state, loader_object.cast_class_loader())
        }
        BootstrapLoader => {
            JClass::new_bootstrap_loader(jvm, int_state)
        }
    }?;
    match name {
        None => {}
        Some(name) => {
            if jvm.include_name_field.load(Ordering::SeqCst) {
                class_object.set_name_(JString::from_rust(jvm, int_state, name.get_referred_name().replace("/", ".").to_string())?)
            }
        }
    }
    Ok(class_object.object())
}


pub fn check_resolved_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Result<Arc<RuntimeClass>, WasException> {
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