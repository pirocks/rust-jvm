use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use by_address::ByAddress;
use itertools::Itertools;
use wtf8::Wtf8Buf;
use classfile_view::view::{ClassBackedView, ClassView};
use java5_verifier::type_infer;
use runtime_class_stuff::{ClassStatus, RuntimeClass, RuntimeClassClass};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::loading::{ClassLoadingError, ClassWithLoader, LoaderName};
use compiler_common::frame_data::SunkVerifierFrames;
use verification::{VerifierContext, verify};
use verification::verifier::TypeSafetyError;
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::{check_initing_or_inited_class, ClassIntrinsicsData, create_class_object};
use crate::class_objects::get_or_create_class_object_force_loader;
use crate::exceptions::WasException;
use crate::java_values::ByAddressAllocatedObject;
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::runtime_class::{initialize_class, prepare_class};
use crate::static_vars::static_vars;
use crate::stdlib::java::lang::class_not_found_exception::ClassNotFoundException;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::utils::pushable_frame_todo;

pub fn define_class_safe<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    parsed: Arc<Classfile>,
    current_loader: LoaderName,
    class_view: ClassBackedView,
) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let class_name = class_view.name().unwrap_name();
    let class_view = Arc::new(class_view);
    let super_class = class_view.super_name().map(|name| check_initing_or_inited_class(jvm, int_state, name.into()).unwrap());
    let interfaces = class_view.interfaces().map(|interface| check_initing_or_inited_class(jvm, int_state, interface.interface_name().into()).unwrap()).collect_vec();
    let runtime_class = Arc::new(RuntimeClass::Object(
        RuntimeClassClass::new_new(&jvm.inheritance_tree, &jvm.all_the_static_fields, &mut jvm.bit_vec_paths.write().unwrap(), class_view.clone(), super_class, interfaces, RwLock::new(ClassStatus::UNPREPARED), &jvm.string_pool, &jvm.class_ids)
    ));
    jvm.classpath.class_cache.write().unwrap().insert(class_view.name().unwrap_name(), parsed.clone());
    let mut class_view_cache = HashMap::new();
    class_view_cache.insert(ClassWithLoader { class_name, loader: current_loader }, class_view.clone() as Arc<dyn ClassView>);
    let mut vf = VerifierContext {
        live_pool_getter: jvm.get_live_object_pool_getter(),
        classfile_getter: jvm.get_class_getter(int_state.current_loader(jvm)),
        string_pool: &jvm.string_pool,
        current_class: class_name,
        class_view_cache: Mutex::new(class_view_cache),
        current_loader: LoaderName::BootstrapLoader, //todo
        verification_types: Default::default(),
        debug: false,
        perf_metrics: &jvm.perf_metrics,
        permissive_types_workaround: false,
    };
    match verify(&mut vf, class_name, LoaderName::BootstrapLoader /*todo*/) {
        Ok(_) => {
            jvm.sink_function_verification_date(&vf.verification_types, runtime_class.clone());
        }
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassNotFoundException(class_name))) => {
            let class = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_str(class_name.get_referred_name()))?;
            let to_throw = ClassNotFoundException::new(jvm, int_state, class)?.object().new_java_handle().unwrap_object().unwrap();
            return Err(WasException { exception_obj: to_throw.cast_throwable() });
        }
        Err(TypeSafetyError::NotSafe(msg)) => {
            dbg!(&msg);
            panic!()
        }
        Err(TypeSafetyError::Java5Maybe) => {
            //todo check for privileged here
            for method_view in class_view.methods() {
                let method_id = jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method_view.method_i());
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
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassFileInvalid(_))) => panic!(),
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassVerificationError)) => panic!(),
    };
    let class_object = create_class_object(jvm, int_state, None, current_loader, ClassIntrinsicsData {
        is_array: false,
        is_primitive: false,
        component_type: None,
        this_cpdtype: class_name.into(),
    })?;
    let mut classes = jvm.classes.write().unwrap();
    classes.anon_classes.push(runtime_class.clone());
    classes.initiating_loaders.insert(class_name.clone().into(), (current_loader, runtime_class.clone()));
    classes.loaded_classes_by_type.entry(current_loader).or_insert(HashMap::new()).insert(class_name.clone().into(), runtime_class.clone());
    classes.class_object_pool.insert(ByAddressAllocatedObject::Owned(class_object.duplicate_discouraged()), ByAddress(runtime_class.clone()));
    drop(classes);
    assert_eq!(class_object.runtime_class(jvm).cpdtype(), CClassName::class().into());
    prepare_class(jvm, int_state, Arc::new(ClassBackedView::from(parsed.clone(), &jvm.string_pool)), &mut static_vars(runtime_class.deref(), jvm));
    runtime_class.set_status(ClassStatus::PREPARED);
    runtime_class.set_status(ClassStatus::INITIALIZING);
    initialize_class(runtime_class.clone(), jvm, int_state)?;
    runtime_class.set_status(ClassStatus::INITIALIZED);
    Ok(get_or_create_class_object_force_loader(jvm, class_name.into(), int_state, current_loader).unwrap().new_java_handle())
}

