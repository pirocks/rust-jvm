use std::collections::BTreeSet;
use std::ops::Deref;
use std::sync::Arc;

use itertools::Itertools;

use gc_memory_layout_common::layout::ObjectMemoryLayout;
use gc_memory_layout_common::memory_regions::AllocatedObjectType;
use inheritance_tree::ClassID;
use runtime_class_stuff::{RuntimeClass, RuntimeClassClass};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;

use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_state::InterpreterStateGuard;
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;

#[derive(Debug, Copy, Clone)]
pub struct Opaque {}


pub fn runtime_class_to_allocated_object_type<'gc>(jvm: &'gc JVMState<'gc>, ref_type: Arc<RuntimeClass<'gc>>, loader: LoaderName, arr_len: Option<usize>) -> AllocatedObjectType {
    let itable = jvm.itables.lock().unwrap().lookup_or_new_itable(&jvm.interface_table, ref_type.clone());
    match ref_type.deref() {
        RuntimeClass::Byte => panic!(),
        RuntimeClass::Boolean => panic!(),
        RuntimeClass::Short => panic!(),
        RuntimeClass::Char => panic!(),
        RuntimeClass::Int => panic!(),
        RuntimeClass::Long => panic!(),
        RuntimeClass::Float => panic!(),
        RuntimeClass::Double => panic!(),
        RuntimeClass::Void => panic!(),
        RuntimeClass::Array(arr) => {
            let cloneable_id = jvm.class_ids.get_id_or_add(arr.cloneable.cpdtype());
            let serializable_id = jvm.class_ids.get_id_or_add(arr.serializable.cpdtype());
            let array_interfaces = BTreeSet::from([cloneable_id, serializable_id]);
            let (array_interfaces, interfaces_len) = leak_interface_array(array_interfaces);
            let object_vtable = jvm.vtables.lock().unwrap().lookup_or_new_vtable(assert_inited_or_initing_class(jvm, CClassName::object().into()));
            let primitive_type = match arr.sub_class.deref() {
                RuntimeClass::Byte => CompressedParsedDescriptorType::ByteType,
                RuntimeClass::Boolean => CompressedParsedDescriptorType::BooleanType,
                RuntimeClass::Short => CompressedParsedDescriptorType::ShortType,
                RuntimeClass::Char => CompressedParsedDescriptorType::CharType,
                RuntimeClass::Int => CompressedParsedDescriptorType::IntType,
                RuntimeClass::Long => CompressedParsedDescriptorType::LongType,
                RuntimeClass::Float => CompressedParsedDescriptorType::FloatType,
                RuntimeClass::Double => CompressedParsedDescriptorType::DoubleType,
                RuntimeClass::Void => panic!(),
                RuntimeClass::Object(_) | RuntimeClass::Array(_) => {
                    return AllocatedObjectType::ObjectArray {
                        sub_type: arr.sub_class.cpdtype().unwrap_ref_type().clone(),
                        len: arr_len.unwrap() as i32,
                        sub_type_loader: loader,
                        object_vtable,
                        array_itable: itable,
                        array_interfaces,
                        interfaces_len,
                    };
                }
                RuntimeClass::Top => panic!(),
            };

            AllocatedObjectType::PrimitiveArray {
                primitive_type,
                len: arr_len.unwrap() as i32,
                object_vtable,
                array_itable: itable,
                array_interfaces,
                interfaces_len,
            }
        }
        RuntimeClass::Object(class_class) => {
            let layout = ObjectMemoryLayout::from_rc(class_class);
            let inheritance_bit_vec = class_class.inheritance_tree_vec.clone();


            let (interfaces, interfaces_len) = leak_interface_array(all_interfaces_recursive(jvm, class_class));
            AllocatedObjectType::Class {
                name: class_class.class_view.name().unwrap_name(),
                loader,
                size: layout.size(),
                vtable: jvm.vtables.lock().unwrap().lookup_or_new_vtable(ref_type.clone()),
                itable,
                inheritance_bit_vec,
                interfaces,
                interfaces_len,
            }
        }
        RuntimeClass::Top => panic!(),
    }
}

pub fn leak_interface_array(interfaces: BTreeSet<ClassID>) -> (*const ClassID, usize) {
    let mut interfaces = interfaces.into_iter().collect_vec();
    interfaces.shrink_to_fit();
    let (ptr, len, capacity) = interfaces.into_raw_parts();
    assert_eq!(len, capacity);
    (ptr, len)
}

pub fn all_interfaces_recursive<'gc>(jvm: &'gc JVMState<'gc>, rc: &RuntimeClassClass<'gc>) -> BTreeSet<ClassID> {
    let mut res = BTreeSet::new();
    if let Some(super_class) = rc.parent.as_ref() {
        res.extend(all_interfaces_recursive(jvm, super_class.unwrap_class_class()).into_iter());
    }
    for interface in rc.interfaces.iter() {
        res.insert(jvm.class_ids.get_id_or_add(interface.cpdtype()));
        res.extend(all_interfaces_recursive(jvm, interface.unwrap_class_class()).into_iter());
    }
    res
}


pub fn setup_args_from_current_frame<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, 'l>, desc: &CMethodDescriptor, is_virtual: bool) -> Vec<NewJavaValueHandle<'gc>> {
    if is_virtual {
        todo!()
    }
    let mut args = vec![];
    for (i, _) in desc.arg_types.iter().enumerate() {
        let current_frame = int_state.current_frame();
        let operand_stack = current_frame.operand_stack(jvm);
        let types_ = operand_stack.types();
        let operand_stack_i = types_.len() - 1 - i;
        let jv = operand_stack.get(operand_stack_i as u16, types_[operand_stack_i].clone());
        args.push(jv);
    }
    args
}