use std::mem::size_of;

use gc_memory_layout_common::AllocatedObjectType;
use jvmti_jni_bindings::jlong;
use rust_jvm_common::compressed_classfile::CPDType;

use crate::{JVMState, NewJavaValue};
use crate::java_values::NativeJavaValue;
use crate::new_java_values::{AllocatedObject, NewJavaValueHandle};

pub struct ArrayWrapper<'gc_life, 'l> {
    pub(crate) allocated_object: AllocatedObject<'gc_life, 'l>,
}

impl<'gc_life, 'l> ArrayWrapper<'gc_life, 'l> {
    pub fn allocated_type(&self) -> AllocatedObjectType {
        let jvm = self.allocated_object.handle.jvm;
        let ptr = self.allocated_object.handle.ptr;
        jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(ptr).clone()
    }

    pub fn len(&self) -> usize {
        let allocated_type = self.allocated_type();
        match allocated_type {
            AllocatedObjectType::Class { .. } => {
                panic!()
            }
            AllocatedObjectType::ObjectArray { len, .. } => {
                len as usize
            }
            AllocatedObjectType::PrimitiveArray { len, .. } => {
                len as usize
            }
        }
    }

    pub fn elem_cpdtype(&self) -> CPDType {
        let allocated_type = self.allocated_type();
        match allocated_type {
            AllocatedObjectType::Class { .. } => {
                panic!()
            }
            AllocatedObjectType::ObjectArray { sub_type, .. } => {
                CPDType::Ref(sub_type)
            }
            AllocatedObjectType::PrimitiveArray { primitive_type, .. } => {
                primitive_type
            }
        }
    }

    pub fn get_i(&self, i: usize) -> NewJavaValueHandle<'gc_life> {
        assert!(i < self.len());
        let jvm = self.allocated_object.handle.jvm;
        let ptr = self.allocated_object.handle.ptr;
        let array_base = unsafe { ptr.as_ptr().offset(size_of::<jlong>() as isize) };
        let native_jv = unsafe { array_base.cast::<NativeJavaValue>().offset(i as isize).read() };
        let cpdtype = self.elem_cpdtype();
        native_jv.to_new_java_value(&cpdtype, jvm)
    }

    pub fn set_i(&self, i: usize, elem: NewJavaValue<'gc_life, '_>) {
        assert!(i < self.len());
        let jvm = self.allocated_object.handle.jvm;
        let ptr = self.allocated_object.handle.ptr;
        let array_base = unsafe { ptr.as_ptr().offset(size_of::<jlong>() as isize) };
        unsafe { array_base.cast::<NativeJavaValue>().offset(i as isize).write(elem.to_native()) };
    }

    pub fn array_iterator(&self) -> ArrayIterator<'gc_life, 'l, '_> {
        ArrayIterator {
            i: 0,
            array_wrapper: self,
        }
    }
}

pub struct ArrayIterator<'gc_life, 'l, 'k> {
    i: usize,
    array_wrapper: &'k ArrayWrapper<'gc_life, 'l>,
}

impl<'gc_life, 'l, 'k> Iterator for ArrayIterator<'gc_life, 'l, 'k> {
    type Item = NewJavaValueHandle<'gc_life>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.array_wrapper.len() {
            return None;
        }
        let res = self.array_wrapper.get_i(self.i);
        self.i += 1;
        Some(res)
    }
}

impl<'gc_life> NewJavaValueHandle<'gc_life> {
    pub fn unwrap_array(&self, jvm: &'gc_life JVMState<'gc_life>) -> ArrayWrapper<'gc_life, '_> {
        match self {
            NewJavaValueHandle::Object(obj) => {
                let allocated_object: AllocatedObject<'gc_life, '_> = obj.as_allocated_obj();
                ArrayWrapper { allocated_object }
            }
            _ => {
                panic!()
            }
        }
    }
}