use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

use by_address::ByAddress;
use itertools::Itertools;
use wtf8::Wtf8Buf;

use another_jit_vm_ir::WasException;
use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use java5_verifier::type_infer;
use runtime_class_stuff::{ClassStatus, RuntimeClass, RuntimeClassArray, RuntimeClassClass};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::LiveObjectIndex;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::loading::{ClassLoadingError, LivePoolGetter, LoaderName};
use rust_jvm_common::loading::LoaderName::BootstrapLoader;
use stage0::compiler_common::frame_data::SunkVerifierFrames;
use verification::{ClassFileGetter, VerifierContext, verify};
use verification::verifier::TypeSafetyError;

use crate::{AllocatedHandle, JavaValueCommon, NewAsObjectOrJavaValue, UnAllocatedObject};
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::class_not_found_exception::ClassNotFoundException;
use crate::java::lang::string::JString;
use crate::java_values::{ByAddressAllocatedObject, default_value};
use crate::jit::MethodResolverImpl;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::new_java_values::unallocated_objects::UnAllocatedObjectObject;
use crate::runtime_class::{initialize_class, prepare_class, static_vars};

//todo only use where spec says
pub fn check_initing_or_inited_class<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc>>, WasException> {
    let class = check_loaded_class(jvm, int_state, ptype.clone())?;
    match class.clone().deref() {
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
            let loader_name = int_state.current_loader(jvm);
            jvm.classes.write().unwrap().initiating_loaders.insert(ptype, (loader_name, class.clone()));
        }
        _ => {}
    }
    match class.status() {
        ClassStatus::UNPREPARED => {
            prepare_class(jvm, int_state, class.view(), &mut static_vars(class.deref(), jvm));
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

pub fn assert_loaded_class<'gc>(jvm: &'gc JVMState<'gc>, ptype: CPDType) -> Arc<RuntimeClass<'gc>> {
    try_assert_loaded_class(jvm, ptype).unwrap()
}

pub fn try_assert_loaded_class<'gc>(jvm: &'gc JVMState<'gc>, ptype: CPDType) -> Option<Arc<RuntimeClass<'gc>>> {
    jvm.classes.read().unwrap().is_loaded(&ptype)
}

pub fn check_loaded_class<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc>>, WasException> {
    let loader = int_state.current_loader(jvm);
    assert!(jvm.thread_state.int_state_guard_valid.with(|valid| valid.borrow().clone()));
    check_loaded_class_force_loader(jvm, int_state, &ptype, loader)
}

pub(crate) fn check_loaded_class_force_loader<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, ptype: &CPDType, loader: LoaderName) -> Result<Arc<RuntimeClass<'gc>>, WasException> {
    // todo cleanup how these guards work
    let is_loaded = jvm.classes.write().unwrap().is_loaded(ptype);
    let res = match is_loaded {
        None => {
            let res = match loader {
                LoaderName::UserDefinedLoader(loader_idx) => {
                    let loader_obj = jvm.classes.read().unwrap().lookup_class_loader(loader_idx).clone();
                    let class_loader: ClassLoader = loader_obj.duplicate_discouraged().cast_class_loader();
                    match ptype.clone() {
                        CPDType::ByteType => Arc::new(RuntimeClass::Byte),
                        CPDType::CharType => Arc::new(RuntimeClass::Char),
                        CPDType::DoubleType => Arc::new(RuntimeClass::Double),
                        CPDType::FloatType => Arc::new(RuntimeClass::Float),
                        CPDType::IntType => Arc::new(RuntimeClass::Int),
                        CPDType::LongType => Arc::new(RuntimeClass::Long),
                        CPDType::Class(class_name) => {
                            let java_string = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(class_name.0.to_str(&jvm.string_pool).replace("/", ".").clone()))?;
                            class_loader.load_class(jvm, int_state, java_string)?.as_runtime_class(jvm)
                        }
                        CPDType::Array { base_type: sub_type, num_nested_arrs } => {
                            drop(jvm.classes.write().unwrap());
                            let sub_class = check_loaded_class(jvm, int_state, sub_type.to_cpdtype())?;
                            let serializable = check_loaded_class(jvm, int_state, CClassName::serializable().into())?;
                            let cloneable = check_loaded_class(jvm, int_state, CClassName::cloneable().into())?;
                            let res = Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class, serializable, cloneable }));
                            let obj = create_class_object(jvm, int_state, None, loader)?.duplicate_discouraged();
                            jvm.classes.write().unwrap().class_object_pool.insert(ByAddressAllocatedObject::Owned(obj.duplicate_discouraged()), ByAddress(res.clone()));
                            assert_eq!(obj.runtime_class(jvm).cpdtype(), CClassName::class().into());
                            res
                        }
                        CPDType::ShortType => Arc::new(RuntimeClass::Short),
                        CPDType::BooleanType => Arc::new(RuntimeClass::Boolean),
                        CPDType::VoidType => Arc::new(RuntimeClass::Void),
                    }
                }
                LoaderName::BootstrapLoader => {
                    bootstrap_load(jvm, int_state, ptype.clone())?
                }
            };
            let mut guard = jvm.classes.write().unwrap();
            guard.initiating_loaders.insert(res.cpdtype(), (loader, res.clone()));
            guard.loaded_classes_by_type.entry(loader).or_insert(HashMap::new()).insert(res.cpdtype(), res.clone());
            Ok(res)
        }
        Some(res) => Ok(res.clone()),
    }?;
    // jvm.inheritance_ids.write().unwrap().register(jvm, &res);
    // jvm.vtables.write().unwrap().notify_load(jvm, res.clone());

    Ok(res)
}

pub struct DefaultClassfileGetter<'l, 'k> {
    pub(crate) jvm: &'k JVMState<'l>,
}

impl ClassFileGetter for DefaultClassfileGetter<'_, '_> {
    fn get_classfile(&self, vf_context: &VerifierContext, _loader: LoaderName, class: CClassName) -> Result<Arc<dyn ClassView>, ClassLoadingError> {
        //todo verification needs to be better hooked in
        Ok(match self.jvm.classpath.lookup(&class, &self.jvm.string_pool) {
            Ok(x) => Arc::new(ClassBackedView::from(x, &self.jvm.string_pool)),
            Err(err) => {
                eprintln!("WARN: CLASS NOT FOUND WHILE VERIFYING:");
                dbg!(vf_context.current_class.0.to_str(&vf_context.string_pool));
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

pub fn bootstrap_load<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc>>, WasException> {
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
        CPDType::Class(class_name) => {
            let classfile = match jvm.classpath.lookup(&class_name, &jvm.string_pool) {
                Ok(x) => x,
                Err(_) => {
                    let class_name_wtf8 = Wtf8Buf::from_string(class_name.0.to_str(&jvm.string_pool).to_string());
                    let class_name_string = JString::from_rust(jvm, int_state, class_name_wtf8)?;

                    let exception = ClassNotFoundException::new(jvm, int_state, class_name_string)?.full_object();
                    let throwable = exception.cast_throwable();
                    throwable.print_stack_trace(jvm,int_state).unwrap();
                    int_state.set_throw(Some(throwable.full_object()));
                    return Err(WasException {});
                }
            };
            let class_view = Arc::new(ClassBackedView::from(classfile.clone(), &jvm.string_pool));
            let parent = match class_view.super_name() {
                Some(super_name) => Some(check_loaded_class(jvm, int_state, super_name.into())?),
                None => None,
            };
            let mut interfaces = vec![];
            for interface in class_view.interfaces() {
                interfaces.push(check_loaded_class(jvm, int_state, interface.interface_name().into())?);
            }

            let res = Arc::new(RuntimeClass::Object(
                RuntimeClassClass::new_new(&jvm.inheritance_tree, &mut jvm.bit_vec_paths.write().unwrap(), class_view.clone(), parent, interfaces, ClassStatus::UNPREPARED.into(), &jvm.string_pool, &jvm.class_ids)
            ));
            let mut verifier_context = VerifierContext {
                live_pool_getter: Arc::new(DefaultLivePoolGetter {}) as Arc<dyn LivePoolGetter>,
                classfile_getter: Arc::new(DefaultClassfileGetter { jvm }) as Arc<dyn ClassFileGetter>,
                string_pool: &jvm.string_pool,
                current_class: class_name,
                class_view_cache: Mutex::new(Default::default()),
                current_loader: LoaderName::BootstrapLoader,
                verification_types: Default::default(),
                debug: class_name == CClassName::string(),
                perf_metrics: &jvm.perf_metrics,
                permissive_types_workaround: false,
            };


            match verify(&mut verifier_context, class_name, LoaderName::BootstrapLoader) {
                Ok(verfied) => {
                    let verification_types = verifier_context.verification_types;
                    jvm.sink_function_verification_date(&verification_types, res.clone());
                }
                Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassNotFoundException(class_name))) => {
                    let jstring = JString::from_rust(jvm,int_state, Wtf8Buf::from_string(class_name.get_referred_name().clone())).unwrap();
                    let exception = ClassNotFoundException::new(jvm, int_state, jstring)?.full_object();
                    let throwable = exception.cast_throwable();
                    throwable.print_stack_trace(jvm,int_state).unwrap();
                    int_state.set_throw(Some(throwable.full_object()));
                    return Err(WasException{});
                }
                Err(TypeSafetyError::NotSafe(not_safe)) => {
                    dbg!(class_name.0.to_str(&jvm.string_pool));
                    dbg!(not_safe);
                    panic!()
                }
                Err(TypeSafetyError::Java5Maybe) => {
                    //todo major dup
                    for method_view in class_view.clone().methods() {
                        let method_id = jvm.method_table.write().unwrap().get_method_id(res.clone(), method_view.method_i());
                        let code = match method_view.code_attribute() {
                            Some(x) => x,
                            None => continue,
                        };
                        let instructs = code.instructions.iter().sorted_by_key(|(offset, instruct)| *offset).map(|(_, instruct)| instruct.clone()).collect_vec();
                        let res = type_infer(&method_view);
                        let frames_tops = res.inferred_frames().iter().map(|(offset, frame)| {
                            (*offset, SunkVerifierFrames::PartialInferredFrame(frame.clone()))
                        }).collect::<HashMap<_, _>>();
                        let frames_no_tops = res.inferred_frames().iter().map(|(offset, frame)| {
                            (*offset, SunkVerifierFrames::PartialInferredFrame(frame.no_tops()))
                        }).collect::<HashMap<_, _>>();
                        jvm.function_frame_type_data.write().unwrap().no_tops.insert(method_id, frames_no_tops);
                        jvm.function_frame_type_data.write().unwrap().tops.insert(method_id, frames_tops);
                    }
                }
                Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassFileInvalid(_))) => {
                    panic!()
                }
                Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassVerificationError)) => panic!(),
            };
            let method_resolver = MethodResolverImpl { jvm, loader: LoaderName::BootstrapLoader };
            // for method in class_view.methods() {
            //     if method.code_attribute().is_some() {
            //         let method_id = jvm.method_table.write().unwrap().get_method_id(res.clone(), method.method_i());
            //         jvm.java_vm_state.add_method(jvm, &method_resolver, method_id)
            //     }
            // }
            jvm.classes.write().unwrap().initiating_loaders.entry(ptype.clone()).or_insert((BootstrapLoader, res.clone()));
            let class_object = create_class_object(jvm, int_state, class_name.0.to_str(&jvm.string_pool).into(), BootstrapLoader)?;
            jvm.classes.write().unwrap().class_object_pool.insert(ByAddressAllocatedObject::Owned(class_object.clone()), ByAddress(res.clone()));
            assert_eq!(class_object.runtime_class(jvm).cpdtype(), CClassName::class().into());
            (class_object, res)
        }
        CPDType::Array { base_type: sub_type, num_nested_arrs } => {
            let sub_class = check_resolved_class(jvm, int_state, ptype.unwrap_array_type())?;
            let serializable = check_resolved_class(jvm, int_state, CClassName::serializable().into())?;
            let cloneable = check_resolved_class(jvm, int_state, CClassName::cloneable().into())?;
            //todo handle class objects for arraus
            (create_class_object(jvm, int_state, None, BootstrapLoader)?, Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class, serializable, cloneable })))
        }
    };
    jvm.classes.write().unwrap().class_object_pool.insert(ByAddressAllocatedObject::Owned(class_object.duplicate_discouraged()), ByAddress(runtime_class.clone()));
    assert_eq!(class_object.runtime_class(jvm).cpdtype(), CClassName::class().into());
    Ok(runtime_class)
}


pub fn get_static_var_types(class_view: &ClassBackedView) -> HashMap<FieldName, CPDType> {
    class_view.fields().filter(|field| field.is_static()).map(|field| (field.field_name(), field.field_type())).collect()
}

//signature here is prov best, b/c returning handle is very messy, and handle can just be put in lives for gc_life static vec
pub fn create_class_object<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, name: Option<String>, loader: LoaderName) -> Result<&'gc AllocatedNormalObjectHandle<'gc>, WasException> {
    let loader_object = match loader {
        LoaderName::UserDefinedLoader(idx) => NewJavaValueHandle::Object(AllocatedHandle::NormalObject(jvm.classes.read().unwrap().lookup_class_loader(idx).duplicate_discouraged())),
        LoaderName::BootstrapLoader => NewJavaValueHandle::null(),
    };
    if name == ClassName::object().get_referred_name().to_string().into() {
        let fields_handles = JVMState::get_class_class_field_numbers().into_values().map(|(field_number, type_)| (field_number, default_value(type_))).collect::<Vec<_>>();
        let fields = fields_handles.iter().map(|(field_number, handle)| (*field_number, handle.as_njv())).collect();
        let new_allocated_object_handle = jvm.allocate_object(UnAllocatedObject::Object(UnAllocatedObjectObject { object_rc: jvm.classes.read().unwrap().class_class.clone(), fields }));
        let allocated_object = jvm.gc.handle_lives_for_gc_life(new_allocated_object_handle.unwrap_normal_object());
        return Ok(allocated_object);
    }
    let class_object = match loader {
        LoaderName::UserDefinedLoader(_idx) => JClass::new(jvm, int_state, loader_object.cast_class_loader()),
        BootstrapLoader => JClass::new_bootstrap_loader(jvm, int_state),
    }?;
    if let Some(name) = name {
        if jvm.include_name_field.load(Ordering::SeqCst) {
            class_object.set_name_(jvm, JString::from_rust(jvm, int_state, Wtf8Buf::from_string(name.replace("/", ".").to_string()))?)
        }
    }
    Ok(class_object.object_gc_life(jvm))
}

pub fn check_resolved_class<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, ptype: CPDType) -> Result<Arc<RuntimeClass<'gc>>, WasException> {
    check_loaded_class(jvm, int_state, ptype)
}

pub fn assert_inited_or_initing_class<'gc>(jvm: &'gc JVMState<'gc>, ptype: CPDType) -> Arc<RuntimeClass<'gc>> {
    let class: Arc<RuntimeClass<'gc>> = assert_loaded_class(jvm, ptype.clone());
    match class.status() {
        ClassStatus::UNPREPARED => {
            dbg!(ptype.jvm_representation(&jvm.string_pool));
            // jvm.perf_metrics.display();
            panic!()
        }
        ClassStatus::PREPARED => panic!(),
        ClassStatus::INITIALIZING => class,
        ClassStatus::INITIALIZED => class,
    }
}