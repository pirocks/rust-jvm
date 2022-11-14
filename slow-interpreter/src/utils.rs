use std::intrinsics::transmute;
use std::sync::Arc;
use wtf8::Wtf8Buf;
use classfile_view::view::field_view::FieldView;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jfieldID, jint};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::classfile::{LineNumber, LineNumberTable};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::descriptor_parser::parse_field_descriptor;


use crate::{check_initing_or_inited_class, JString, JVMState, NewAsObjectOrJavaValue, NewJavaValue, OpaqueFrame, WasException};
use crate::better_java_stack::frame_iter::FrameIterFrameRef;
use crate::better_java_stack::frames::{PushableFrame};
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter::common::invoke::static_::invoke_static_impl;
use crate::interpreter::common::invoke::virtual_::invoke_virtual;
use crate::interpreter::common::ldc::load_class_constant_by_type;
use crate::java_values::{ExceptionReturn, JavaValue};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::array_out_of_bounds_exception::ArrayOutOfBoundsException;
use crate::stdlib::java::lang::boolean::Boolean;
use crate::stdlib::java::lang::byte::Byte;
use crate::stdlib::java::lang::char::Char;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::double::Double;
use crate::stdlib::java::lang::float::Float;
use crate::stdlib::java::lang::illegal_argument_exception::IllegalArgumentException;
use crate::stdlib::java::lang::int::Int;
use crate::stdlib::java::lang::long::Long;
use crate::stdlib::java::lang::reflect::field::Field;
use crate::stdlib::java::lang::short::Short;

pub fn lookup_method_parsed<'gc>(jvm: &'gc JVMState<'gc>, class: Arc<RuntimeClass<'gc>>, name: MethodName, descriptor: &CMethodDescriptor) -> Option<(u16, Arc<RuntimeClass<'gc>>)> {
    // dbg!(class.view().name().unwrap_name().0.to_str(&jvm.string_pool));
    lookup_method_parsed_impl(jvm, class, name, descriptor)
}

pub fn lookup_method_parsed_impl<'gc>(jvm: &'gc JVMState<'gc>, class: Arc<RuntimeClass<'gc>>, name: MethodName, descriptor: &CMethodDescriptor) -> Option<(u16, Arc<RuntimeClass<'gc>>)> {
    let view = class.view();
    // dbg!(view.name().unwrap_name().0.to_str(&jvm.string_pool));
    let posible_methods = view.lookup_method_name(name);
    let filtered = posible_methods.into_iter().filter(|m| if m.is_signature_polymorphic() { true } else { m.desc() == descriptor }).collect::<Vec<_>>();
    assert!(filtered.len() <= 1);
    match filtered.iter().next() {
        None => {
            let class_name = match class.view().super_name() {
                Some(x) => x,
                None => {
                    // dbg!(name.0.to_str(&jvm.string_pool));
                    // dbg!(descriptor.jvm_representation(&jvm.string_pool));
                    // dbg!(view.name().unwrap_name().0.to_str(&jvm.string_pool));
                    return None;
                }
            }; //todo is this unwrap safe?
            let lookup_type = CPDType::Class(class_name);
            let super_class = assert_inited_or_initing_class(jvm, lookup_type); //todo this unwrap could fail, and this should really be using check_inited_class
            match lookup_method_parsed_impl(jvm, super_class, name, descriptor) {
                None => {
                    for interface in class.view().interfaces() {
                        let interface_class = assert_inited_or_initing_class(jvm, interface.interface_name().into());
                        if let Some(res) = lookup_method_parsed_impl(jvm, interface_class, name, descriptor) {
                            return Some(res);
                        }
                    }
                    None
                }
                Some(res) => Some(res)
            }
        }
        Some(method_view) => Some((method_view.method_i(), class.clone())),
    }
}

pub fn throw_npe_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<T, WasException<'gc>> {
    let _ = throw_npe::<T>(jvm, int_state);
    Err(WasException { exception_obj: todo!() })
}

pub fn throw_npe<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> T {
    todo!()
    /*let npe_object = match NullPointerException::new(jvm, int_state) {
        Ok(npe) => npe,
        Err(WasException {}) => {
            eprintln!("Warning error encountered creating NPE");
            return T::invalid_default();
        }
    }
        .object()
        .into();
    int_state.set_throw(Some(npe_object));
    T::invalid_default()*/
}

pub fn throw_array_out_of_bounds_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, index: jint) -> Result<T, WasException<'gc>> {
    let mut throw = None;
    let _ = throw_array_out_of_bounds::<T>(jvm, int_state, &mut throw, index);
    Err(throw.unwrap())
}

pub fn throw_array_out_of_bounds<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, throw: &mut Option<WasException<'gc>>, index: jint) -> T {
    let bounds_object = match ArrayOutOfBoundsException::new(jvm, int_state, index) {
        Ok(npe) => npe,
        Err(WasException { exception_obj }) => {
            todo!();
            eprintln!("Warning error encountered creating Array out of bounds");
            return T::invalid_default();
        }
    }
        .object()
        .new_java_handle().unwrap_object_nonnull();
    *throw = Some(WasException{ exception_obj: bounds_object.cast_throwable() });
    T::invalid_default()
}

pub fn throw_illegal_arg_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<T, WasException<'gc>> {
    let _ = throw_illegal_arg::<T>(jvm, int_state);
    Err(WasException { exception_obj: todo!() })
}

pub fn throw_illegal_arg<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> T {
    let illegal_arg_object = match IllegalArgumentException::new(jvm, int_state) {
        Ok(illegal_arg) => illegal_arg,
        Err(WasException { exception_obj }) => {
            eprintln!("Warning error encountered creating illegal arg exception");
            return T::invalid_default();
        }
    }.object();
    todo!();// int_state.set_throw(Some(AllocatedHandle::NormalObject(illegal_arg_object)));
    T::invalid_default()
}

pub fn java_value_to_boxed_object<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, java_value: JavaValue<'gc>) -> Result<Option<AllocatedNormalObjectHandle<'gc>>, WasException<'gc>> {
    Ok(match java_value {
        //todo what about that same object optimization
        JavaValue::Long(param) => Long::new(jvm, int_state, param)?.object().into(),
        JavaValue::Int(param) => Int::new(jvm, int_state, param)?.object().into(),
        JavaValue::Short(param) => Short::new(jvm, int_state, param)?.object().into(),
        JavaValue::Byte(param) => Byte::new(jvm, int_state, param)?.object().into(),
        JavaValue::Boolean(param) => Boolean::new(jvm, int_state, param)?.object().into(),
        JavaValue::Char(param) => Char::new(jvm, int_state, param)?.object().into(),
        JavaValue::Float(param) => Float::new(jvm, int_state, param)?.object().into(),
        JavaValue::Double(param) => Double::new(jvm, int_state, param)?.object().into(),
        JavaValue::Object(obj) => todo!(), /*obj*/
        JavaValue::Top => panic!(),
    })
}

pub fn pushable_frame_todo<'any1, 'any2, 'any3>() -> &'any3 mut OpaqueFrame<'any1, 'any2> {
    todo!()
}

pub fn run_static_or_virtual<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    class: &Arc<RuntimeClass<'gc>>,
    method_name: MethodName,
    desc: &CMethodDescriptor,
    args: Vec<NewJavaValue<'gc, '_>>,
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException<'gc>> {
    let view = class.view();
    let res_fun = view.lookup_method(method_name, desc);
    let method_view = match res_fun {
        Some(x) => x,
        None => panic!(),
    };
    if method_view.is_static() {
        invoke_static_impl(jvm, int_state, desc, class.clone(), method_view.method_i(), &method_view, args)
    } else {
        // let (resolved_rc, method_i) = virtual_method_lookup(jvm, int_state, method_name, &desc, class.clone()).unwrap();
        // let view = resolved_rc.view();
        // let method_view = view.method_view_i(method_i);
        invoke_virtual(jvm, int_state, method_name, desc, args)
    }
}

pub fn unwrap_or_npe<'gc, 'l, T>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, to_unwrap: Option<T>) -> Result<T, WasException<'gc>> {
    match to_unwrap {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(unwrapped) => Ok(unwrapped),
    }
}

pub fn get_all_fields<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: Arc<RuntimeClass<'gc>>, include_interface: bool) -> Result<Vec<(Arc<RuntimeClass<'gc>>, usize)>, WasException<'gc>> {
    let mut res = vec![];
    get_all_fields_impl(jvm, int_state, class, &mut res, include_interface)?;
    Ok(res)
}

fn get_all_fields_impl<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: Arc<RuntimeClass<'gc>>, res: &mut Vec<(Arc<RuntimeClass<'gc>>, usize)>, include_interface: bool) -> Result<(), WasException<'gc>> {
    class.view().fields().enumerate().for_each(|(i, _)| {
        res.push((class.clone(), i));
    });

    match class.view().super_name() {
        None => {
            let object = check_initing_or_inited_class(jvm, int_state, CClassName::object().into())?;
            object.view().fields().enumerate().for_each(|(i, _)| {
                res.push((object.clone(), i));
            });
        }
        Some(super_name) => {
            let super_ = check_initing_or_inited_class(jvm, int_state, super_name.into())?;
            get_all_fields_impl(jvm, int_state, super_, res, include_interface)?
        }
    }

    if include_interface {
        for interface in class.view().interfaces() {
            let interface = check_initing_or_inited_class(jvm, int_state, interface.interface_name().into())?;
            interface.view().fields().enumerate().for_each(|(i, _)| {
                res.push((interface.clone(), i));
            });
        }
    }
    Ok(())
}

//shouldn't take class as arg and should be an impl method on Field
pub fn field_object_from_view<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    class_obj: Arc<RuntimeClass<'gc>>,
    f: FieldView,
) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let field_class_name_ = class_obj.clone().cpdtype();
    let parent_runtime_class = load_class_constant_by_type(jvm, int_state, field_class_name_)?;

    let field_name = f.field_name();

    let field_desc_str = f.field_desc();
    let field_type = parse_field_descriptor(field_desc_str.as_str()).unwrap().field_type;

    let signature = f.signature_attribute();

    let modifiers = f.access_flags() as i32;
    let slot = f.field_i() as i32;
    let clazz = parent_runtime_class.cast_class().expect("todo");
    let field_name_str = field_name.0.to_str(&jvm.string_pool);
    let name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(field_name_str))?.intern(jvm, int_state)?;
    let type_ = JClass::from_type(jvm, int_state, CPDType::from_ptype(&field_type, &jvm.string_pool))?;
    let signature = match signature {
        None => None,
        Some(signature) => Some(JString::from_rust(jvm, int_state, signature)?),
    };

    let annotations_ = vec![]; //todo impl annotations.

    Ok(Field::init(jvm, int_state, clazz, name, type_, modifiers, slot, signature, annotations_)?.new_java_value_handle())
}


pub fn get_all_methods<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: Arc<RuntimeClass<'gc>>, include_interface: bool) -> Result<Vec<(Arc<RuntimeClass<'gc>>, u16)>, WasException<'gc>> {
    let mut res = vec![];
    get_all_methods_impl(jvm, int_state, class, &mut res, include_interface)?;
    Ok(res)
}

fn get_all_methods_impl<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: Arc<RuntimeClass<'gc>>, res: &mut Vec<(Arc<RuntimeClass<'gc>>, u16)>, include_interface: bool) -> Result<(), WasException<'gc>> {
    class.view().methods().for_each(|m| {
        res.push((class.clone(), m.method_i()));
    });
    match class.view().super_name() {
        None => {
            let object = check_initing_or_inited_class(jvm, int_state, CClassName::object().into())?;
            object.view().methods().for_each(|m| {
                res.push((object.clone(), m.method_i()));
            });
        }
        Some(super_name) => {
            let super_ = check_initing_or_inited_class(jvm, int_state, super_name.into())?;
            get_all_methods_impl(jvm, int_state, super_, res, include_interface)?;
        }
    }
    if include_interface {
        let view = class.view();
        let interfaces = view.interfaces();
        for interface in interfaces {
            let interface = check_initing_or_inited_class(jvm, int_state, interface.interface_name().into())?;
            interface.view().methods().for_each(|m| {
                res.push((interface.clone(), m.method_i()));
            });
        }
    }
    Ok(())
}

pub fn new_field_id<'gc>(jvm: &'gc JVMState<'gc>, runtime_class: Arc<RuntimeClass<'gc>>, field_i: usize) -> jfieldID {
    let id = jvm.field_table.write().unwrap().register_with_table(runtime_class, field_i as u16);
    unsafe { transmute(id) }
}

pub fn lookup_line_number(line_number_table: &LineNumberTable, stack_entry: &FrameIterFrameRef) -> Option<LineNumber> {
    if let Some(pc) = stack_entry.try_pc() {
        return line_number_table.lookup_pc(pc)
    }
    None
}
