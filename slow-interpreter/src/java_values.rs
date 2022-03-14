use std::cell::{RefCell, UnsafeCell};
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::{size_of, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr::{NonNull, null, null_mut, slice_from_raw_parts};
use std::slice;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::LocalKey;

use iced_x86::CC_be::na;
use iced_x86::CC_e::e;
use iced_x86::CC_le::le;
use iced_x86::OpCodeOperandKind::al;
use itertools::{Itertools, repeat_n};
use lazy_static::lazy_static;
use num_traits::one;
use parking_lot::OnceState::New;

use add_only_static_vec::AddOnlyVec;
use early_startup::Regions;
use gc_memory_layout_common::{AllocatedObjectType, MemoryRegions};
use jvmti_jni_bindings::{jbyte, jfieldID, jint, jlong, jmethodID, jobject};
use rust_jvm_common::compressed_classfile::{CompressedParsedRefType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};

use crate::check_initing_or_inited_class;
use crate::class_loading::{assert_inited_or_initing_class, check_resolved_class};
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::jvm_state::JVMState;
use crate::new_java_values::{AllocatedObject, AllocatedObjectHandle, NewJavaValue, NewJavaValueHandle, UnAllocatedObject, UnAllocatedObjectArray, UnAllocatedObjectObject};
use crate::runtime_class::{FieldNumber, RuntimeClass, RuntimeClassClass};
use crate::rust_jni::native_util::from_object;
use crate::threading::safepoints::Monitor2;

pub struct GC<'gc_life> {
    pub memory_region: Mutex<MemoryRegions>,
    //doesn't really need to be atomic usize
    vm_temp_owned_roots: RwLock<HashMap<NonNull<c_void>, AtomicUsize>>,
    pub(crate) all_allocated_object: RwLock<HashSet<NonNull<c_void>>>,
    //todo deprecated/ not in use
    phantom: PhantomData<&'gc_life ()>,
    pub objects_that_live_for_gc_life: AddOnlyVec<AllocatedObjectHandle<'gc_life>>,
}

impl<'gc_life> GC<'gc_life> {
    #[must_use]
    pub fn register_root_reentrant(&'gc_life self, jvm: &'gc_life JVMState<'gc_life>, ptr: NonNull<c_void>) -> AllocatedObjectHandle<'gc_life> {
        let mut guard = self.vm_temp_owned_roots.write().unwrap();
        let count = guard.entry(ptr).or_insert(AtomicUsize::new(0));
        count.fetch_add(1, Ordering::SeqCst);
        AllocatedObjectHandle {
            jvm,
            ptr,
        }
    }

    pub fn deregister_root_reentrant(&'gc_life self, ptr: NonNull<c_void>) {
        let mut guard = self.vm_temp_owned_roots.write().unwrap();
        let count = guard.get(&ptr).unwrap();
        count.fetch_sub(1, Ordering::SeqCst);
        if count.load(Ordering::SeqCst) == 0 {
            guard.remove(&ptr);
        }
    }

    pub fn handle_lives_for_gc_life(&'gc_life self, handle: AllocatedObjectHandle<'gc_life>) -> AllocatedObject<'gc_life, 'gc_life> {
        let index = self.objects_that_live_for_gc_life.len();
        self.objects_that_live_for_gc_life.push(handle);
        let handle_ref: &'gc_life AllocatedObjectHandle<'gc_life> = &self.objects_that_live_for_gc_life[index];
        AllocatedObject {
            handle: handle_ref
        }
    }

    pub fn allocate_object<'l>(&'gc_life self, jvm: &'gc_life JVMState<'gc_life>, object: UnAllocatedObject<'gc_life, 'l>) -> AllocatedObjectHandle<'gc_life> {
        // let ptr = NonNull::new(Box::into_raw(box object)).unwrap();
        let mut guard = self.memory_region.lock().unwrap();
        let allocated_object_type = match &object {
            UnAllocatedObject::Array(arr) => {
                assert!(arr.whole_array_runtime_class.cpdtype().is_array());
                runtime_class_to_allocated_object_type(&arr.whole_array_runtime_class, LoaderName::BootstrapLoader, Some(arr.elems.len()))
            }//todo loader name nonsense
            UnAllocatedObject::Object(obj) => runtime_class_to_allocated_object_type(&obj.object_rc, LoaderName::BootstrapLoader, None),
        };
        let mut memory_region = guard.find_or_new_region_for(allocated_object_type);
        let allocated = memory_region.get_allocation();
        let allocated_size = memory_region.region_elem_size;
        unsafe { libc::memset(allocated.as_ptr(), 0, allocated_size); }
        let handle = self.register_root_reentrant(jvm, allocated);//should register before putting in all objects so can't be gced
        self.all_allocated_object.write().unwrap().insert(allocated);
        match object {
            UnAllocatedObject::Object(UnAllocatedObjectObject { object_rc, fields }) => {
                for (i, field) in fields.iter() {
                    unsafe {
                        assert_eq!(size_of::<NativeJavaValue>(), size_of::<jlong>());
                        let field_ptr = allocated.as_ptr().cast::<NativeJavaValue>().offset(i.0 as isize);
                        field_ptr.write(field.to_native());
                    }
                }
            }
            UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems }) => {
                unsafe {
                    (allocated.as_ptr() as *mut i32).write(elems.len() as i32);
                    let array_base = allocated.as_ptr().offset(size_of::<jlong>() as isize);
                    assert_eq!(allocated_size, (elems.len() + 1) as usize * size_of::<jlong>());
                    for (i, elem) in elems.into_iter().enumerate() {
                        array_base.cast::<NativeJavaValue>().offset(i as isize).write(elem.to_native())
                    }
                }
            }
        }

        handle
        // GcManagedObject {
        //     obj: match object {
        //         Object::Array(ArrayObject { whole_array_runtime_class, loader, len, elems_base, phantom_data, elem_type }) => {
        //             unsafe {
        //                 (allocated.as_ptr() as *mut i32).write(len);
        //                 let elems = slice::from_raw_parts_mut(elems_base, len as usize);
        //                 let new_elems_base = allocated.as_ptr().offset(size_of::<jlong>() as isize).cast::<NativeJavaValue<'gc_life>>();
        //                 for (i, current_elem) in elems.iter().enumerate() {
        //                     let current_new_elem_ptr = new_elems_base.offset((i /* * size_of::<jlong>()*/) as isize);//not a void pointer so need to mul here
        //                     current_new_elem_ptr.write(*current_elem);
        //                 }
        //                 dbg!(&slice::from_raw_parts(new_elems_base, len as usize));
        //                 assert_eq!(allocated_size, (len + 1) as usize * size_of::<jlong>());
        //                 Arc::new(Object::Array(ArrayObject { whole_array_runtime_class, loader, len, elems_base: new_elems_base, phantom_data: Default::default(), elem_type }))
        //             }
        //         }
        //         Object::Object(NormalObject { objinfo: ObjectFieldsAndClass { fields, class_pointer }, obj_ptr }) => {
        //
        //         }
        //     },
        //     raw_ptr: allocated,
        //     gc: self,
        //     jvm,
        // }
    }

    fn gc_recurse(obj: NonNull<c_void>, visited: &mut HashSet<NonNull<c_void>>) {
        if visited.contains(&obj) {
            return;
        }
        unsafe {
            todo!()
            /*if obj.as_ptr() == transmute(0xDEADDEADDEADDEADusize) {
                //todo need better handling of top, but is fine for now
                return;
            }*/
        }
        visited.insert(obj);
        match todo!()/*unsafe { obj.as_ref() }*/ {
            Object::Array(arr) => {
                if let CPDType::Ref(_) = arr.elem_type {
                    todo!()
                    /*for elem in unsafe { todo!()/*arr.elems.get().as_ref()*/ }.unwrap() {
                        unsafe {
                            Self::gc_recurse(match NonNull::new(todo!()) {
                                None => continue,
                                Some(ptr) => ptr
                            }, visited);
                        }
                    }*/
                }
            }
            Object::Object(obj) => {
                Self::gc_recurse_super(todo!()/*obj*/, obj.objinfo.class_pointer.unwrap_class_class(), visited)
            }
        }
    }

    fn gc_recurse_super<'l>(obj: &NormalObject<'gc_life, 'l>, class_class: &RuntimeClassClass, visited: &mut HashSet<NonNull<c_void>>) {
        for (_name, (index, rtype)) in class_class.field_numbers.iter() {
            if let CPDType::Ref(_) = rtype {
                Self::gc_recurse(
                    match NonNull::new(todo!() /*unsafe { obj.objinfo.fields[*index].get().as_ref().unwrap().object }*/) {
                        None => continue,
                        Some(ptr) => ptr,
                    },
                    visited,
                )
            }
        }
        if let Some(parent) = &class_class.parent {
            Self::gc_recurse_super(obj, parent.unwrap_class_class(), visited)
        }
    }

    fn gc_with_roots(&self, roots: HashSet<NonNull<c_void>>) {
        let mut visited = HashSet::new();
        for root in roots.iter() {
            Self::gc_recurse(*root, &mut visited);
        }
        let all_objs = self.all_allocated_object.read().unwrap();
        let to_frees = all_objs.difference(&visited).cloned().collect_vec();
        for to_free in to_frees.iter() {
            // eprintln!("Freeing:{:?}", to_free.as_ptr());
            drop(unsafe { Box::from_raw(to_free.as_ptr()) })
        }
        drop(all_objs);
        let mut guard = self.all_allocated_object.write().unwrap();
        for to_free in to_frees {
            guard.remove(&to_free);
        }
    }

    pub fn gc_jvm(&self, jvm: &'gc_life JVMState<'gc_life>) {
        todo!()
        /*if !jvm.vm_live() {
            return;
        }
        let interpreter_states = jvm
            .thread_state
            .all_java_threads
            .read()
            .unwrap()
            .values()
            .map(|jt| {
                unsafe {
                    jt.gc_suspend();
                }
                todo!()
                /*let guard = InterpreterStateGuard {
                    int_state: Some(jt.interpreter_state.write().unwrap()),
                    thread: jt.clone(),
                    registered: false,
                };
                (guard.cloned_stack_snapshot(jvm), guard.throw())*/
            })
            .collect_vec();
        let mut roots = HashSet::new();
        unsafe {
            // dbg!(interpreter_states.len());
            for (stack, throw) in interpreter_states {
                // dbg!(stack.len());
                /*if let Some(throw) = throw {
                    roots.insert(throw.raw_ptr);
                }*/
                todo!();
                for stack_entry in stack {
                    // dbg!(stack_entry.operand_stack.len());
                    // dbg!(stack_entry.local_vars().len());
                    for local_var in stack_entry.local_vars().iter().cloned().chain(stack_entry.operand_stack().iter().cloned()).chain(stack_entry.native_local_refs.iter().flat_map(|hashet| hashet.iter().map(|raw| JavaValue::Object(from_object(jvm, *raw))))) {
                        if let JavaValue::Object(Some(gc_managed)) = local_var {
                            // dbg!(gc_managed.raw_ptr);
                            roots.insert(gc_managed.raw_ptr);
                        }
                    }
                }
            }
        }
        for (root, _) in self.vm_temp_owned_roots.read().unwrap().iter() {
            roots.insert(todo!() /*root.clone()*/);
        }
        let classes_guard = jvm.classes.read().unwrap();
        for root in jvm
            .string_internment
            .read()
            .unwrap()
            .strings
            .values()
            .cloned()
            .chain(jvm.thread_state.system_thread_group.read().unwrap().as_ref().map(|thread_group| thread_group.clone().object()).iter().cloned())
            .chain(jvm.thread_state.all_java_threads.read().unwrap().values().map(|jt| jt.thread_object().object()))
            .chain(classes_guard.classes_gc_roots())
        {
            roots.insert(root.raw_ptr);
        }
        self.gc_with_roots(todo!() /*roots*/)*/
    }

    pub fn new(regions: early_startup::Regions) -> Self {
        Self {
            memory_region: Mutex::new(MemoryRegions::new(regions)),
            vm_temp_owned_roots: RwLock::new(Default::default()),
            all_allocated_object: Default::default(),
            phantom: PhantomData::default(),
            objects_that_live_for_gc_life: AddOnlyVec::new(),
        }
    }
}

#[derive(Clone)]
pub enum ByAddressAllocatedObject<'gc_life, 'l> {
    Owned(AllocatedObject<'gc_life, 'l>),
    LookupOnly(usize),
}

impl<'gc_life, 'l> ByAddressAllocatedObject<'gc_life, 'l> {
    pub fn raw_ptr_usize(&self) -> usize {
        match self {
            ByAddressAllocatedObject::Owned(owned) => {
                owned.raw_ptr_usize()
            }
            ByAddressAllocatedObject::LookupOnly(lookup) => {
                *lookup
            }
        }
    }

    pub fn owned_inner(self) -> AllocatedObject<'gc_life, 'l> {
        match self {
            ByAddressAllocatedObject::Owned(owned) => owned,
            ByAddressAllocatedObject::LookupOnly(_) => panic!()
        }
    }

    pub fn owned_inner_ref(&self) -> AllocatedObject<'gc_life, 'l> {
        match self {
            ByAddressAllocatedObject::Owned(owned) => owned.clone(),
            ByAddressAllocatedObject::LookupOnly(_) => panic!()
        }
    }
}

impl Hash for ByAddressAllocatedObject<'_, '_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.raw_ptr_usize())
    }
}

impl Eq for ByAddressAllocatedObject<'_, '_> {}

impl PartialEq for ByAddressAllocatedObject<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_ptr_usize() == other.raw_ptr_usize()
    }
}

pub struct GcManagedObject<'gc_life> {
    obj: Arc<Object<'gc_life, 'gc_life>>,
    //todo this double gc life thing is kinda unsafe
    raw_ptr: NonNull<c_void>,
    //allocated from a box
    gc: &'gc_life GC<'gc_life>,
    jvm: &'gc_life JVMState<'gc_life>,
}

impl<'gc_life> GcManagedObject<'gc_life> {
    pub fn from_native(raw_ptr: NonNull<c_void>, jvm: &'gc_life JVMState<'gc_life>) -> Self {
        let handle = jvm.gc.register_root_reentrant(jvm, raw_ptr);
        dbg!(&handle);
        todo!();
        let guard = jvm.gc.memory_region.lock().unwrap();
        let allocated_type = guard.find_object_allocated_type(raw_ptr);
        let obj = match allocated_type {
            AllocatedObjectType::Class { size, name, loader } => {
                let classes = jvm.classes.read().unwrap();
                let runtime_class = classes.loaded_classes_by_type(loader, &(*name).into());
                let runtime_class_class = runtime_class.unwrap_class_class();
                let num_fields = runtime_class_class.recursive_num_fields;
                unsafe {
                    Arc::new(Object::Object(NormalObject {
                        objinfo: ObjectFieldsAndClass {
                            fields: RwLock::new(slice::from_raw_parts_mut(raw_ptr.as_ptr() as *mut NativeJavaValue<'gc_life>, num_fields)),
                            class_pointer: runtime_class.clone(),
                        },
                        obj_ptr: Some(raw_ptr.cast()),
                    }))
                }
            }
            AllocatedObjectType::ObjectArray { sub_type, sub_type_loader, len } => {
                let classes = jvm.classes.read().unwrap();
                let runtime_class = classes.loaded_classes_by_type(sub_type_loader, &CPDType::array(CPDType::Ref(sub_type.clone())));
                unsafe {
                    Arc::new(Object::Array(ArrayObject {
                        whole_array_runtime_class: runtime_class.clone(),
                        loader: *sub_type_loader,
                        len: *len,
                        elems_base: raw_ptr.as_ptr().offset(size_of::<jlong>() as isize) as *mut NativeJavaValue<'gc_life>,
                        phantom_data: Default::default(),
                        elem_type: CPDType::Ref(sub_type.clone()),
                    }))
                }
            }
            AllocatedObjectType::PrimitiveArray { primitive_type, len } => {
                let classes = jvm.classes.read().unwrap();
                //todo loader nonsense
                let runtime_class = classes.loaded_classes_by_type(&LoaderName::BootstrapLoader, &CPDType::array(*primitive_type));
                unsafe {
                    Arc::new(Object::Array(ArrayObject {
                        whole_array_runtime_class: runtime_class.clone(),
                        loader: LoaderName::BootstrapLoader,//todo loader nonsense
                        len: *len,
                        phantom_data: Default::default(),
                        elem_type: primitive_type.clone(),
                        elems_base: raw_ptr.as_ptr().offset(size_of::<jlong>() as isize) as *mut NativeJavaValue<'gc_life>,
                    }))
                }
            }
        };
        Self { obj, raw_ptr, gc: jvm.gc, jvm }
    }

    pub fn from_native_assert_already_registered(raw_ptr: NonNull<c_void>, gc: &'gc_life GC<'gc_life>) -> Self {
        todo!();
        assert!(gc.vm_temp_owned_roots.read().unwrap().contains_key(&raw_ptr));
        Self { obj: todo!(), raw_ptr, gc, jvm: todo!() }
    }

    pub fn self_check(&self) {
        assert!(self.gc.vm_temp_owned_roots.read().unwrap().contains_key(&(self.raw_ptr)));
        if !self.gc.all_allocated_object.read().unwrap().contains(&(self.raw_ptr)) {
            panic!()
        }
    }

    pub fn strong_count(&self) -> usize {
        self.gc.vm_temp_owned_roots.read().unwrap().get(&(self.raw_ptr)).unwrap().load(Ordering::SeqCst)
    }

    pub fn to_allocated_object(&self) -> AllocatedObject<'gc_life, 'static> {
        todo!()
    }

    pub fn to_allocated_object_not_static(&self) -> AllocatedObject<'gc_life, 'gc_life> {
        todo!()
    }
}

impl<'gc_life> Deref for GcManagedObject<'gc_life> {
    type Target = Object<'gc_life, 'gc_life>;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}


impl<'gc_life> DerefMut for GcManagedObject<'gc_life> {
    #[allow(mutable_transmutes)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { transmute(self.obj.deref()) }//todo mega unsafe this whole model needs a rethink
    }
}


impl<'gc_life> Clone for GcManagedObject<'gc_life> {
    fn clone(&self) -> Self {
        //this doesn't leak b/c if we ever try to create a cycle we put into a field and deregister as a root.
        todo!();
        // self.gc.register_root_reentrant(self.raw_ptr);
        Self { obj: self.obj.clone(), raw_ptr: self.raw_ptr, gc: self.gc, jvm: self.jvm }
    }
}

impl Drop for GcManagedObject<'_> {
    fn drop(&mut self) {
        self.gc.deregister_root_reentrant(self.raw_ptr)
    }
}

impl<'gc_life> GcManagedObject<'gc_life> {
    pub fn lookup_field(&self, jvm: &'gc_life JVMState<'gc_life>, field_name: FieldName) -> JavaValue<'gc_life> {
        self.deref().lookup_field(jvm, field_name)
    }

    pub fn unwrap_normal_object(&self) -> &NormalObject<'gc_life, 'gc_life> {
        self.deref().unwrap_normal_object()
    }

    pub fn ptr_eq(one: &GcManagedObject<'gc_life>, two: &GcManagedObject<'gc_life>) -> bool {
        one.raw_ptr == two.raw_ptr
    }

    pub fn raw_ptr_usize(&self) -> usize {
        self.raw_ptr.as_ptr() as usize
    }
}

// #[derive(Copy)]
pub enum JavaValue<'gc_life> {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),

    Float(f32),
    Double(f64),
    Object(Option<GcManagedObject<'gc_life>>),

    Top, //should never be interacted with by the bytecode
}

impl<'gc_life> JavaValue<'gc_life> {
    pub(crate) fn self_check(&self) {
        if let JavaValue::Object(Some(obj)) = self {
            obj.self_check()
        }
    }

    pub fn to_native(&self) -> NativeJavaValue<'gc_life> {
        /*match self.clone() {
            JavaValue::Long(val_) => NativeJavaValue { long: val_ },
            JavaValue::Int(val_) => NativeJavaValue { int: val_ },
            JavaValue::Short(val_) => NativeJavaValue { int: val_ as i32 },
            JavaValue::Byte(val_) => NativeJavaValue { int: val_ as i32 },
            JavaValue::Boolean(val_) => NativeJavaValue { int: val_ as i32 },
            JavaValue::Char(val_) => NativeJavaValue { int: val_ as i32 },
            JavaValue::Float(val_) => NativeJavaValue { float: val_ },
            JavaValue::Double(val_) => NativeJavaValue { double: val_ },
            JavaValue::Object(val_) => {
                NativeJavaValue {
                    object: match val_ {
                        None => null_mut(),
                        Some(gc_managed) => {
                            gc_managed.raw_ptr.as_ptr()
                        }
                    },
                }
            }
            JavaValue::Top => unsafe { transmute(0xDEADDEADDEADDEADusize) },
        }*/
        todo!()
    }

    pub fn to_new<'anything>(&self) -> NewJavaValue<'gc_life, 'anything> {
        todo!()
    }
}
// pub trait CycleDetectingDebug {
//     fn cycle_fmt<'gc_life>(&self, prev: &Vec<&GcManagedObject<'gc_life>>, f: &mut Formatter<'_>) -> Result<(), Error>;
// }

// impl<'gc_life> CycleDetectingDebug for JavaValue<'gc_life> {
//     fn cycle_fmt(&self, prev: &Vec<&GcManagedObject<'gc_life>>, f: &mut Formatter<'_>) -> Result<(), Error> {
//         match self {
//             JavaValue::Long(l) => { write!(f, "{}", l) }
//             JavaValue::Int(l) => { write!(f, "{}", l) }
//             JavaValue::Short(l) => { write!(f, "{}", l) }
//             JavaValue::Byte(l) => { write!(f, "{}", l) }
//             JavaValue::Boolean(l) => { write!(f, "{}", l) }
//             JavaValue::Char(l) => { write!(f, "{}", l) }
//             JavaValue::Float(l) => { write!(f, "{}", l) }
//             JavaValue::Double(l) => { write!(f, "{}", l) }
//             JavaValue::Object(o) => {
//                 match o {
//                     None => {
//                         write!(f, "null", )
//                     }
//                     Some(s) => {
//                         if prev.iter().any(|above| Arc::ptr_eq(above, s)) {
//                             write!(f, "<cycle>")
//                         } else {
//                             let mut new = prev.clone();
//                             new.push(s);
//                             s.cycle_fmt(&new, f)
//                         }
//                     }
//                 }
//             }
//             JavaValue::Top => { write!(f, "top") }
//         }
//     }
// }
//
// impl<'gc_life> CycleDetectingDebug for Object<'gc_life> {
//     fn cycle_fmt(&self, prev: &Vec<&GcManagedObject<'gc_life>>, f: &mut Formatter<'_>) -> Result<(), Error> {
//         write!(f, "\n")?;
//         for _ in 0..prev.len() {
//             write!(f, " ")?;
//         }
//         match &self {
//             Object::Array(a) => {
//                 write!(f, "[")?;
//                 unsafe {
//                     a.elems.get().as_ref().unwrap()
//                 }.iter().for_each(|x| {
//                     x.cycle_fmt(prev, f).unwrap();
//                     write!(f, ",").unwrap();
//                 });
//                 write!(f, "]")
//             }
//             Object::Object(o) => {
//                 o.cycle_fmt(prev, f)
//             }
//         }
//     }
// }
//
// impl<'gc_life> CycleDetectingDebug for NormalObject<'gc_life> {
//     fn cycle_fmt(&self, prev: &Vec<&GcManagedObject<'gc_life>>, f: &mut Formatter<'_>) -> Result<(), Error> {
// //         let o = self;
// //         if o.class_pointer.view().name() == ClassName::class().into() {
// //             write!(f, "need a jvm pointer here to give more info on class object")?;
// //         } else if o.class_pointer.view().name() == ClassName::string().into() {
// //             let fields_borrow = o.fields_mut();
// //             let value_field = fields_borrow.get("value").unwrap();
// //             match &value_field.unwrap_object() {
// //                 None => {
// //                     write!(f, "(String Object: {:?})", "weird af string obj.")?;
// //                 }
// //                 Some(_) => {
// //                     write!(f, "(String Object: {:?})", value_field.unwrap_array().unwrap_char_array())?;
// //                 }
// //             }
// //         } else {
// //             write!(f, "{:?}", &o.class_pointer.view().name())?;
// //             write!(f, "-")?;
// // //        write!(f, "{:?}", self.class_pointer.static_vars)?;
// //             write!(f, "-")?;
// //             o.fields_mut().iter().for_each(|(n, v)| {
// //                 write!(f, "({},", n).unwrap();
// //                 v.cycle_fmt(prev, f).unwrap();
// //                 write!(f, ")").unwrap();
// //             });
// //             write!(f, "-")?;
// //         }
// //         Result::Ok(())
//         writeln!(f, "object")
//     }
// }

impl<'gc_life> Debug for JavaValue<'gc_life> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            JavaValue::Long(elem) => {
                write!(f, "Long:{}", elem)
            }
            JavaValue::Int(elem) => {
                write!(f, "Int:{}", elem)
            }
            JavaValue::Short(elem) => {
                write!(f, "Short:{}", elem)
            }
            JavaValue::Byte(elem) => {
                write!(f, "Byte:{}", elem)
            }
            JavaValue::Boolean(elem) => {
                write!(f, "Boolean:{}", elem)
            }
            JavaValue::Char(elem) => {
                write!(f, "Char:{}", elem)
            }
            JavaValue::Float(elem) => {
                write!(f, "Float:{}", elem)
            }
            JavaValue::Double(elem) => {
                write!(f, "Double:{}", elem)
            }
            JavaValue::Object(obj) => {
                write!(f, "obj:{:?}", obj.as_ref().map(|obj| obj.raw_ptr.as_ptr()).unwrap_or(null_mut()))
            }
            JavaValue::Top => write!(f, "top"),
        }
    }
}

impl<'gc_life> JavaValue<'gc_life> {
    pub fn null() -> Self {
        Self::Object(None)
    }

    pub fn unwrap_int(&self) -> i32 {
        self.try_unwrap_int().unwrap()
    }

    pub fn try_unwrap_int(&self) -> Option<i32> {
        match self {
            JavaValue::Int(i) => *i,
            JavaValue::Byte(i) => *i as i32,
            JavaValue::Boolean(i) => *i as i32,
            JavaValue::Char(c) => *c as i32,
            JavaValue::Short(i) => *i as i32,
            _ => {
                return None;
            }
        }
            .into()
    }

    pub fn unwrap_float(&self) -> f32 {
        self.try_unwrap_float().unwrap()
    }
    pub fn try_unwrap_float(&self) -> Option<f32> {
        match self {
            JavaValue::Float(f) => (*f).into(),
            _ => None,
        }
    }
    pub fn unwrap_double(&self) -> f64 {
        self.try_unwrap_double().unwrap()
    }

    pub fn unwrap_long(&self) -> i64 {
        self.try_unwrap_long().unwrap()
    }

    pub fn try_unwrap_double(&self) -> Option<f64> {
        match self {
            JavaValue::Double(f) => (*f).into(),
            _ => None,
        }
    }

    pub fn try_unwrap_long(&self) -> Option<i64> {
        match self {
            JavaValue::Long(l) => (*l).into(),
            _ => {
                dbg!(self);
                None
            }
        }
    }

    pub fn unwrap_byte(&self) -> i8 {
        let res = self.unwrap_int();
        assert!(res <= jbyte::MAX as i32);
        assert!(res >= jbyte::MIN as i32);
        res as i8
    }

    pub fn unwrap_boolean(&self) -> u8 {
        let res = self.unwrap_int();
        assert!(res <= u8::MAX as i32);
        assert!(res >= u8::MIN as i32);
        res as u8
    }

    pub fn unwrap_short(&self) -> i16 {
        let res = self.unwrap_int();
        assert!(res <= i16::MAX as i32);
        assert!(res >= i16::MIN as i32);
        res as i16
    }

    pub fn unwrap_object(&self) -> Option<GcManagedObject<'gc_life>> {
        self.try_unwrap_object().unwrap()
    }

    pub fn unwrap_object_nonnull(&self) -> GcManagedObject<'gc_life> {
        match match self.try_unwrap_object() {
            Some(x) => x,
            None => unimplemented!(),
        } {
            Some(x) => x,
            None => unimplemented!(),
        }
    }

    pub fn unwrap_array<'l>(&'l self) -> &'l ArrayObject<'gc_life, 'gc_life> {
        match self {
            JavaValue::Object(o) => o.as_ref().unwrap().unwrap_array(),
            _ => panic!(),
        }
    }

    pub fn unwrap_array_mut<'l>(&'l mut self) -> &'l mut ArrayObject<'gc_life, 'gc_life> {
        todo!()
    }

    pub fn try_unwrap_object(&self) -> Option<Option<GcManagedObject<'gc_life>>> {
        match self {
            JavaValue::Object(o) => Some(o.clone()),
            _ => {
                // dbg!(other);
                None
            }
        }
    }

    pub fn deep_clone(&self, jvm: &'gc_life JVMState<'gc_life>) -> Self {
        todo!()
        /*match &self {
            JavaValue::Object(o) => JavaValue::Object(match o {
                None => None,
                Some(o) => jvm.allocate_object(todo!()).into(),
            }),
            JavaValue::Top => panic!(),
            jv => (*jv).clone(),
        }*/
    }
    pub fn empty_byte_array<'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<AllocatedObjectHandle<'gc_life>, WasException> {
        let byte_array = check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::ByteType))?;
        Ok(jvm.allocate_object(UnAllocatedObject::new_array(byte_array, vec![])))
    }

    fn new_object_impl(runtime_class: &Arc<RuntimeClass<'gc_life>>) -> ObjectFieldsAndClass<'gc_life, 'gc_life> {
        let fields = repeat_n(JavaValue::Top, runtime_class.unwrap_class_class().num_vars()).map(|jv| UnsafeCell::new(jv.to_native())).collect_vec();
        ObjectFieldsAndClass { fields: todo!(), class_pointer: runtime_class.clone() }
    }

    pub fn new_object(jvm: &'gc_life JVMState<'gc_life>, runtime_class: Arc<RuntimeClass<'gc_life>>) -> AllocatedObjectHandle<'gc_life> {
        assert!(!runtime_class.view().is_abstract());

        let class_class = runtime_class.unwrap_class_class();

        let mut fields = class_class.field_numbers_reverse.iter().map(|(i, (_, cpd_type))| (*i, default_value_njv(cpd_type))).collect::<HashMap<_, _>>();

        jvm.allocate_object(UnAllocatedObject::Object(UnAllocatedObjectObject {
            object_rc: runtime_class,
            fields,
        }))
            .into()
    }

    pub fn new_vec<'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, len: usize, val: NewJavaValue<'gc_life,'_>, elem_type: CPDType) -> Result<AllocatedObjectHandle<'gc_life>, WasException> {
        let mut buf: Vec<NewJavaValue<'gc_life,'_>> = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(val.clone());
        }
        Ok(jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class: check_initing_or_inited_class(jvm,int_state,CPDType::array(elem_type)).unwrap(), elems: buf })/*Object::Array(ArrayObject::new_array(jvm, int_state, buf, elem_type, jvm.thread_state.new_monitor("array object monitor".to_string()))?)*/))
    }

    pub fn new_vec_from_vec(jvm: &'gc_life JVMState<'gc_life>, vals: Vec<NewJavaValue<'gc_life,'_>>, elem_type: CPDType) -> AllocatedObjectHandle<'gc_life> {
        let whole_array_runtime_class = assert_inited_or_initing_class(jvm, CPDType::array(elem_type));
        jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray{ whole_array_runtime_class, elems: vals })/*Object::Array(ArrayObject {
            whole_array_runtime_class: todo!(),
            loader: todo!(),
            len: todo!(),
            elems_base: todo!(),
            phantom_data: Default::default(),
            elem_type,
        })*/)
    }

    pub fn unwrap_normal_object(&self) -> &NormalObject<'gc_life, 'gc_life> {
        //todo these are longer than ideal
        self.try_unwrap_normal_object().unwrap()
    }

    pub fn try_unwrap_normal_object(&self) -> Option<&NormalObject<'gc_life, 'gc_life>> {
        //todo these are longer than ideal
        match self {
            JavaValue::Object(ref_) => match match ref_.as_ref() {
                None => return None,
                Some(obj) => obj.deref(),
            } {
                Object::Array(_) => None,
                Object::Object(o) => o.into(),
            },
            _ => None,
        }
    }

    pub fn unwrap_char(&self) -> u16 {
        self.unwrap_int() as u16
        // match self {
        //     JavaValue::Char(c) => {
        //         *c
        //     }
        //     _ => {
        //         dbg!(self);
        //         panic!()
        //     }
        // }
    }

    pub fn to_type(&self) -> RuntimeType {
        match self {
            JavaValue::Long(_) => RuntimeType::LongType,
            JavaValue::Int(_) => RuntimeType::IntType,
            JavaValue::Short(_) => RuntimeType::IntType,
            JavaValue::Byte(_) => RuntimeType::IntType,
            JavaValue::Boolean(_) => RuntimeType::IntType,
            JavaValue::Char(_) => RuntimeType::IntType,
            JavaValue::Float(_) => RuntimeType::FloatType,
            JavaValue::Double(_) => RuntimeType::DoubleType,
            JavaValue::Object(obj) => RuntimeType::Ref(match obj {
                None => RuntimeRefType::NullType,
                Some(not_null) => match not_null.deref() {
                    Object::Array(array) => RuntimeRefType::Array(array.elem_type.clone().into()),
                    Object::Object(obj) => RuntimeRefType::Class(obj.objinfo.class_pointer.cpdtype().unwrap_class_type()),
                },
            }),
            JavaValue::Top => RuntimeType::TopType,
        }
    }

    pub fn is_size_2(&self) -> bool {
        match self {
            JavaValue::Long(_) => true,
            JavaValue::Double(_) => true,
            _ => false,
        }
    }

    pub fn is_size_1(&self) -> bool {
        !self.is_size_2()
    }
}

impl<'gc_life> Clone for JavaValue<'gc_life> {
    fn clone(&self) -> Self {
        match self {
            JavaValue::Long(l) => JavaValue::Long(*l),
            JavaValue::Int(i) => JavaValue::Int(*i),
            JavaValue::Short(s) => JavaValue::Short(*s),
            JavaValue::Byte(b) => JavaValue::Byte(*b),
            JavaValue::Boolean(b) => JavaValue::Boolean(*b),
            JavaValue::Char(c) => JavaValue::Char(*c),
            JavaValue::Float(f) => JavaValue::Float(*f),
            JavaValue::Double(d) => JavaValue::Double(*d),
            JavaValue::Object(o) => JavaValue::Object(o.clone()),
            JavaValue::Top => JavaValue::Top,
        }
    }
}

impl<'gc_life> PartialEq for JavaValue<'gc_life> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            JavaValue::Long(x) => match other {
                JavaValue::Long(x1) => x == x1,
                _ => false,
            },
            JavaValue::Int(x) => match other {
                JavaValue::Int(x1) => x == x1,
                _ => false,
            },
            JavaValue::Short(x) => match other {
                JavaValue::Short(x1) => x == x1,
                _ => false,
            },
            JavaValue::Byte(x) => match other {
                JavaValue::Byte(x1) => x == x1,
                _ => false,
            },
            JavaValue::Boolean(x) => match other {
                JavaValue::Boolean(x1) => x == x1,
                _ => false,
            },
            JavaValue::Char(x) => match other {
                JavaValue::Char(x1) => x == x1,
                _ => false,
            },
            JavaValue::Float(x) => match other {
                JavaValue::Float(x1) => x == x1,
                _ => false,
            },
            JavaValue::Double(x) => match other {
                JavaValue::Double(x1) => x == x1,
                _ => false,
            },
            /*JavaValue::Array(x) => {
                match other {
                    JavaValue::Array(x1) => x == x1,
                    _ => false
                }
            }*/
            JavaValue::Object(x) => {
                match other {
                    JavaValue::Object(x1) => {
                        todo!()
                        /*                        match x {
                                                    None => x1.is_none(),
                                                    Some(o) => match x1 {
                                                        None => false,
                                                        Some(o1) => Arc::ptr_eq(o, o1),
                                                    },
                                                }
                        */
                    }
                    _ => false,
                }
            }
            JavaValue::Top => {
                matches!(other, JavaValue::Top)
            }
        }
    }
}

pub enum Object<'gc_life, 'underlying_data> {
    Array(ArrayObject<'gc_life, 'underlying_data>),
    Object(NormalObject<'gc_life, 'underlying_data>),
}

//todo should really fix this
unsafe impl<'gc_life> Send for Object<'gc_life, '_> {}

unsafe impl<'gc_life> Sync for Object<'gc_life, '_> {}

impl<'gc_life, 'l> Object<'gc_life, 'l> {
    pub fn lookup_field(&self, jvm: &'gc_life JVMState<'gc_life>, s: FieldName) -> JavaValue<'gc_life> {
        let class_pointer = self.unwrap_normal_object().objinfo.class_pointer.clone();
        let (field_number, rtype) = match class_pointer.unwrap_class_class().field_numbers.get(&s) {
            None => {
                dbg!(class_pointer.view().name().unwrap_object_name().0.to_str(&jvm.string_pool));
                dbg!(s.0.to_str(&jvm.string_pool));
                panic!()
            }
            Some(res) => res,
        };
        let normal_object = self.unwrap_normal_object();
        let guard = normal_object.objinfo.fields.read().unwrap();
        guard[field_number.0 as usize].to_java_value(rtype, jvm)
    }

    pub fn unwrap_normal_object(&self) -> &NormalObject<'gc_life, 'l> {
        match self {
            Object::Array(_) => panic!(),
            Object::Object(o) => o,
        }
    }

    pub fn unwrap_normal_object_mut(&mut self) -> &mut NormalObject<'gc_life, 'l> {
        match self {
            Object::Array(_) => panic!(),
            Object::Object(o) => o,
        }
    }

    pub fn try_unwrap_normal_object(&self) -> Option<&NormalObject<'gc_life, 'l>> {
        match self {
            Object::Array(_) => None,
            Object::Object(o) => Some(o),
        }
    }

    pub fn unwrap_array(&self) -> &ArrayObject<'gc_life, 'l> {
        match self {
            Object::Array(a) => a,
            Object::Object(obj) => {
                dbg!(obj.objinfo.class_pointer.view().name().unwrap_name());
                dbg!(obj.objinfo.class_pointer.unwrap_class_class().class_view.name());
                panic!()
            }
        }
    }

    pub fn unwrap_array_mut(&mut self) -> &mut ArrayObject<'gc_life, 'l> {
        match self {
            Object::Array(a) => a,
            Object::Object(obj) => {
                dbg!(obj.objinfo.class_pointer.view().name().unwrap_name());
                dbg!(obj.objinfo.class_pointer.unwrap_class_class().class_view.name());
                panic!()
            }
        }
    }

    pub fn deep_clone(&self, jvm: &'gc_life JVMState<'gc_life>) -> Self {
        match &self {
            Object::Array(a) => {
                // let sub_array = a.array_iterator(jvm).map(|x| x.deep_clone(jvm).to_native()).collect();//todo
                todo!();
                Object::Array(ArrayObject {
                    whole_array_runtime_class: a.whole_array_runtime_class.clone(),
                    loader: a.loader,
                    len: todo!(),
                    elems_base: todo!(),
                    phantom_data: Default::default(),
                    elem_type: a.elem_type.clone(),
                })
            }
            Object::Object(o) => {
                todo!()
                // let new_fields = UnsafeCell::new(o.fields_mut().iter().map(|(s, jv)| { (s.clone(), jv.deep_clone(jvm)) }).collect());
                // Object::Object(NormalObject {
                //     monitor: jvm.thread_state.new_monitor("".to_string()),
                //     fields: new_fields,
                //     class_pointer: o.class_pointer.clone(),
                // })
            }
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Object::Array(_) => true,
            Object::Object(_) => false,
        }
    }

    pub fn object_array(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, object_array: Vec<JavaValue<'gc_life>>, class_type: CPDType) -> Result<Object<'gc_life, 'gc_life>, WasException> {
        Ok(Object::Array(ArrayObject::new_array(jvm, int_state, object_array, class_type, jvm.thread_state.new_monitor("".to_string()))?))
    }

    pub fn monitor(&self) -> &Monitor2 {
        match self {
            Object::Array(a) => todo!(),  /*&a.monitor*/
            Object::Object(o) => todo!(), /*&o.monitor*/
        }
    }

    pub fn monitor_unlock<'k>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'k>) {
        self.monitor().unlock(jvm, int_state).unwrap();
    }

    pub fn monitor_lock<'k>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'k>) {
        let monitor_to_lock = self.monitor();
        monitor_to_lock.lock(jvm, int_state).unwrap();
    }
}

pub struct ArrayObject<'gc_life, 'l> {
    pub whole_array_runtime_class: Arc<RuntimeClass<'gc_life>>,
    pub loader: LoaderName,
    pub len: jint,
    pub elems_base: *mut NativeJavaValue<'gc_life/*, 'l*/>,
    //pointer to elems bas
    pub phantom_data: PhantomData<&'l ()>,
    pub elem_type: CPDType,
}

pub struct ArrayIterator<'gc_life, 'l, 'k> {
    elems: &'l ArrayObject<'gc_life, 'k>,
    jvm: &'gc_life JVMState<'gc_life>,
    i: usize,
}

impl<'gc_life> Iterator for ArrayIterator<'gc_life, '_, '_> {
    type Item = JavaValue<'gc_life>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.elems.len as usize {
            return None;
        }
        let next = self.elems.get_i(self.jvm, self.i as i32);
        self.i += 1;
        Some(next)
    }
}

impl<'gc_life> ArrayObject<'gc_life, '_> {
    pub fn get_i(&self, jvm: &'gc_life JVMState<'gc_life>, i: i32) -> JavaValue<'gc_life> {
        let inner_type = &self.elem_type;
        unsafe { self.elems_base.offset(i as isize).as_ref().unwrap().to_java_value(inner_type, jvm) }
    }

    pub fn set_i(&mut self, jvm: &'gc_life JVMState<'gc_life>, i: i32, jv: JavaValue<'gc_life>) {
        let native = jv.to_native();
        unsafe { *self.elems_base.offset(i as isize).as_mut().unwrap() = native; }
    }

    pub fn array_iterator<'l>(&'l self, jvm: &'gc_life JVMState<'gc_life>) -> ArrayIterator<'gc_life, 'l, '_> {
        ArrayIterator { elems: self, jvm, i: 0 }
    }

    pub fn len(&self) -> i32 {
        self.len
    }

    pub fn new_array<'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, elems: Vec<JavaValue<'gc_life>>, type_: CPDType, monitor: Arc<Monitor2>) -> Result<Self, WasException> {
        check_resolved_class(jvm, int_state, CPDType::array(type_/*CPRefType::Array(box type_.clone())*/))?;
        Ok(Self {
            whole_array_runtime_class: todo!(),
            loader: todo!(),
            len: todo!(),
            elems_base: todo!(),
            phantom_data: Default::default(),
            elem_type: type_,
        })
    }
}

#[derive(Copy, Clone)]
pub union NativeJavaValue<'gc_life/*, 'l*/> {
    pub(crate) byte: i8,
    pub(crate) boolean: u8,
    pub(crate) short: i16,
    pub(crate) char: u16,
    pub(crate) int: i32,
    pub(crate) long: i64,
    pub(crate) float: f32,
    pub(crate) double: f64,
    pub(crate) object: *mut c_void,
    phantom_data: PhantomData<&'gc_life ()>,
    // phantom_data2: PhantomData<&'l ()>,//the owned java value needs to still be alive for ref to stay alive
    pub as_u64: u64,
}

impl Debug for NativeJavaValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "NativeJavaValue({:?})", self.object) }
    }
}

impl<'gc_life> NativeJavaValue<'gc_life> {
    pub fn to_java_value(&self, ptype: &CPDType, jvm: &'gc_life JVMState<'gc_life>) -> JavaValue<'gc_life> {
        unsafe {
            match ptype {
                CPDType::ByteType => JavaValue::Byte(self.byte),
                CPDType::CharType => JavaValue::Char(self.char),
                CPDType::DoubleType => JavaValue::Double(self.double),
                CPDType::FloatType => JavaValue::Float(self.float),
                CPDType::IntType => JavaValue::Int(self.int),
                CPDType::LongType => JavaValue::Long(self.long),
                CPDType::Ref(_) => match NonNull::new(self.object) {
                    None => JavaValue::Object(None),
                    Some(nonnull) => JavaValue::Object(Some(GcManagedObject::from_native(nonnull, jvm))),
                },
                CPDType::ShortType => JavaValue::Short(self.short),
                CPDType::BooleanType => JavaValue::Boolean(self.boolean),
                CPDType::VoidType => panic!(),
            }
        }
    }

    pub fn to_new_java_value(&self, ptype: &CPDType, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
        unsafe {
            match ptype {
                CPDType::ByteType => NewJavaValueHandle::Byte(self.byte),
                CPDType::CharType => NewJavaValueHandle::Char(self.char),
                CPDType::DoubleType => NewJavaValueHandle::Double(self.double),
                CPDType::FloatType => NewJavaValueHandle::Float(self.float),
                CPDType::IntType => NewJavaValueHandle::Int(self.int),
                CPDType::LongType => NewJavaValueHandle::Long(self.long),
                CPDType::Ref(_) => {
                    match NonNull::new(self.object) {
                        None => {
                            NewJavaValueHandle::Null
                        }
                        Some(ptr) => {
                            NewJavaValueHandle::Object(jvm.gc.register_root_reentrant(jvm, ptr))
                        }
                    }
                }
                CPDType::ShortType => NewJavaValueHandle::Short(self.short),
                CPDType::BooleanType => NewJavaValueHandle::Boolean(self.boolean),
                CPDType::VoidType => panic!(),
            }
        }
    }


    pub fn to_new_java_value_rtype(&self, rtype: &RuntimeType, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
        unsafe {
            match rtype {
                RuntimeType::DoubleType => NewJavaValueHandle::Double(self.double),
                RuntimeType::FloatType => NewJavaValueHandle::Float(self.float),
                RuntimeType::IntType => NewJavaValueHandle::Int(self.int),
                RuntimeType::LongType => NewJavaValueHandle::Long(self.long),
                RuntimeType::Ref(ref_) => {
                    match ref_ {
                        RuntimeRefType::Array(_) |
                        RuntimeRefType::Class(_) => {
                            match NonNull::new(self.object) {
                                Some(ptr) => {
                                    NewJavaValueHandle::Object(jvm.gc.register_root_reentrant(jvm, ptr))
                                }
                                None => {
                                    NewJavaValueHandle::Null
                                }
                            }
                        }
                        RuntimeRefType::NullType => {
                            assert_eq!(self.as_u64, 0);
                            NewJavaValueHandle::Null
                        }
                    }
                }
                RuntimeType::TopType => panic!(),
            }
        }
    }
}

#[derive(Copy, Clone)]
pub union StackNativeJavaValue<'gc_life> {
    int: i32,
    long: i64,
    float: f32,
    double: f64,
    pub(crate) object: *mut c_void,
    pub as_u64: u64,
    phantom_data: PhantomData<&'gc_life ()>,
}

impl<'gc_life> StackNativeJavaValue<'gc_life> {
    pub fn to_java_value(&self, rtype: RuntimeType, jvm: &'gc_life JVMState<'gc_life>) -> JavaValue<'gc_life> {
        unsafe {
            match rtype {
                RuntimeType::DoubleType => JavaValue::Double(self.double),
                RuntimeType::FloatType => JavaValue::Float(self.float),
                RuntimeType::IntType => {
                    // assert_eq!(self.as_u64 & 0xffff_ffff, 0);
                    JavaValue::Int(self.int)
                }
                RuntimeType::LongType => JavaValue::Long(self.long),
                RuntimeType::Ref(_) => match NonNull::new(self.object) {
                    None => JavaValue::Object(None),
                    Some(nonnull) => JavaValue::Object(Some(GcManagedObject::from_native(nonnull, jvm))),
                },
                RuntimeType::TopType => panic!(),
            }
        }
    }
}

pub mod native_objects {
    use std::collections::HashMap;
    use std::os::raw::c_void;
    use std::ptr::NonNull;
    use std::sync::Arc;

    use crate::threading::safepoints::Monitor2;

    pub struct ObjectMetaDataAndMonitors {
        monitors: HashMap<NonNull<c_void>, Arc<Monitor2>>,
    }
}

pub struct ObjectFieldsAndClass<'gc_life, 'l> {
    //ordered by alphabetical and super first
    pub fields: RwLock<&'l mut [NativeJavaValue<'gc_life>]>,
    pub class_pointer: Arc<RuntimeClass<'gc_life>>,
}

pub struct NormalObject<'gc_life, 'l> {
    pub objinfo: ObjectFieldsAndClass<'gc_life, 'l>,
    pub obj_ptr: Option<NonNull<NativeJavaValue<'gc_life>>>, //None means we have no object allocated backing this
}

impl<'gc_life, 'l> NormalObject<'gc_life, 'l> {
    pub fn set_var_top_level(&self, name: FieldName, jv: JavaValue<'gc_life>) {
        let (field_index, ptype) = self.objinfo.class_pointer.unwrap_class_class().field_numbers.get(&name).unwrap();
        /**unsafe {
                                                                                                                                            /*self.objinfo.fields[*field_index].get().as_mut()*/
                                                                                                                                        }.unwrap() = jv.to_native();*/
        todo!()
    }

    pub fn set_var(&self, class_pointer: Arc<RuntimeClass<'gc_life>>, name: FieldName, jv: JavaValue<'gc_life>) {
        jv.self_check();
        unsafe {
            let top_class_pointer = self.objinfo.class_pointer.clone();
            self.set_var_impl(top_class_pointer.unwrap_class_class(), class_pointer, name, jv, true)
        }
    }

    unsafe fn set_var_impl(&self, current_class_pointer: &RuntimeClassClass, class_pointer: Arc<RuntimeClass<'gc_life>>, name: FieldName, jv: JavaValue<'gc_life>, mut do_class_check: bool) {
        if current_class_pointer.class_view.name() == class_pointer.view().name() || !do_class_check {
            let field_index = match current_class_pointer.field_numbers.get(&name) {
                None => {
                    do_class_check = false;
                }
                Some((field_index, ptype)) => {
                    self.objinfo.fields.write().unwrap().get_mut(field_index.0 as usize).map(|set| *set = jv.to_native());
                    return;
                }
            };
        }
        if let Some(parent_class) = current_class_pointer.parent.as_ref() {
            self.set_var_impl(parent_class.unwrap_class_class(), class_pointer, name, jv, do_class_check);
        } else {
            panic!()
        }
    }

    pub fn get_var_top_level(&self, jvm: &'gc_life JVMState<'gc_life>, name: FieldName) -> JavaValue<'gc_life> {
        let name = name.into();
        let (field_index, ptype) = self.objinfo.class_pointer.unwrap_class_class().field_numbers.get(&name).unwrap();
        todo!()
        /*unsafe {
            self.objinfo.fields[*field_index].get().as_ref()
        }.unwrap().to_java_value(ptype.clone(), jvm)*/
    }

    // pub fn type_check(&self, class_pointer: Arc<RuntimeClass>) -> bool {
    //     Self::type_check_impl(&self.objinfo, class_pointer)
    // }

    /*fn type_check_impl(objinfo: &ObjectFieldsAndClass, class_pointer: Arc<RuntimeClass>) -> bool {
        if objinfo.class_pointer.view().name() == class_pointer.view().name() {
            return true;
        }
        return Self::type_check_impl(match objinfo.parent.as_ref() {
            Some(x) => x,
            None => return false,
        }, class_pointer);
    }*/

    /*fn find_matching_cname(objinfo: &ObjectFieldsAndClass, c_name: ReferenceTypeView) -> bool {
        if objinfo.class_pointer.view().name() == ReferenceTypeView::Class(ClassName::class()) && c_name == ReferenceTypeView::Class(ClassName::object()) {
            return true;
        }
        if objinfo.class_pointer.view().name() == c_name {
            return true;
        }
        let super_ = objinfo.parent.as_ref().map(|parent| Self::find_matching_cname(parent.deref(), c_name.clone())).unwrap_or(false);
        let interfaces = objinfo.class_pointer.unwrap_class_class().interfaces.iter().any(|interface| interface.view().name() == c_name.clone());
        return interfaces || super_;
    }*/

    pub fn get_var(&self, jvm: &'gc_life JVMState<'gc_life>, class_pointer: Arc<RuntimeClass<'gc_life>>, name: FieldName) -> JavaValue<'gc_life> {
        // if !self.type_check(class_pointer.clone()) {
        //     dbg!(name);
        //     dbg!(class_pointer.view().name());
        //     dbg!(self.objinfo.class_pointer.view().name());
        //     panic!()
        // }
        let res = unsafe { Self::get_var_impl(self, jvm, self.objinfo.class_pointer.unwrap_class_class(), class_pointer.clone(), name, true) };
        // self.expected_type_check(class_pointer, expected_type, name, &res);
        // res.self_check();
        res
    }

    /*fn expected_type_check(&self, class_pointer: Arc<RuntimeClass>, expected_type: PTypeView, name: String, res: &JavaValue) {
        match expected_type {
            PTypeView::ByteType => {}
            PTypeView::CharType => {}
            PTypeView::DoubleType => {}
            PTypeView::FloatType => {}
            PTypeView::IntType => {}
            PTypeView::LongType => {}
            PTypeView::Ref(ref_) => match ref_ {
                ReferenceTypeView::Class(c_name) => {
                    if !Self::find_matching_cname(&match match res.unwrap_object() {
                        Some(x) => x,
                        None => return,
                    }.try_unwrap_normal_object() {
                        Some(x) => x,
                        None => return,
                    }.objinfo, ReferenceTypeView::Class(c_name.clone())) {
                        dbg!(name);
                        dbg!(class_pointer.view().name());
                        dbg!(self.objinfo.class_pointer.view().name());
                        dbg!(self.objinfo.fields.keys().collect_vec());
                        dbg!(res.unwrap_normal_object().objinfo.class_pointer.view().name());
                        dbg!(c_name);
                        panic!()
                    }
                }
                ReferenceTypeView::Array(_) => {}
            }
            PTypeView::ShortType => {}
            PTypeView::BooleanType => {}
            PTypeView::VoidType => {}
            PTypeView::TopType => {}
            PTypeView::NullType => {}
            PTypeView::Uninitialized(_) => {}
            PTypeView::UninitializedThis => {}
            PTypeView::UninitializedThisOrClass(_) => {}
        };
    }*/

    unsafe fn get_var_impl(&self, jvm: &'gc_life JVMState<'gc_life>, current_class_pointer: &RuntimeClassClass, class_pointer: Arc<RuntimeClass<'gc_life>>, name: FieldName, mut do_class_check: bool) -> JavaValue<'gc_life> {
        if current_class_pointer.class_view.name() == class_pointer.view().name() || !do_class_check {
            match current_class_pointer.field_numbers.get(&name) {
                Some((field_number, ptype)) => {
                    todo!()
                    /*return self.objinfo.fields[*field_number].get().as_ref().unwrap().to_java_value(ptype.clone(), jvm);*/
                }
                None => {
                    do_class_check = false;
                }
            }
        }
        if let Some(parent_class) = current_class_pointer.parent.as_ref() {
            return self.get_var_impl(jvm, parent_class.unwrap_class_class(), class_pointer, name, do_class_check);
        } else {
            panic!()
        }
    }
}

impl<'gc_life> Debug for NormalObject<'gc_life, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        /*self.cycle_fmt(&vec![], f)*/
        todo!()
    }
}

pub fn default_value<'gc_life>(type_: &CPDType) -> NewJavaValueHandle<'gc_life> {
    match type_ {
        CPDType::ByteType => NewJavaValueHandle::Byte(0),
        CPDType::CharType => NewJavaValueHandle::Char('\u{000000}' as u16),
        CPDType::DoubleType => NewJavaValueHandle::Double(0.0),
        CPDType::FloatType => NewJavaValueHandle::Float(0.0),
        CPDType::IntType => NewJavaValueHandle::Int(0),
        CPDType::LongType => NewJavaValueHandle::Long(0),
        CPDType::Ref(_) => NewJavaValueHandle::Null,
        CPDType::ShortType => NewJavaValueHandle::Short(0),
        CPDType::BooleanType => NewJavaValueHandle::Boolean(0),
        CPDType::VoidType => panic!(),
    }
}

pub fn default_value_njv<'gc_life, 'any>(type_: &CPDType) -> NewJavaValue<'gc_life, 'any> {
    match type_ {
        CPDType::ByteType => NewJavaValue::Byte(0),
        CPDType::CharType => NewJavaValue::Char('\u{000000}' as u16),
        CPDType::DoubleType => NewJavaValue::Double(0.0),
        CPDType::FloatType => NewJavaValue::Float(0.0),
        CPDType::IntType => NewJavaValue::Int(0),
        CPDType::LongType => NewJavaValue::Long(0),
        CPDType::Ref(_) => NewJavaValue::Null,
        CPDType::ShortType => NewJavaValue::Short(0),
        CPDType::BooleanType => NewJavaValue::Boolean(0),
        CPDType::VoidType => panic!(),
    }
}

impl<'gc_life> ArrayObject<'gc_life, '_> {
    pub fn unwrap_object_array(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<Option<GcManagedObject<'gc_life>>> {
        /*unsafe { self.elems.get().as_ref() }.unwrap().iter().map(|x| { x.to_java_value(self.elem_type.clone(), jvm).unwrap_object() }).collect()*/
        todo!()
    }

    /*pub fn unwrap_mut(&self) -> &mut Vec<JavaValue<'gc_life>> {
        unsafe { self.elems.get().as_mut() }.unwrap()
    }*/
    /*pub fn unwrap_object_array_nonnull(&self) -> Vec<GcManagedObject<'gc_life>> {
        self.mut_array().iter().map(|x| { x.unwrap_object_nonnull() }).collect()
    }*/
    pub fn unwrap_byte_array(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<jbyte> {
        assert_eq!(self.elem_type, CPDType::ByteType);
        self.array_iterator(jvm).map(|x| x.unwrap_byte()).collect()
    }
    pub fn unwrap_char_array(&self, jvm: &'gc_life JVMState<'gc_life>) -> String {
        assert_eq!(self.elem_type, CPDType::CharType);
        let mut res = String::new();
        self.array_iterator(jvm).for_each(|x| res.push(x.unwrap_int() as u8 as char)); //todo fix this
        res
    }
}

impl<'gc_life> std::convert::From<Option<GcManagedObject<'gc_life>>> for JavaValue<'gc_life> {
    fn from(f: Option<GcManagedObject<'gc_life>>) -> Self {
        JavaValue::Object(f)
    }
}

pub trait ExceptionReturn {
    fn invalid_default() -> Self;
}

impl ExceptionReturn for i64 {
    fn invalid_default() -> Self {
        i64::MAX
    }
}

impl ExceptionReturn for i32 {
    fn invalid_default() -> Self {
        i32::MAX
    }
}

impl ExceptionReturn for i16 {
    fn invalid_default() -> Self {
        i16::MAX
    }
}

impl ExceptionReturn for i8 {
    fn invalid_default() -> Self {
        i8::MAX
    }
}

impl ExceptionReturn for u8 {
    fn invalid_default() -> Self {
        u8::MAX
    }
}

impl ExceptionReturn for u16 {
    fn invalid_default() -> Self {
        u16::MAX
    }
}

impl ExceptionReturn for f32 {
    fn invalid_default() -> Self {
        f32::MAX
    }
}

impl ExceptionReturn for f64 {
    fn invalid_default() -> Self {
        f64::MAX
    }
}

impl ExceptionReturn for jobject {
    fn invalid_default() -> Self {
        null_mut()
    }
}

impl ExceptionReturn for *const i8 {
    fn invalid_default() -> Self {
        null()
    }
}

impl ExceptionReturn for *const u16 {
    fn invalid_default() -> Self {
        null()
    }
}

impl ExceptionReturn for *mut c_void {
    fn invalid_default() -> Self {
        null_mut()
    }
}

impl<'gc_life> ExceptionReturn for JavaValue<'gc_life> {
    fn invalid_default() -> Self {
        JavaValue::Top
    }
}

impl ExceptionReturn for () {
    fn invalid_default() -> Self {
        ()
    }
}

impl ExceptionReturn for jfieldID {
    fn invalid_default() -> Self {
        null_mut()
    }
}

impl ExceptionReturn for jmethodID {
    fn invalid_default() -> Self {
        null_mut()
    }
}