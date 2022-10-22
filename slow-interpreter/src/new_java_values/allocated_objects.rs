use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr::NonNull;
use std::sync::Arc;
use gc_memory_layout_common::allocated_object_types::AllocatedObjectType;
use gc_memory_layout_common::layout::ArrayMemoryLayout;

use runtime_class_stuff::{FieldNumberAndFieldType, RuntimeClass};
use runtime_class_stuff::field_numbers::FieldNumber;
use runtime_class_stuff::hidden_fields::HiddenJVMField;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use rust_jvm_common::NativeJavaValue;

use crate::{JavaValue, JVMState, NewJavaValue, NewJavaValueHandle};
use crate::class_loading::assert_loaded_class;
use crate::java_values::{array_native_to_new_java_value, GcManagedObject, native_to_new_java_value};
use crate::new_java_values::java_value_common::JavaValueCommon;

impl<'gc> Clone for AllocatedNormalObjectHandle<'gc> {
    fn clone(&self) -> Self {
        self.duplicate_discouraged()
    }
}

pub struct AllocatedArrayObjectHandle<'gc> {
    pub(crate) jvm: &'gc JVMState<'gc>,
    pub ptr: NonNull<c_void>,
}


impl Drop for AllocatedArrayObjectHandle<'_> {
    fn drop(&mut self) {
        self.jvm.gc.deregister_root_reentrant(self.ptr)
    }
}

impl<'gc> AllocatedArrayObjectHandle<'gc> {
    pub fn allocated_type(&self) -> AllocatedObjectType {
        let jvm = self.jvm;
        let ptr = self.ptr;
        jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(ptr).clone()
    }

    fn array_layout(&self) -> ArrayMemoryLayout{
        let allocated_type = self.allocated_type();
        ArrayMemoryLayout::from_cpdtype(self.elem_cpdtype())
    }

    pub fn len(&self) -> i32 {
        let memory_layout = self.array_layout();
        let res = *unsafe { memory_layout.calculate_len_address(self.ptr).as_ref() };
        assert!(res >= 0);
        res
    }

    pub fn elem_cpdtype(&self) -> CPDType {
        let allocated_type = self.allocated_type();
        match allocated_type {
            AllocatedObjectType::Class { .. } => {
                panic!()
            }
            AllocatedObjectType::ObjectArray { sub_type, .. } => {
                sub_type.to_cpdtype()
            }
            AllocatedObjectType::PrimitiveArray { primitive_type, .. } => {
                primitive_type
            }
            AllocatedObjectType::Raw { .. } => {
                panic!()
            }
        }
    }

    pub fn get_i(&self, i: i32) -> NewJavaValueHandle<'gc> {
        assert!(i < self.len());
        let jvm = self.jvm;
        let ptr = self.ptr;
        let array_layout = self.array_layout();
        let native_jv_ptr = array_layout.calculate_index_address(ptr, i);
        let cpdtype = self.elem_cpdtype();
        unsafe { array_native_to_new_java_value(native_jv_ptr.as_ref(), cpdtype, jvm) }
    }

    pub fn set_i(&self, i: i32, elem: NewJavaValue<'gc, '_>) {
        assert!(i < self.len());
        let ptr = self.ptr;
        let array_layout = self.array_layout();
        let mut native_jv_ptr = array_layout.calculate_index_address(ptr, i);
        unsafe { elem.set_array_native(native_jv_ptr.as_mut()) }
    }

    pub fn array_iterator<'l>(&'l self) -> ArrayIterator<'gc, 'l> {
        ArrayIterator {
            i: 0,
            array_wrapper: self,
        }
    }
}

pub struct ArrayIterator<'gc, 'l> {
    i: i32,
    array_wrapper: &'l AllocatedArrayObjectHandle<'gc>,
}

impl<'gc, 'l> Iterator for ArrayIterator<'gc, 'l> {
    type Item = NewJavaValueHandle<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.array_wrapper.len() {
            return None;
        }
        let res = self.array_wrapper.get_i(self.i);
        self.i += 1;
        Some(res)
    }
}


pub struct AllocatedNormalObjectHandle<'gc> {
    pub(crate) jvm: &'gc JVMState<'gc>,
    pub ptr: NonNull<c_void>,
}

impl Drop for AllocatedNormalObjectHandle<'_> {
    fn drop(&mut self) {
        self.jvm.gc.deregister_root_reentrant(self.ptr)
    }
}

impl<'gc> AllocatedNormalObjectHandle<'gc> {
    pub fn to_jv(&self) -> JavaValue<'gc> {
        todo!()
    }

    pub fn new_java_handle(self) -> NewJavaValueHandle<'gc> {
        NewJavaValueHandle::Object(AllocatedHandle::NormalObject(self))
    }

    //todo dup
    pub fn runtime_class(&self, jvm: &'gc JVMState<'gc>) -> Arc<RuntimeClass<'gc>> {
        let allocated_obj_type = jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(self.ptr).clone();
        assert_loaded_class(jvm, allocated_obj_type.as_cpdtype())
    }

    pub fn new_java_value<'l>(&'l self) -> NewJavaValue<'gc, 'l> {
        NewJavaValue::AllocObject(AllocatedObject::NormalObject(self))
    }

    pub fn to_gc_managed(&self) -> GcManagedObject<'gc> {
        todo!()
    }

    pub fn raw_ptr_usize(&self) -> usize {
        self.ptr.as_ptr() as usize
    }

    fn raw_set_var<'any>(ptr: NonNull<c_void>, field_number: FieldNumber, val: NewJavaValue<'gc, 'any>) {
        unsafe {
            ptr.cast::<NativeJavaValue<'gc>>().as_ptr().offset(field_number.0 as isize).write(val.to_native());
        }
    }

    pub fn set_var_hidden<'any>(&self, current_class_pointer: &Arc<RuntimeClass<'gc>>, field_name: HiddenJVMField, val: NewJavaValue<'gc, 'any>) {
        let field_number = current_class_pointer.unwrap_class_class().object_layout.hidden_field_numbers.get(&field_name).unwrap().number;
        Self::raw_set_var(self.ptr, field_number, val)
    }

    pub fn set_var<'any>(&self, current_class_pointer: &Arc<RuntimeClass<'gc>>, field_name: FieldName, val: NewJavaValue<'gc, 'any>) {
        let field_number = current_class_pointer.unwrap_class_class().object_layout.field_numbers.get(&field_name).unwrap().number;
        Self::raw_set_var(self.ptr, field_number, val)
    }

    pub fn set_var_top_level<'any>(&self, jvm: &'gc JVMState<'gc>, field_name: FieldName, val: NewJavaValue<'gc, 'any>) {
        let current_class_pointer = self.runtime_class(jvm);
        self.set_var(&current_class_pointer, field_name, val)
    }

    pub fn get_var(&self, jvm: &'gc JVMState<'gc>, current_class_pointer: &Arc<RuntimeClass<'gc>>, field_name: FieldName) -> NewJavaValueHandle<'gc> {
        let FieldNumberAndFieldType { number, cpdtype } = &current_class_pointer.unwrap_class_class().object_layout.field_numbers.get(&field_name).unwrap();
        self.raw_get_var(jvm, *number, *cpdtype)
    }


    pub fn get_var_hidden(&self, jvm: &'gc JVMState<'gc>, current_class_pointer: &Arc<RuntimeClass<'gc>>, field_name: HiddenJVMField) -> NewJavaValueHandle<'gc> {
        let FieldNumberAndFieldType { number, cpdtype } = &current_class_pointer.unwrap_class_class().object_layout.hidden_field_numbers.get(&field_name).unwrap();
        self.raw_get_var(jvm, *number, *cpdtype)
    }

    pub fn raw_get_var(&self, jvm: &'gc JVMState<'gc>, number: FieldNumber, cpdtype: CPDType) -> NewJavaValueHandle<'gc> {
        unsafe {
            let native_jv = self.ptr.cast::<NativeJavaValue<'gc>>().as_ptr().offset(number.0 as isize).read();
            native_to_new_java_value(native_jv, cpdtype, jvm)
        }
    }

    pub fn get_var_top_level(&self, jvm: &'gc JVMState<'gc>, field_name: FieldName) -> NewJavaValueHandle<'gc> {
        let current_class_pointer = self.runtime_class(jvm);
        self.get_var(jvm, &current_class_pointer, field_name)
    }

    pub fn duplicate_discouraged(&self) -> Self {
        self.jvm.gc.register_root_reentrant(self.jvm, self.ptr).unwrap_normal_object()
    }

    pub fn as_allocated_obj(&self) -> AllocatedObject<'gc, '_> {
        AllocatedObject::NormalObject(self)
    }
}

pub enum AllocatedHandle<'gc> {
    Array(AllocatedArrayObjectHandle<'gc>),
    NormalObject(AllocatedNormalObjectHandle<'gc>),
}

impl Eq for AllocatedHandle<'_> {}


impl<'gc> PartialEq for AllocatedHandle<'gc> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr() == other.ptr()
    }
}

impl<'gc> AllocatedHandle<'gc> {
    pub fn new_java_value(&self) -> NewJavaValue<'gc, '_> {
        NewJavaValue::AllocObject(AllocatedObject::Handle(self))
    }

    pub fn new_java_value_handle(self) -> NewJavaValueHandle<'gc> {
        NewJavaValueHandle::Object(self)
    }

    pub fn to_jv<'any>(&'any self) -> JavaValue<'gc> {
        todo!()
    }

    pub fn is_array(&self, jvm: &'gc JVMState<'gc>) -> bool {
        let rc = self.runtime_class(jvm);
        rc.cpdtype().is_array()
    }

    pub fn unwrap_array(&self) -> &'_ AllocatedArrayObjectHandle<'gc> {
        match self {
            AllocatedHandle::Array(arr) => arr,
            AllocatedHandle::NormalObject(_) => panic!()
        }
    }

    /*pub fn unwrap_array(&self, jvm: &'gc JVMState<'gc>) -> ArrayWrapper<'gc, '_> {
        assert!(self.is_array(jvm));
        ArrayWrapper {
            allocated_object: self.as_allocated_obj()
        }
    }*/

    pub fn unwrap_normal_object(self) -> AllocatedNormalObjectHandle<'gc> {
        match self {
            AllocatedHandle::Array(_) => panic!(),
            AllocatedHandle::NormalObject(normal_obj) => normal_obj
        }
    }
    pub fn unwrap_normal_object_ref(&self) -> &AllocatedNormalObjectHandle<'gc> {
        match self {
            AllocatedHandle::Array(_) => panic!(),
            AllocatedHandle::NormalObject(normal_obj) => normal_obj
        }
    }

    pub fn ptr(&self) -> NonNull<c_void> {
        match self {
            AllocatedHandle::Array(arr) => {
                arr.ptr
            }
            AllocatedHandle::NormalObject(obj) => {
                obj.ptr
            }
        }
    }

    pub fn runtime_class(&self, jvm: &'gc JVMState<'gc>) -> Arc<RuntimeClass<'gc>> {
        let allocated_obj_type = jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(self.ptr()).clone();
        assert_loaded_class(jvm, allocated_obj_type.as_cpdtype())
    }

    pub fn as_allocated_obj(&self) -> AllocatedObject<'gc, '_> {
        AllocatedObject::Handle(self)
    }

    pub fn duplicate_discouraged(&self) -> Self {
        match self {
            AllocatedHandle::Array(arr) => {
                let fix_this = arr.jvm.gc.register_root_reentrant(arr.jvm, arr.ptr);
                std::mem::forget(fix_this);
                AllocatedHandle::Array(AllocatedArrayObjectHandle { jvm: arr.jvm, ptr: arr.ptr })
            }
            AllocatedHandle::NormalObject(handle) => {
                AllocatedHandle::NormalObject(handle.duplicate_discouraged())
            }
        }
    }
}

impl Debug for AllocatedHandle<'_> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}


pub struct AllocatedObjectHandleByAddress<'gc>(pub AllocatedHandle<'gc>);

impl Hash for AllocatedObjectHandleByAddress<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.ptr().as_ptr() as usize);
    }
}

impl PartialEq for AllocatedObjectHandleByAddress<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr() == other.0.ptr()
    }
}

impl Eq for AllocatedObjectHandleByAddress<'_> {}

pub enum AllocatedObjectCOW<'gc, 'k> {
    Handle(AllocatedHandle<'gc>),
    Ref(&'k AllocatedHandle<'gc>),
}

impl<'gc, 'k> AllocatedObjectCOW<'gc, 'k> {
    pub fn as_allocated_object(&'k self) -> &'k AllocatedHandle<'gc> {
        match self {
            AllocatedObjectCOW::Handle(handle) => {
                handle
            }
            AllocatedObjectCOW::Ref(allocated_object) => {
                allocated_object.clone()
            }
        }
    }
}

#[derive(Clone)]
pub enum AllocatedObject<'gc, 'l> {
    Handle(&'l AllocatedHandle<'gc>),
    NormalObject(&'l AllocatedNormalObjectHandle<'gc>),
    ArrayObject(&'l AllocatedArrayObjectHandle<'gc>),
}

impl<'gc, 'l> AllocatedObject<'gc, 'l> {
    pub fn unwrap_normal_object(&self) -> &'l AllocatedNormalObjectHandle<'gc> {
        match self {
            AllocatedObject::Handle(handle) => {
                match handle {
                    AllocatedHandle::Array(_) => panic!(),
                    AllocatedHandle::NormalObject(normal_obj) => normal_obj
                }
            }
            AllocatedObject::NormalObject(normal_obj) => normal_obj,
            AllocatedObject::ArrayObject(_) => panic!()
        }
    }

    pub fn duplicate_discouraged(&self) -> AllocatedHandle<'gc> {
        match self {
            AllocatedObject::Handle(handle) => {
                match handle {
                    AllocatedHandle::NormalObject(normal_obj) => {
                        normal_obj.jvm.gc.register_root_reentrant(normal_obj.jvm, normal_obj.ptr)
                    }
                    AllocatedHandle::Array(array_object) => {
                        array_object.jvm.gc.register_root_reentrant(array_object.jvm, array_object.ptr)
                    }
                }
            }
            AllocatedObject::NormalObject(normal_obj) => {
                normal_obj.jvm.gc.register_root_reentrant(normal_obj.jvm, normal_obj.ptr)
            }
            AllocatedObject::ArrayObject(array_object) => {
                array_object.jvm.gc.register_root_reentrant(array_object.jvm, array_object.ptr)
            }
        }
    }

    //todo dup
    pub fn runtime_class(&self, jvm: &'gc JVMState<'gc>) -> Arc<RuntimeClass<'gc>> {
        let allocated_obj_type = jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(self.ptr()).clone();
        assert_loaded_class(jvm, allocated_obj_type.as_cpdtype())
    }
}

impl<'gc, 'l> AllocatedObject<'gc, 'l> {
    pub fn raw_ptr_usize(&self) -> usize {
        self.ptr().as_ptr() as usize
    }

    pub fn ptr(&self) -> NonNull<c_void> {
        match self {
            AllocatedObject::Handle(handle) => handle.ptr(),
            AllocatedObject::NormalObject(obj) => obj.ptr,
            AllocatedObject::ArrayObject(obj) => obj.ptr
        }
    }
}