use std::ffi::c_void;
use std::sync::Arc;

use wtf8::Wtf8Buf;
use another_jit_vm::IRMethodID;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use gc_memory_layout_common::memory_regions::BaseAddressAndMask;
use runtime_class_stuff::{RuntimeClass, RuntimeClassClass};
use runtime_class_stuff::method_numbers::MethodNumber;
use rust_jvm_common::{FieldId, MethodId};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::code::CompressedCode;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::method_shape::{MethodShape, MethodShapeID};
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::class_loading::assert_inited_or_initing_class;
use crate::ir_to_java_layer::compiler::{PartialYetAnotherLayoutImpl, YetAnotherLayoutImpl};
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::jvm_state::JVMState;

pub mod ir;
pub mod state;

#[derive(Clone, Debug)]
pub struct ResolvedInvokeVirtual {
    pub address: *const c_void,
    pub ir_method_id: IRMethodID,
    pub method_id: MethodId,
    pub new_frame_size: usize,
}


#[derive(Debug, Copy, Clone)]
pub struct NotCompiledYet {
    pub needs_compiling: MethodId,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct CompiledCodeID(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IRInstructionIndex(u32);


#[derive(Clone, Copy)]
pub struct MethodResolver<'gc> {
    pub(crate) jvm: &'gc JVMState<'gc>,
    pub(crate) loader: LoaderName,
}


pub trait MethodResolverAndRecompileConditions {}


impl<'gc> MethodResolver<'gc> {
    pub fn lookup_static(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (loader_name, rc) = classes_guard.get_loader_and_runtime_class(&on)?;
        // assert_eq!(loader_name, self.loader);
        let view = rc.view();
        let string_pool = &self.jvm.string_pool;
        let method_view = match view.lookup_method(name, &desc) {
            Some(x) => x,
            None => {
                let super_name = view.super_name().unwrap();
                assert_inited_or_initing_class(self.jvm, super_name.clone().into());
                return self.lookup_static(super_name.into(), name, desc);
            }
        };
        assert!(method_view.is_static());
        let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
        Some((method_id, method_view.is_native()))
    }

    pub fn lookup_virtual(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> MethodShapeID {
        self.jvm.method_shapes.lookup_method_shape_id(MethodShape { name, desc })
    }

    pub fn lookup_native_virtual(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<MethodId> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (loader_name, rc) = classes_guard.get_loader_and_runtime_class(&on)?;
        assert_eq!(loader_name, self.loader);
        if name == MethodName::method_clone() {
            let object = assert_inited_or_initing_class(self.jvm, CPDType::object());
            let view = object.view();
            let method_views = view.lookup_method_name(MethodName::method_clone());
            let method_view = method_views.into_iter().next().unwrap();
            let method_id = self.jvm.method_table.write().unwrap().get_method_id(object, method_view.method_i());
            Some(method_id)
        } else {
            let view = rc.view();
            let method_view = view.lookup_method(name, &desc)?;
            assert!(!method_view.is_static());
            if !method_view.is_native() {
                return None;
            }
            let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
            Some(method_id)
        }
    }


    pub fn lookup_interface(&self, on: &CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (loader_name, rc) = classes_guard.get_loader_and_runtime_class(&on)?;
        // assert_eq!(loader_name, self.loader);
        Some(self.lookup_interface_impl(name, &desc, rc).unwrap())
    }

    fn lookup_interface_impl(&self, name: MethodName, desc: &CMethodDescriptor, rc: Arc<RuntimeClass<'gc>>) -> Option<(MethodId, bool)> {
        let view = rc.view();
        if let Some(parent_rc) = rc.unwrap_class_class().parent.as_ref() {
            if let Some(res) = self.lookup_interface_impl(name, desc, parent_rc.clone()) {
                return Some(res);
            }
        }
        for interface in rc.unwrap_class_class().interfaces.iter() {
            if let Some(res) = self.lookup_interface_impl(name, desc, interface.clone()) {
                return Some(res);
            }
        }
        let method_view = view.lookup_method(name, &desc)?;
        assert!(!method_view.is_static());
        let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
        Some((method_id, method_view.is_native()))
    }

    pub fn lookup_special(&self, on: &CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (loader_name, rc) = classes_guard.get_loader_and_runtime_class(on)?;
        self.lookup_special_impl(name, &desc, rc)
    }

    fn lookup_special_impl(&self, name: MethodName, desc: &CMethodDescriptor, rc: Arc<RuntimeClass<'gc>>) -> Option<(MethodId, bool)> {
        let view = rc.view();
        if let Some(method_view) = view.lookup_method(name, &desc) {
            assert!(!method_view.is_static());
            let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
            return Some((method_id, method_view.is_native()));
        }
        if let Some(parent_rc) = rc.unwrap_class_class().parent.as_ref() {
            if let Some(res) = self.lookup_special_impl(name, desc, parent_rc.clone()) {
                return Some(res);
            }
        }
        for interface in rc.unwrap_class_class().interfaces.iter() {
            if let Some(res) = self.lookup_special_impl(name, desc, interface.clone()) {
                return Some(res);
            }
        }
        None
    }

    pub fn lookup_type_inited_initing(&self, cpdtype: &CPDType) -> Option<(Arc<RuntimeClass<'gc>>, LoaderName)> {
        let read_guard = self.jvm.classes.read().unwrap();
        let rc = read_guard.is_inited_or_initing(cpdtype)?;
        let loader = read_guard.get_initiating_loader(&rc);
        Some((rc, loader))
    }

    pub fn lookup_method_layout(&self, methodid: usize) -> YetAnotherLayoutImpl {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let function_frame_type = self.jvm.function_frame_type_data_no_tops.read().unwrap();
        let frames = function_frame_type.get(&methodid).unwrap();
        let code = method_view.code_attribute().unwrap();
        YetAnotherLayoutImpl::new(frames, code)
    }

    pub fn lookup_partial_method_layout(&self, method_id: usize) -> PartialYetAnotherLayoutImpl {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        PartialYetAnotherLayoutImpl::new(code)
    }

    fn using_method_view_impl<T>(&self, method_id: MethodId, using: impl FnOnce(&MethodView) -> T) -> T{
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        using(&method_view)
    }

    pub fn is_synchronized(&self, method_id: MethodId) -> bool {
        self.using_method_view_impl(method_id,|method_view|{
            method_view.is_synchronized()
        })
    }

    pub fn is_static(&self, method_id: MethodId) -> bool {
        self.using_method_view_impl(method_id, |method_view|{
            method_view.is_static()
        })
    }

    pub fn get_compressed_code(&self, method_id: MethodId) -> CompressedCode {
        self.using_method_view_impl(method_id, |method_view|{
            method_view.code_attribute().unwrap().clone()
        })
    }

    pub fn num_args(&self, method_id: MethodId) -> u16 {
        self.using_method_view_impl(method_id, |method_view|{
            method_view.num_args()
        })
    }

    pub fn lookup_method_desc(&self, method_id: MethodId) -> CMethodDescriptor {
        self.using_method_view_impl(method_id, |method_view|{
            method_view.desc().clone()
        })
    }

    pub fn lookup_ir_method_id_and_address(&self, method_id: MethodId) -> Option<(IRMethodID, *const c_void)> {
        let ir_method_id = self.jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 })?;
        let ptr = self.jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_id);
        Some((ir_method_id, ptr.as_ptr()))
    }

    pub fn get_field_id(&self, runtime_class: Arc<RuntimeClass<'gc>>, field_name: FieldName) -> FieldId {
        let view = runtime_class.view();
        let field_view = view.lookup_field(field_name).unwrap();
        self.jvm.field_table.write().unwrap().get_field_id(runtime_class, field_view.field_i())
    }

    pub fn get_cpdtype_id(&self, cpdtype: CPDType) -> CPDTypeID {
        self.jvm.cpdtype_table.write().unwrap().get_cpdtype_id(cpdtype)
    }

    pub fn get_commpressed_version_of_wtf8(&self, wtf8: &Wtf8Buf) -> CompressedWtf8String {
        self.jvm.wtf8_pool.add_entry(wtf8.clone())
    }

    pub fn lookup_method_shape(&self, method_shape: MethodShape) -> MethodShapeID {
        self.jvm.method_shapes.lookup_method_shape_id(method_shape)
    }

    pub fn lookup_method_number(&self, rc: Arc<RuntimeClass<'gc>>, method_shape: MethodShape) -> MethodNumber {
        let new_target_class = if rc.cpdtype().is_array() {
            let object_rc = assert_inited_or_initing_class(self.jvm, CClassName::object().into());
            object_rc
        } else {
            rc
        };
        self.lookup_method_number_recurse(new_target_class.unwrap_class_class(), method_shape)
    }

    fn lookup_method_number_recurse(&self, rc: &RuntimeClassClass, method_shape: MethodShape) -> MethodNumber {
        *match rc.method_numbers.get(&method_shape) {
            Some(x) => x,
            None => {
                dbg!(method_shape.name.0.to_str(&self.jvm.string_pool));
                dbg!(method_shape.desc.jvm_representation(&self.jvm.string_pool));
                panic!()
                /*match rc.parent.as_ref() {
                    None => {
                        dbg!(method_shape.name.0.to_str(&self.jvm.string_pool));
                        dbg!(method_shape.desc.jvm_representation(&self.jvm.string_pool));
                        panic!()
                    }
                    Some(parent) => {
                        return self.lookup_method_number_recurse(parent.unwrap_class_class(), method_shape)
                    }
                }*/
            }
        }
    }

    pub fn known_addresses_for_type(&self, cpd_type: CPDType) -> Vec<BaseAddressAndMask> {
        self.jvm.known_addresses.known_addresses_for_type(cpd_type)
    }

    pub fn debug_checkcast_assertions(&self) -> bool {
        self.jvm.checkcast_debug_assertions
    }
}
