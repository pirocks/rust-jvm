use std::collections::BTreeSet;
use std::ops::Deref;
use std::sync::Arc;

use gc_memory_layout_common::memory_regions::AllocatedObjectType;
use inheritance_tree::ClassID;
use runtime_class_stuff::{RuntimeClass, RuntimeClassClass};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::CompressedParsedDescriptorType;
use rust_jvm_common::loading::LoaderName;

use crate::class_loading::assert_inited_or_initing_class;
use crate::jvm_state::JVMState;

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
            let (array_interfaces, interfaces_len) = jvm.interface_arrays.write().unwrap().add_interfaces(array_interfaces);
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
            let layout = &class_class.object_layout;
            let inheritance_bit_vec = class_class.inheritance_tree_vec.clone();


            let (interfaces, interfaces_len) = jvm.interface_arrays.write().unwrap().add_interfaces(all_interfaces_recursive(jvm, class_class));
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

