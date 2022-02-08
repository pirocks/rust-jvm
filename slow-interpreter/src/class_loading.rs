use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::Ordering;

use by_address::ByAddress;
use iced_x86::OpCodeOperandKind::cl;
use itertools::Itertools;
use wtf8::Wtf8Buf;

use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::LiveObjectIndex;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::loading::{ClassLoadingError, LivePoolGetter, LoaderName};
use rust_jvm_common::loading::LoaderName::BootstrapLoader;
use verification::{ClassFileGetter, VerifierContext, verify};
use verification::verifier::TypeSafetyError;

use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::class_not_found_exception::ClassNotFoundException;
use crate::java::lang::string::JString;
use crate::java_values::{ByAddressAllocatedObject, default_value, GcManagedObject, JavaValue, NormalObject, Object, ObjectFieldsAndClass};
use crate::jit::MethodResolver;
use crate::jvm_state::{ClassStatus, JVMState};
use crate::new_java_values::{AllocatedObject, UnAllocatedObject, UnAllocatedObjectObject};
use crate::NewJavaValue;
use crate::runtime_class::{FieldNumber, initialize_class, prepare_class, RuntimeClass, RuntimeClassArray, RuntimeClassClass};

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
            prepare_class(jvm, int_state, class.view(), &mut class.static_vars(jvm));
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
            // assert!(int_state.throw().is_none());
            let res = initialize_class(class, jvm, int_state)?;
            res.set_status(ClassStatus::INITIALIZED);
            Ok(res)
        }
        ClassStatus::INITIALIZING => Ok(class),
        ClassStatus::INITIALIZED => Ok(class),
    }
}

pub fn assert_loaded_class(jvm: &'gc_life JVMState<'gc_life>, ptype: CPDType) -> Arc<RuntimeClass<'gc_life>> {
    jvm.classes.read().unwrap().is_loaded(&ptype).unwrap()
}

pub fn check_loaded_class(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
    let loader = int_state.current_loader(jvm);
    assert!(jvm.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
    check_loaded_class_force_loader(jvm, int_state, &ptype, loader)
}

pub(crate) fn check_loaded_class_force_loader(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, ptype: &CPDType, loader: LoaderName) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
    // todo cleanup how these guards work
    let guard = jvm.classes.write().unwrap();
    let res = match guard.is_loaded(ptype) {
        None => {
            let res = match loader {
                LoaderName::UserDefinedLoader(loader_idx) => {
                    let loader_obj = jvm.classes.read().unwrap().lookup_class_loader(loader_idx).clone();
                    let class_loader: ClassLoader = NewJavaValue::AllocObject(loader_obj.clone().into()).to_jv().cast_class_loader();
                    match ptype.clone() {
                        CPDType::ByteType => Arc::new(RuntimeClass::Byte),
                        CPDType::CharType => Arc::new(RuntimeClass::Char),
                        CPDType::DoubleType => Arc::new(RuntimeClass::Double),
                        CPDType::FloatType => Arc::new(RuntimeClass::Float),
                        CPDType::IntType => Arc::new(RuntimeClass::Int),
                        CPDType::LongType => Arc::new(RuntimeClass::Long),
                        CPDType::Ref(ref_) => match ref_ {
                            CPRefType::Class(class_name) => {
                                drop(guard);
                                let java_string = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(class_name.0.to_str(&jvm.string_pool).replace("/", ".").clone()))?;
                                let res = class_loader.load_class(jvm, int_state, java_string)?.as_runtime_class(jvm);
                                res
                            }
                            CPRefType::Array(sub_type) => {
                                drop(guard);
                                let sub_class = check_loaded_class(jvm, int_state, sub_type.deref().clone())?;
                                let res = Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class }));
                                let obj = create_class_object(jvm, int_state, None, loader)?;
                                jvm.classes.write().unwrap().class_object_pool.insert(ByAddressAllocatedObject(obj), ByAddress(res.clone()));
                                res
                            }
                        },
                        CPDType::ShortType => Arc::new(RuntimeClass::Short),
                        CPDType::BooleanType => Arc::new(RuntimeClass::Boolean),
                        CPDType::VoidType => Arc::new(RuntimeClass::Void),
                    }
                }
                LoaderName::BootstrapLoader => {
                    drop(guard);
                    let res = bootstrap_load(jvm, int_state, ptype.clone())?;
                    res
                }
            };
            let mut guard = jvm.classes.write().unwrap();
            guard.initiating_loaders.entry(res.cpdtype()).insert((loader, res.clone()));
            guard.loaded_classes_by_type.entry(loader).or_insert(HashMap::new()).insert(res.cpdtype(), res.clone());
            Ok(res)
        }
        Some(res) => Ok(res.clone()),
    }?;
    jvm.inheritance_ids.write().unwrap().register(jvm, &res);
    Ok(res)
}

pub struct DefaultClassfileGetter<'l, 'k> {
    pub(crate) jvm: &'k JVMState<'l>,
}

impl ClassFileGetter for DefaultClassfileGetter<'_, '_> {
    fn get_classfile(&self, _loader: LoaderName, class: CClassName) -> Result<Arc<dyn ClassView>, ClassLoadingError> {
        //todo verification needs to be better hooked in
        Ok(match self.jvm.classpath.lookup(&class, &self.jvm.string_pool) {
            Ok(x) => Arc::new(ClassBackedView::from(x, &self.jvm.string_pool)),
            Err(err) => {
                eprintln!("WARN: CLASS NOT FOUND WHILE VERIFYING:");
                dbg!(&err);
                dbg!(class.0.to_str(&self.jvm.string_pool));
                return Err(err);
            }
        })
    }
}

pub struct DefaultLivePoolGetter {}

impl LivePoolGetter for DefaultLivePoolGetter {
    fn elem_type(&self, _idx: LiveObjectIndex) -> CPRefType {
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
                        let class_name_wtf8 = Wtf8Buf::from_string(class_name.0.to_str(&jvm.string_pool).to_string());
                        let class_name_string = todo!()/*JString::from_rust(jvm, int_state, class_name_wtf8)?*/;

                        let exception = todo!()/*ClassNotFoundException::new(jvm, int_state, class_name_string)?.object()*/;
                        int_state.set_throw(Some(todo!()/*exception.into()*/));
                        return Err(WasException);
                    }
                };
                let class_view = Arc::new(ClassBackedView::from(classfile.clone(), &jvm.string_pool));
                let mut verifier_context = VerifierContext {
                    live_pool_getter: Arc::new(DefaultLivePoolGetter {}) as Arc<dyn LivePoolGetter>,
                    classfile_getter: Arc::new(DefaultClassfileGetter { jvm }) as Arc<dyn ClassFileGetter>,
                    string_pool: &jvm.string_pool,
                    class_view_cache: Mutex::new(Default::default()),
                    current_loader: LoaderName::BootstrapLoader,
                    verification_types: Default::default(),
                    debug: class_name == CClassName::string(),
                };
                match verify(&mut verifier_context, class_name, LoaderName::BootstrapLoader) {
                    Ok(_) => {}
                    Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassNotFoundException(_))) => {
                        return Err(WasException);
                    }
                    Err(TypeSafetyError::NotSafe(_)) => panic!(),
                    Err(TypeSafetyError::Java5Maybe) => panic!(),
                    Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassFileInvalid(_))) => {
                        panic!()
                    }
                    Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassVerificationError)) => panic!(),
                };
                let parent = match class_view.super_name() {
                    Some(super_name) => Some(check_loaded_class(jvm, int_state, super_name.into())?),
                    None => None,
                };
                let mut interfaces = vec![];
                for interface in class_view.interfaces() {
                    interfaces.push(check_loaded_class(jvm, int_state, interface.interface_name().into())?);
                }
                let (recursive_num_fields, field_numbers) = get_field_numbers(&class_view, &parent);
                let static_var_types = get_static_var_types(class_view.deref());
                let res = Arc::new(RuntimeClass::Object(
                    RuntimeClassClass::new(class_view, field_numbers, recursive_num_fields, Default::default(), parent, interfaces, ClassStatus::UNPREPARED.into(), static_var_types))
                );
                let verification_types = verifier_context.verification_types;
                jvm.sink_function_verification_date(&verification_types, res.clone());
                let method_resolver = MethodResolver { jvm, loader: LoaderName::BootstrapLoader };
                // for method in class_view.methods() {
                //     if method.code_attribute().is_some() {
                //         let method_id = jvm.method_table.write().unwrap().get_method_id(res.clone(), method.method_i());
                //         jvm.java_vm_state.add_method(jvm, &method_resolver, method_id)
                //     }
                // }
                jvm.classes.write().unwrap().initiating_loaders.entry(ptype.clone()).or_insert((BootstrapLoader, res.clone()));
                let class_object = create_class_object(jvm, int_state, class_name.0.to_str(&jvm.string_pool).into(), BootstrapLoader)?;
                jvm.classes.write().unwrap().class_object_pool.insert(ByAddressAllocatedObject(class_object.clone()), ByAddress(res.clone()));
                (class_object, res)
            }
            CPRefType::Array(sub_type) => {
                let sub_class = check_resolved_class(jvm, int_state, sub_type.deref().clone())?;
                //todo handle class objects for arraus
                (create_class_object(jvm, int_state, None, BootstrapLoader)?, Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class })))
            }
        },
    };
    jvm.classes.write().unwrap().class_object_pool.insert(ByAddressAllocatedObject(class_object), ByAddress(runtime_class.clone()));
    Ok(runtime_class)
}

pub fn get_field_numbers(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> (usize, HashMap<FieldName, (FieldNumber, CompressedParsedDescriptorType)>) {
    let start_field_number = parent.as_ref().map(|parent| parent.unwrap_class_class().num_vars()).unwrap_or(0);
    let field_numbers = class_view.fields().filter(|field| !field.is_static()).map(|name| (name.field_name(), name.field_type())).sorted_by_key(|(name, _ptype)| name.0).enumerate().map(|(index, (name, ptype))| (name, (FieldNumber(index + start_field_number), ptype))).collect::<HashMap<_, _>>();
    (start_field_number + field_numbers.len(), field_numbers)
}

pub fn get_static_var_types(class_view: &ClassBackedView) -> HashMap<FieldName, CPDType> {
    class_view.fields().filter(|field|field.is_static()).map(|field|(field.field_name(),field.field_type())).collect()
}

//signature here is prov best, b/c returning handle is very messy, and handle can just be put in lives for gc_life static vec
pub fn create_class_object(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, name: Option<String>, loader: LoaderName) -> Result<AllocatedObject<'gc_life,'gc_life>, WasException> {
    let loader_object = match loader {
        LoaderName::UserDefinedLoader(idx) => JavaValue::Object(jvm.classes.read().unwrap().lookup_class_loader(idx).clone().to_gc_managed().into()),
        LoaderName::BootstrapLoader => JavaValue::null(),
    };
    if name == ClassName::object().get_referred_name().to_string().into() {
        let fields_handles = JVMState::get_class_field_numbers().into_values().map(|(field_number, type_)| (field_number, default_value(type_))).collect::<Vec<_>>();
        let fields = fields_handles.iter().map(|(field_number, handle)|(*field_number, handle.as_njv())).collect();
        let new_allocated_object_handle = jvm.allocate_object(UnAllocatedObject::Object(UnAllocatedObjectObject { object_rc: jvm.classes.read().unwrap().class_class.clone(), fields }));
        let allocated_object = jvm.gc.handle_lives_for_gc_life(new_allocated_object_handle);
        return Ok(allocated_object);
    }
    let class_object = match loader {
        LoaderName::UserDefinedLoader(_idx) => JClass::new(jvm, int_state, loader_object.cast_class_loader()),
        BootstrapLoader => JClass::new_bootstrap_loader(jvm, int_state),
    }?;
    if let Some(name) = name {
        if jvm.include_name_field.load(Ordering::SeqCst) {
            class_object.set_name_(JString::from_rust(jvm, int_state, Wtf8Buf::from_string(name.replace("/", ".").to_string()))?)
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