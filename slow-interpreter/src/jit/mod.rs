use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::AtomicPtr;

use wtf8::Wtf8Buf;

use another_jit_vm::IRMethodID;
use array_memory_layout::accessor::Accessor;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use gc_memory_layout_common::memory_regions::{AllocatedTypeID, RegionHeader};
use inheritance_tree::ClassID;
use jvmti_jni_bindings::jint;
use method_table::interface_table::InterfaceID;
use runtime_class_stuff::{RuntimeClass, RuntimeClassClass};
use runtime_class_stuff::field_numbers::FieldNameAndClass;
use runtime_class_stuff::method_numbers::MethodNumber;
use rust_jvm_common::{FieldId, MethodId};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::code::CompressedCode;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::method_shape::{MethodShape, MethodShapeID};
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;
use stage0::compiler_common::{MethodResolver, PartialYetAnotherLayoutImpl, YetAnotherLayoutImpl};

use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter::common::invoke::native::native_method_resolve;
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::jvm_state::JVMState;

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
pub struct MethodResolverImpl<'gc> {
    pub(crate) jvm: &'gc JVMState<'gc>,
    pub(crate) loader: LoaderName,
}

impl<'gc> MethodResolverImpl<'gc> {
    // fn lookup_interface_impl(&self, name: MethodName, desc: &CMethodDescriptor, rc: Arc<RuntimeClass<'gc>>) -> Option<(MethodId, bool)> {
    //     let view = rc.view();
    //     if let Some(parent_rc) = rc.unwrap_class_class().parent.as_ref() {
    //         if let Some(res) = self.lookup_interface_impl(name, desc, parent_rc.clone()) {
    //             return Some(res);
    //         }
    //     }
    //     for jni_interface in rc.unwrap_class_class().interfaces.iter() {
    //         if let Some(res) = self.lookup_interface_impl(name, desc, jni_interface.clone()) {
    //             return Some(res);
    //         }
    //     }
    //     let method_view = view.lookup_method(name, &desc)?;
    //     assert!(!method_view.is_static());
    //     let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
    //     Some((method_id, method_view.is_native()))
    // }

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

    fn lookup_method_number_recurse(&self, rc: &RuntimeClassClass, method_shape: MethodShape) -> MethodNumber {
        *match rc.method_numbers.get(&method_shape) {
            Some(x) => x,
            None => {
                dbg!(rc.class_view.name().jvm_representation(&self.jvm.string_pool));
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
}


impl<'gc> MethodResolver<'gc> for MethodResolverImpl<'gc> {
    fn lookup_static(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (_loader_name, rc) = classes_guard.get_loader_and_runtime_class(&on)?;
        let view = rc.view();
        let method_view = match view.lookup_method(name, &desc) {
            Some(x) => x,
            None => {
                let super_name = match view.super_name() {
                    Some(x) => x,
                    None => {
                        let string_pool = self.jvm.string_pool;
                        dbg!(on.jvm_representation(string_pool));
                        dbg!(name.0.to_str(string_pool));
                        dbg!(desc.jvm_representation(string_pool));
                        //todo I bet this is needs to go looking in interfaces as well
                        //todo needs to handle link_to_static etc.
                        todo!()

                    },
                };
                assert_inited_or_initing_class(self.jvm, super_name.clone().into());
                return self.lookup_static(super_name.into(), name, desc);
            }
        };
        assert!(method_view.is_static());
        let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
        Some((method_id, method_view.is_native()))
    }

    fn lookup_virtual(&self, _on: CPDType, name: MethodName, desc: CMethodDescriptor) -> MethodShapeID {
        self.jvm.method_shapes.lookup_method_shape_id(MethodShape { name, desc })
    }

    fn lookup_native_virtual(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<MethodId> {
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

    fn lookup_interface_id(&self, interface: CPDType) -> Option<InterfaceID> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (_, rc) = classes_guard.get_loader_and_runtime_class(&interface)?;
        Some(self.jvm.interface_table.get_interface_id(rc))
    }

    fn lookup_interface_class_id(&self, interface: CPDType) -> ClassID {
        self.jvm.class_ids.get_id_or_add(interface)
    }

    fn lookup_interface_method_number(&self, interface: CPDType, method_shape: MethodShape) -> Option<MethodNumber> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (_, rc) = classes_guard.get_loader_and_runtime_class(&interface)?;
        rc.unwrap_class_class().method_numbers.get(&method_shape).cloned()
    }

    fn lookup_special(&self, on: &CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (_loader_name, rc) = classes_guard.get_loader_and_runtime_class(on)?;
        self.lookup_special_impl(name, &desc, rc)
    }


    fn lookup_type_inited_initing(&self, cpdtype: &CPDType) -> Option<(Arc<RuntimeClass<'gc>>, LoaderName)> {
        let read_guard = self.jvm.classes.read().unwrap();
        let rc = read_guard.is_inited_or_initing(cpdtype)?;
        let loader = read_guard.get_initiating_loader(&rc);
        Some((rc, loader))
    }

    fn allocated_object_type_id(&self, rc: Arc<RuntimeClass<'gc>>, loader: LoaderName, arr_len: Option<jint>) -> AllocatedTypeID {
        let allocated_object_type = runtime_class_to_allocated_object_type(self.jvm, rc, loader, arr_len);
        let mut guard = self.jvm.gc.memory_region.lock().unwrap();
        guard.lookup_or_add_type(&allocated_object_type)
    }

    fn allocated_object_region_header_pointer(&self, id: AllocatedTypeID) -> *const AtomicPtr<RegionHeader> {
        self.jvm.gc.memory_region.lock().unwrap().get_region_header_raw_ptr(id)
    }

    fn lookup_method_layout(&self, method_id: usize) -> YetAnotherLayoutImpl {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let function_frame_type = &self.jvm.function_frame_type_data.read().unwrap().no_tops;
        let frames = function_frame_type.get(&method_id).unwrap();
        let code = method_view.code_attribute().unwrap();
        YetAnotherLayoutImpl::new(frames, code)
    }

    fn lookup_native_method_layout(&self, method_id: usize) -> NativeStackframeMemoryLayout {
        NativeStackframeMemoryLayout {
            num_locals: self.num_locals(method_id)
        }
    }

    fn lookup_partial_method_layout(&self, method_id: usize) -> PartialYetAnotherLayoutImpl {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        PartialYetAnotherLayoutImpl::new(code)
    }

    fn using_method_view_impl<T>(&self, method_id: MethodId, using: impl FnOnce(&MethodView) -> T) -> T {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        using(&method_view)
    }

    fn is_synchronized(&self, method_id: MethodId) -> bool {
        self.using_method_view_impl(method_id, |method_view| {
            method_view.is_synchronized()
        })
    }

    fn is_static(&self, method_id: MethodId) -> bool {
        self.using_method_view_impl(method_id, |method_view| {
            method_view.is_static()
        })
    }

    fn is_native(&self, method_id: MethodId) -> bool {
        self.using_method_view_impl(method_id, |method_view| {
            method_view.is_native()
        })
    }

    fn get_compressed_code(&self, method_id: MethodId) -> CompressedCode {
        self.using_method_view_impl(method_id, |method_view| {
            method_view.code_attribute().unwrap().clone()
        })
    }

    fn num_args(&self, method_id: MethodId) -> u16 {
        self.using_method_view_impl(method_id, |method_view| {
            method_view.num_args()
        })
    }

    fn num_locals(&self, method_id: MethodId) -> u16 {
        self.using_method_view_impl(method_id, |method_view| {
            assert!(method_view.is_native());
            method_view.desc().count_local_vars_needed() + if method_view.is_static() { 0 } else { 1 }
        } as u16)
    }

    fn lookup_method_desc(&self, method_id: MethodId) -> CMethodDescriptor {
        self.using_method_view_impl(method_id, |method_view| {
            method_view.desc().clone()
        })
    }

    fn lookup_ir_method_id_and_address(&self, method_id: MethodId) -> Option<(IRMethodID, *const c_void)> {
        let ir_method_id = self.jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 })?;
        let ptr = self.jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_id);
        Some((ir_method_id, ptr.as_ptr()))
    }

    fn get_field_id(&self, runtime_class: Arc<RuntimeClass<'gc>>, field_name: FieldName) -> FieldId {
        let view = runtime_class.view();
        let field_view = view.lookup_field(field_name).unwrap();
        self.jvm.field_table.write().unwrap().get_field_id(runtime_class, field_view.field_i())
    }

    fn get_cpdtype_id(&self, cpdtype: CPDType) -> CPDTypeID {
        self.jvm.cpdtype_table.write().unwrap().get_cpdtype_id(cpdtype)
    }

    fn get_commpressed_version_of_wtf8(&self, wtf8: &Wtf8Buf) -> CompressedWtf8String {
        self.jvm.wtf8_pool.add_entry(wtf8.clone())
    }

    fn lookup_method_shape(&self, method_shape: MethodShape) -> MethodShapeID {
        self.jvm.method_shapes.lookup_method_shape_id(method_shape)
    }

    fn lookup_method_number(&self, rc: Arc<RuntimeClass<'gc>>, method_shape: MethodShape) -> MethodNumber {
        let new_target_class = if rc.cpdtype().is_array() {
            let object_rc = assert_inited_or_initing_class(self.jvm, CClassName::object().into());
            object_rc //todo handle arrays being serializable and cloneable
        } else {
            rc
        };
        self.lookup_method_number_recurse(new_target_class.unwrap_class_class(), method_shape)
    }


    fn debug_checkcast_assertions(&self) -> bool {
        self.jvm.checkcast_debug_assertions
    }

    fn compile_interpreted(&self, method_id: MethodId) -> bool {
        self.jvm.config.compile_threshold > self.jvm.function_execution_count.function_instruction_count(method_id)
    }

    fn string_pool(&self) -> &CompressedClassfileStringPool {
        &self.jvm.string_pool
    }

    fn resolve_static_field<'l>(&self, runtime_class: &'l RuntimeClass<'gc>, field_name: FieldName) -> (&'l RuntimeClassClass<'gc>, NonNull<u64>, CPDType) {
        static_field_address(self.jvm, runtime_class, field_name)
    }

    fn is_direct_invoke(&self, class: Arc<RuntimeClass<'gc>>, method_name: MethodName, desc: CMethodDescriptor) -> Option<unsafe extern "C" fn()> {
        let class_name = class.unwrap_class_class().class_view.name().unwrap_name();
        let is_direct = self.jvm.direct_invoke_whitelist.is_direct_invoke_whitelisted(class_name, method_name, desc.clone());
        if !is_direct {
            return None
        }
        let class_view = class.view();
        let method_view = class_view.lookup_method(method_name, &desc).unwrap();
        Some(native_method_resolve(self.jvm, class, &method_view).unwrap())
    }
}


pub fn static_field_address<'gc, 'l>(jvm: &'gc JVMState<'gc>, runtime_class: &'l RuntimeClass<'gc>, field_name: FieldName) -> (&'l RuntimeClassClass<'gc>, NonNull<u64>, CPDType) {
    static_field_address_impl(jvm, runtime_class.unwrap_class_class(), field_name).unwrap()
}

pub fn static_field_address_impl<'gc, 'l>(jvm: &'gc JVMState<'gc>, class_class: &'l RuntimeClassClass<'gc>, field_name: FieldName) -> Option<(&'l RuntimeClassClass<'gc>, NonNull<u64>, CPDType)> {
    let class_name = class_class.class_view.name().unwrap_name();
    let static_field = jvm.all_the_static_fields.get(FieldNameAndClass { field_name, class_name });
    Some((class_class, static_field.raw_address().cast(), static_field.expected_type()))
}
