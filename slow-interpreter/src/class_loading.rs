use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use by_address::ByAddress;
use itertools::Itertools;

use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::{LivePoolGetter, LoaderName};
use rust_jvm_common::loading::LoaderName::BootstrapLoader;
use verification::{ClassFileGetter, VerifierContext, verify};

use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::class_not_found_exception::ClassNotFoundException;
use crate::java::lang::string::JString;
use crate::java_values::{GcManagedObject, JavaValue, NormalObject, Object, ObjectFieldsAndClass};
use crate::jvm_state::{ClassStatus, JVMState};
use crate::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassArray, RuntimeClassClass};

//todo only use where spec says
pub fn check_initing_or_inited_class(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
    let class = check_loaded_class(jvm, int_state, ptype.clone())?;
    match class.deref() {
        RuntimeClass::Byte => {
            check_initing_or_inited_class(jvm, int_state, CClassName::byte().into())?;
            return Ok(class);
        }
        RuntimeClass::Boolean => {
            check_initing_or_inited_class(jvm, int_state, CClassName::boolean().into())?;
            return Ok(class);
        }
        RuntimeClass::Short => {
            check_initing_or_inited_class(jvm, int_state, CClassName::short().into())?;
            return Ok(class);
        }
        RuntimeClass::Char => {
            check_initing_or_inited_class(jvm, int_state, CClassName::character().into())?;
            return Ok(class);
        }
        RuntimeClass::Int => {
            check_initing_or_inited_class(jvm, int_state, CClassName::int().into())?;
            return Ok(class);
        }
        RuntimeClass::Long => {
            check_initing_or_inited_class(jvm, int_state, CClassName::long().into())?;
            return Ok(class);
        }
        RuntimeClass::Float => {
            check_initing_or_inited_class(jvm, int_state, CClassName::float().into())?;
            return Ok(class);
        }
        RuntimeClass::Double => {
            check_initing_or_inited_class(jvm, int_state, CClassName::double().into())?;
            return Ok(class);
        }
        RuntimeClass::Void => {
            check_initing_or_inited_class(jvm, int_state, CClassName::void().into())?;
            return Ok(class);
        }
        RuntimeClass::Array(a) => {
            check_initing_or_inited_class(jvm, int_state, a.sub_class.cpdtype())?;
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

pub fn assert_loaded_class(jvm: &'gc_life JVMState<'gc_life>, ptype: CPDType) -> Arc<RuntimeClass<'gc_life>> {
    match jvm.classes.read().unwrap().initiating_loaders.get(&ptype) {
        None => panic!(),
        Some((_, res)) => res.clone()
    }
}

pub fn check_loaded_class(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
    let loader = int_state.current_loader();
    check_loaded_class_force_loader(jvm, int_state, &ptype, loader)
}

pub(crate) fn check_loaded_class_force_loader(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, ptype: &CPDType, loader: LoaderName) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
// todo cleanup how these guards work
    let guard = jvm.classes.write().unwrap();
    match guard.initiating_loaders.get(ptype) {
        None => {
            let res = match loader {
                LoaderName::UserDefinedLoader(loader_idx) => {
                    let loader_obj = jvm.class_loaders.write().unwrap().get_by_left(&loader_idx).unwrap().clone().0;
                    let class_loader: ClassLoader = JavaValue::Object(loader_obj.into()).cast_class_loader();
                    match ptype.clone() {
                        CPDType::ByteType => Arc::new(RuntimeClass::Byte),
                        CPDType::CharType => Arc::new(RuntimeClass::Char),
                        CPDType::DoubleType => Arc::new(RuntimeClass::Double),
                        CPDType::FloatType => Arc::new(RuntimeClass::Float),
                        CPDType::IntType => Arc::new(RuntimeClass::Int),
                        CPDType::LongType => Arc::new(RuntimeClass::Long),
                        CPDType::Ref(ref_) => {
                            match ref_ {
                                CPRefType::Class(class_name) => {
                                    drop(guard);
                                    let java_string = JString::from_rust(jvm, int_state, class_name.0.to_str(&jvm.string_pool).replace("/", ".").clone())?;
                                    class_loader.load_class(jvm, int_state, java_string)?.as_runtime_class(jvm)
                                }
                                CPRefType::Array(sub_type) => {
                                    drop(guard);
                                    let sub_class = check_loaded_class(jvm, int_state, sub_type.deref().clone())?;
                                    let res = Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class }));
                                    let obj = create_class_object(jvm, int_state, None, loader)?;
                                    jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(obj), ByAddress(res.clone()));
                                    res
                                }
                            }
                        }
                        CPDType::ShortType => Arc::new(RuntimeClass::Short),
                        CPDType::BooleanType => Arc::new(RuntimeClass::Boolean),
                        CPDType::VoidType => Arc::new(RuntimeClass::Void),
                    }
                }
                LoaderName::BootstrapLoader => {
                    drop(guard);
                    bootstrap_load(jvm, int_state, ptype.clone())?
                }
            };
            let mut guard = jvm.classes.write().unwrap();
            guard.initiating_loaders.entry(res.cpdtype()).insert((loader, res.clone()));
            guard.loaded_classes_by_type.entry(loader).or_insert(HashMap::new()).insert(res.cpdtype(), res.clone());
            Ok(res)
        }
        Some((_, res)) => Ok(res.clone())
    }
}

pub struct DefaultClassfileGetter<'l, 'k> {
    pub(crate) jvm: &'k JVMState<'l>,
}

impl ClassFileGetter for DefaultClassfileGetter<'_, '_> {
    fn get_classfile(&self, _loader: LoaderName, class: CClassName) -> Arc<Classfile> {
        //todo verification needs to be better hooked in
        match self.jvm.classpath.lookup(&class, &self.jvm.string_pool) {
            Ok(x) => x,
            Err(err) => {
                dbg!(err);
                dbg!(class);
                panic!()
            }
        }
    }
}

pub struct DefaultLivePoolGetter {}

impl LivePoolGetter for DefaultLivePoolGetter {
    fn elem_type(&self, _idx: usize) -> CPRefType {
        todo!()
    }
}

static mut BOOTSRAP_LOAD_COUNT: usize = 0;

pub fn bootstrap_load(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
    unsafe {
        BOOTSRAP_LOAD_COUNT += 1;
        if BOOTSRAP_LOAD_COUNT % 1000 == 0 {
            dbg!(BOOTSRAP_LOAD_COUNT);
        }
    }
    let (class_object, runtime_class) = match ptype.clone() {
        //todo replace these names with actual tupes
        CPDType::ByteType => (create_class_object(jvm, int_state, Some(ClassName::raw_byte().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Byte)),
        CPDType::CharType => (create_class_object(jvm, int_state, Some(ClassName::raw_char().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Char)),
        CPDType::DoubleType => (create_class_object(jvm, int_state, Some(ClassName::raw_double().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Double)),
        CPDType::FloatType => (create_class_object(jvm, int_state, Some(ClassName::raw_float().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Float)),
        CPDType::IntType => (create_class_object(jvm, int_state, Some(ClassName::raw_int().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Int)),
        CPDType::LongType => (create_class_object(jvm, int_state, Some(ClassName::raw_long().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Long)),
        CPDType::ShortType => (create_class_object(jvm, int_state, Some(ClassName::raw_short().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Short)),
        CPDType::BooleanType => (create_class_object(jvm, int_state, Some(ClassName::raw_boolean().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Boolean)),
        CPDType::VoidType => (create_class_object(jvm, int_state, Some(ClassName::raw_void().get_referred_name().to_string()), BootstrapLoader)?, Arc::new(RuntimeClass::Void)),
        CPDType::Ref(ref_) => match ref_ {
            CPRefType::Class(class_name) => {
                let classfile = match jvm.classpath.lookup(&class_name, &jvm.string_pool) {
                    Ok(x) => x,
                    Err(_) => {
                        let class_name_string = JString::from_rust(jvm, int_state, class_name.0.to_str(&jvm.string_pool).to_string())?;
                        let exception = ClassNotFoundException::new(jvm, int_state, class_name_string)?.object();
                        int_state.set_throw(exception.into());
                        return Err(WasException);
                    }
                };
                let class_view = Arc::new(ClassBackedView::from(classfile.clone(), &jvm.string_pool));
                let mut verifier_context = VerifierContext {
                    live_pool_getter: Arc::new(DefaultLivePoolGetter {}) as Arc<dyn LivePoolGetter>,
                    classfile_getter: Arc::new(DefaultClassfileGetter {
                        jvm
                    }) as Arc<dyn ClassFileGetter>,
                    string_pool: &jvm.string_pool,
                    method_descriptor_pool: &jvm.method_descriptor_pool,
                    current_loader: LoaderName::BootstrapLoader,
                    verification_types: Default::default(),
                    debug: class_name == CClassName::string(),
                };
                verify(&mut verifier_context, class_view.deref(), LoaderName::BootstrapLoader).unwrap();
                let parent = match class_view.super_name() {
                    Some(super_name) => {
                        Some(check_loaded_class(jvm, int_state, super_name.into())?)
                    }
                    None => None
                };
                let mut interfaces = vec![];
                for interface in class_view.interfaces() {
                    interfaces.push(check_loaded_class(jvm, int_state, interface.interface_name().into())?);
                }
                let start_field_number = parent.as_ref().map(|parent| parent.unwrap_class_class().num_vars()).unwrap_or(0);
                let field_numbers = class_view
                    .fields()
                    .filter(|field| !field.is_static())
                    .map(|name| (name.field_name(), name.field_type()))
                    .sorted_by_key(|(name, _ptype)| name.0)
                    .enumerate()
                    .map(|(index, (name, ptype))| (name, (index + start_field_number, ptype)))
                    .collect::<HashMap<_, _>>();
                let res = Arc::new(RuntimeClass::Object(RuntimeClassClass {
                    class_view: class_view.clone(),
                    field_numbers,
                    static_vars: Default::default(),
                    parent,
                    interfaces,
                    status: ClassStatus::UNPREPARED.into(),
                }));
                let verification_types = verifier_context.verification_types;
                jvm.sink_function_verification_date(&verification_types, res.clone());
                jvm.classes.write().unwrap().initiating_loaders.entry(ptype.clone()).or_insert((BootstrapLoader, res.clone()));
                let class_object = create_class_object(jvm, int_state, class_name.0.to_str(&jvm.string_pool).into(), BootstrapLoader)?;
                jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(class_object.clone()), ByAddress(res.clone()));
                (class_object, res)
            }
            CPRefType::Array(sub_type) => {
                let sub_class = check_resolved_class(jvm, int_state, sub_type.deref().clone())?;
                //todo handle class objects for arraus
                (create_class_object(jvm, int_state, None, BootstrapLoader)?, Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class })))
            }
        },
    };
    jvm.classes.write().unwrap().class_object_pool.insert(ByAddress(class_object), ByAddress(runtime_class.clone()));
    Ok(runtime_class)
}

pub fn create_class_object(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, name: Option<String>, loader: LoaderName) -> Result<GcManagedObject<'gc_life>, WasException> {
    let loader_object = match loader {
        LoaderName::UserDefinedLoader(idx) => {
            JavaValue::Object(jvm.class_loaders.read().unwrap().get_by_left(&idx).unwrap().clone().0.into())
        }
        LoaderName::BootstrapLoader => {
            JavaValue::null()
        }
    };
    if name == ClassName::object().get_referred_name().to_string().into() {
        return Ok(jvm.allocate_object(Object::Object(NormalObject {
            monitor: jvm.thread_state.new_monitor("object class object monitor".to_string()),
            objinfo: ObjectFieldsAndClass {
                fields: JVMState::get_class_field_numbers().into_values().map(|_| UnsafeCell::new(JavaValue::Top.to_native())).collect_vec(),
                class_pointer: jvm.classes.read().unwrap().class_class.clone(),
            },
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
    if let Some(name) = name {
        if jvm.include_name_field.load(Ordering::SeqCst) {
            class_object.set_name_(JString::from_rust(jvm, int_state, name.replace("/", ".").to_string())?)
        }
    }
    Ok(class_object.object())
}


pub fn check_resolved_class(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
    check_loaded_class(jvm, int_state, ptype)
}

pub fn assert_inited_or_initing_class(jvm: &'gc_life JVMState<'gc_life>, ptype: CPDType) -> Arc<RuntimeClass<'gc_life>> {
    let class: Arc<RuntimeClass<'gc_life>> = assert_loaded_class(jvm, ptype.clone());
    match class.status() {
        ClassStatus::UNPREPARED => panic!(),
        ClassStatus::PREPARED => panic!(),
        ClassStatus::INITIALIZING => class,
        ClassStatus::INITIALIZED => class,
    }
}