use std::sync::Arc;

use classfile_view::loading::{ClassLoadingError, LoaderName};
use classfile_view::view::{ClassView, HasAccessFlags};
use descriptor_parser::parse_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::instructions::invoke::special::invoke_special_impl;
use crate::java_values::{default_value, JavaValue, Object};
use crate::runtime_class::RuntimeClass;

//todo jni should really live in interpreter state

pub fn push_new_object(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    target_classfile: &Arc<RuntimeClass>
) {
    let loader_arc = todo!()/*target_classfile.loader()*/;//&int_state.current_frame().class_pointer().loader(jvm).clone();//todo fix loaders.
    let object_pointer = JavaValue::new_object(jvm, target_classfile.clone());
    let new_obj = JavaValue::Object(object_pointer.clone());
    default_init_fields(jvm, int_state, loader_arc.clone(), object_pointer, target_classfile.view()).unwrap();
    int_state.current_frame_mut().push(new_obj);
}

fn default_init_fields(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    loader: LoaderName,
    object_pointer: Option<Arc<Object>>,
    view: &ClassView,
) -> Result<(), ClassLoadingError> {
    if let Some(super_name) = view.super_name() {
        let loaded_super = assert_inited_or_initing_class(jvm, int_state, super_name.into());//todo this shouldn't be doing any loading so its okay to not override loader
        default_init_fields(jvm, int_state, loader.clone(), object_pointer.clone(), &loaded_super.view());
    }
    for field in view.fields() {
        if !field.is_static() {
            //todo should I look for constant val attributes?
            /*let _value_i = match field.constant_value_attribute() {
                None => {}
                Some(_i) => _i,
            };*/
            let name = field.field_name();
            let type_ = field.field_type();
            let val = default_value(type_);
            {
                object_pointer.clone().unwrap().unwrap_normal_object().fields_mut().insert(name, val);
            }
        }
    }
    Ok(())
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

// pub fn check_inited_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Result<Arc<RuntimeClass>, ClassLoadingError> {
//     if let Some(class_) = jvm.classes.read().unwrap().is_loaded(&ptype) {
//         return Ok(class_)
//     }
//     check_inited_class_override_loader(jvm, int_state, &ptype, int_state.current_loader())
// }


// pub fn check_inited_class_override_loader(
//     jvm: &JVMState,
//     int_state: &mut InterpreterStateGuard,
//     ptype: &PTypeView,
//     loader: LoaderName,
// ) -> Result<Arc<RuntimeClass>, ClassLoadingError> {
//     //todo racy/needs sychronization
//     let before = int_state.int_state.as_ref().unwrap().call_stack.len();
//     if ptype == &ClassName::Str("javax/swing/JFrame".to_string()).into() {
//         dbg!(&loader);
//     }
//     let maybe_status = jvm.classes.read().unwrap().get_status(loader.clone(), ptype.clone());
//     let res = match maybe_status {
//         None => {
//             unknown_class_load(jvm, int_state, ptype, loader.clone())
//         }
//         Some(status) => match status {
//             ClassStatus::PREPARED => from_prepared_to_inited(jvm, int_state, &ptype, loader.clone()), //todo this is wrong?
//             ClassStatus::INITIALIZING => return Ok(jvm.classes.read().unwrap().get_initializing_class(loader, ptype)),
//             ClassStatus::INITIALIZED => return Ok(jvm.classes.read().unwrap().get_initialized_class(loader, ptype))
//         }
//     }?;
//     //todo race?
//     jvm.classes.write().unwrap().transition_initialized(loader, res.clone());
//     let after = int_state.int_state.as_ref().unwrap().call_stack.len();
//     assert_eq!(after, before);
//     Ok(res)
// }
//
// fn from_prepared_to_inited(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: &&PTypeView, loader: LoaderName) -> Result<Arc<RuntimeClass>, ClassLoadingError> {
//     if let Some(class_name) = ptype.unwrap_type_to_name() {
//         jvm.tracing.trace_class_loads(&class_name)
//     }
//     Ok(match &ptype {
//         PTypeView::Ref(ref_) => match ref_ {
//             ReferenceTypeView::Class(class_name) => {
//                 check_inited_class_impl(jvm, int_state, class_name, loader.clone())?
//             }
//             ReferenceTypeView::Array(arr) => {
//                 let array_type_class = check_inited_class_override_loader(jvm, int_state, arr.deref(), loader.clone())?;
//                 Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class: array_type_class, loader: loader.clone() }))
//             }
//         },
//         PTypeView::TopType | PTypeView::NullType | PTypeView::Uninitialized(_) | PTypeView::UninitializedThis |
//         PTypeView::UninitializedThisOrClass(_) => panic!(),
//         _ => panic!()
//     })
// }
//
// fn unknown_class_load(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: &PTypeView, loader: LoaderName) -> Result<Arc<RuntimeClass>, ClassLoadingError> {
//     let runtime_class = match ptype {
//         PTypeView::ByteType => {
//             // move init init
//             Arc::new(RuntimeClass::Byte)
//         }
//         PTypeView::CharType => {
//             // move init init
//             Arc::new(RuntimeClass::Char)
//         }
//         PTypeView::DoubleType => {
//             // move init init
//             Arc::new(RuntimeClass::Double)
//         }
//         PTypeView::FloatType => {
//             // move init init
//             Arc::new(RuntimeClass::Float)
//         }
//         PTypeView::IntType => {
//             // move init init
//             Arc::new(RuntimeClass::Int)
//         }
//         PTypeView::LongType => {
//             // move init init
//             Arc::new(RuntimeClass::Long)
//         }
//         PTypeView::ShortType => {
//             // move init init
//             Arc::new(RuntimeClass::Short)
//         }
//         PTypeView::BooleanType => {
//             // move init init
//             Arc::new(RuntimeClass::Boolean)
//         }
//         PTypeView::VoidType => {
//             // move init init
//             Arc::new(RuntimeClass::Void)
//         }
//         PTypeView::Ref(ref_) => match ref_ {
//             ReferenceTypeView::Class(class_name) => {
//                 if class_name == &ClassName::raw_byte() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::ByteType, loader)
//                 } else if class_name == &ClassName::raw_char() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::CharType, loader)
//                 } else if class_name == &ClassName::raw_double() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::DoubleType, loader)
//                 } else if class_name == &ClassName::raw_float() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::FloatType, loader)
//                 } else if class_name == &ClassName::raw_int() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::IntType, loader)
//                 } else if class_name == &ClassName::raw_long() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::LongType, loader)
//                 } else if class_name == &ClassName::raw_short() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::ShortType, loader)
//                 } else if class_name == &ClassName::raw_boolean() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::BooleanType, loader)
//                 } else if class_name == &ClassName::raw_void() {
//                     check_inited_class_override_loader(jvm, int_state, &PTypeView::VoidType, loader)
//                 } else {
//                     //todo call ClassLoader.loadClass
//                     check_inited_class_impl(jvm, int_state, class_name, loader)
//                 }?
//                 //todo handle standard case loading of completely unloaded class
//             }
//             ReferenceTypeView::Array(arr) => {
//                 let init = check_inited_class_override_loader(jvm, int_state, arr.deref(), loader.clone())?;
//                 Arc::new(RuntimeClass::Array(RuntimeClassArray { sub_class: init, loader: loader.clone() }))
//             }
//         }
//         PTypeView::TopType | PTypeView::NullType | PTypeView::Uninitialized(_) | PTypeView::UninitializedThis | PTypeView::UninitializedThisOrClass(_) => panic!()
//     };
//     Ok(runtime_class)
// }
//
// fn check_inited_class_impl(
//     jvm: &JVMState,
//     int_state: &mut InterpreterStateGuard,
//     class_name: &ClassName,
//     loader_name: LoaderName,
// ) -> Result<Arc<RuntimeClass>, ClassLoadingError> {
//     // if class_name.get_referred_name().as_str() == "java/lang/invoke/DirectMethodHandle$Lazy"{
//     //     unsafe { breakpoint(); }
//     // }
//     return match loader_name {
//         LoaderName::UserDefinedLoader(idx) => {
//             dbg!(loader_name);
//             let loader: ClassLoader = JavaValue::Object(jvm.class_loaders.read().unwrap().get_by_left(&idx).unwrap().0.clone().into()).cast_class_loader();
//             dbg!(loader.to_string(jvm, int_state).to_rust_string());
//             let class_name_as_jstring = JString::from_rust(jvm, int_state, class_name.get_referred_name().to_string().replace("/", "."));
//             let res = loader.load_class(jvm, int_state, class_name_as_jstring);
//             Ok(Arc::new(res.as_runtime_class(jvm).with_different_loader(loader_name)))
//             // let class_ = jvm.classes.read().unwrap().get_initializing_class(LoaderName::BootstrapLoader, &class_name.clone().into());
//             // let new_class = Arc::new(class_.with_different_loader(loader_name));
//             // jvm.classes.write().unwrap().transition_initializing(loader_name, new_class.clone());
//             // jvm.classes.write().unwrap().transition_initialized(loader_name, new_class.clone());
//             // todo need to make a copy of runtime class and change loader to correct.
//             // jvm.classes.write().unwrap().initiating_loaders.entry(class_name.clone().into()).or_insert(loader_name);
//             // Ok(new_class)
//         }
//         LoaderName::BootstrapLoader => {
//             find_class_from_bootloader(jvm, int_state, loader_name, class_name.clone())
//         }
//     };
// }
//
//
// pub fn find_class_from_bootloader(jvm: &JVMState, int_state: &mut InterpreterStateGuard, loader_name: LoaderName, class_name: ClassName) -> Result<Arc<RuntimeClass>, ClassLoadingError> {
//     let target_classfile = match jvm.classpath.lookup(&class_name) {
//         Ok(target_classfile) => target_classfile,
//         Err(err) => {
//             if class_name == ClassName::new("sun/dc/DuctusRenderingEngine") {
//                 let jclass_name = JString::from_rust(jvm, int_state, class_name.get_referred_name().to_string());
//                 let class_not_found_exception = class_not_found_exception::ClassNotFoundException::new(jvm, int_state, jclass_name);
//                 int_state.set_throw(class_not_found_exception.object().into());
//                 return Err(err);
//             } else {
//                 dbg!(class_name);
//                 int_state.print_stack_trace();
//                 panic!()
//             }
//         }
//     };
//     if let Err(_) = verify(&VerifierContext {
//         live_pool_getter: jvm.get_live_object_pool_getter(),
//         classfile_getter: jvm.get_class_getter(LoaderName::BootstrapLoader),
//         current_loader: loader_name,
//     }, &Arc::new(ClassView::from(target_classfile.clone())), loader_name) {
//         panic!()
//     }
//     let prepared = Arc::new(prepare_class(jvm, target_classfile.clone(), loader_name));
//     if let Some(super_name) = prepared.view().super_name() {
//         dbg!(&super_name);
//         dbg!(int_state.current_loader());
//         check_inited_class(jvm, int_state, super_name.into())?;
//     };
//     jvm.classes.write().unwrap().transition_prepared(loader_name, prepared.clone());
//     jvm.classes.write().unwrap().transition_initializing(loader_name, prepared.clone());
//     if let Some(jvmti) = &jvm.jvmti_state {
//         jvmti.built_in_jdwp.class_prepare(jvm, &class_name, int_state);
//     }
//
//     let inited_target = initialize_class(prepared.clone(), jvm, int_state);
//     if inited_target.is_none() {
//         return Err(ClassLoadingError::ClassNotFoundException);
//     }
//     jvm.classes.write().unwrap().transition_initialized(loader_name, prepared.clone());
//     Ok(prepared)
// }