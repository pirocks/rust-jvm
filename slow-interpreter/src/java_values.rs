use std::cell::{RefCell, UnsafeCell};
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::{size_of, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr::{NonNull, null, null_mut};
use std::slice;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::LocalKey;

use itertools::{Itertools, repeat_n};
use lazy_static::lazy_static;

use early_startup::Regions;
use jvmti_jni_bindings::{jbyte, jfieldID, jint, jlong, jmethodID, jobject};
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};

use crate::class_loading::check_resolved_class;
use crate::gc_memory_layout_common::{AllocatedObjectType, MemoryRegions};
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::jvm_state::JVMState;
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};
use crate::rust_jni::native_util::from_object;
use crate::threading::safepoints::Monitor2;

// thread_local! {
//     static THIS_THREAD_MEMORY_REGIONS: RefCell<MemoryRegions> = RefCell::new(MemoryRegions::new());
// }

pub struct GC<'gc_life> {
    pub memory_region: Mutex<MemoryRegions>,
    //doesn't really need to be atomic usize
    vm_temp_owned_roots: RwLock<HashMap<NonNull<c_void>, AtomicUsize>>,
    pub(crate) all_allocated_object: RwLock<HashSet<NonNull<c_void>>>,
    phantom: PhantomData<&'gc_life ()>,
}

impl<'gc_life> GC<'gc_life> {
    pub fn register_root_reentrant(&'gc_life self, ptr: NonNull<c_void>) {
        let mut guard = self.vm_temp_owned_roots.write().unwrap();
        let count = guard.entry(ptr).or_insert(AtomicUsize::new(0));
        count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn deregister_root_reentrant(&'gc_life self, ptr: NonNull<c_void>) {
        let mut guard = self.vm_temp_owned_roots.write().unwrap();
        let count = guard.get(&ptr).unwrap();
        count.fetch_sub(1, Ordering::SeqCst);
        if count.load(Ordering::SeqCst) == 0 {
            guard.remove(&ptr);
        }
    }

    pub fn allocate_object(&'gc_life self, jvm: &'gc_life JVMState<'gc_life>, object: Object<'gc_life, 'l>) -> GcManagedObject<'gc_life> {
        // let ptr = NonNull::new(Box::into_raw(box object)).unwrap();
        let mut guard = self.memory_region.lock().unwrap();
        let allocated_object_type = match &object {
            Object::Array(arr) => {
                todo!()
            }
            Object::Object(obj) => {
                runtime_class_to_allocated_object_type(&obj.objinfo.class_pointer.clone(), LoaderName::BootstrapLoader, None)
            }
        };
        let mut memory_region = guard.find_or_new_region_for(allocated_object_type, None);
        let allocated = memory_region.deref_mut().get_mut().get_allocation();
        self.all_allocated_object.write().unwrap().insert(allocated);
        self.register_root_reentrant(allocated);
        GcManagedObject {
            obj: match object {
                Object::Array(ArrayObject {
                                  len,
                                  elems,
                                  phantom_data,
                                  elem_type
                              }) => {
                    assert_eq!(len as usize, elems.len());
                    unsafe {
                        let new_elems = slice::from_raw_parts_mut(allocated.cast::<NativeJavaValue<'gc_life>>().as_ptr(), len as usize);
                        for (i, elem) in elems.iter().enumerate() {
                            new_elems[i] = *elem;
                        }
                        Arc::new(Object::Array(ArrayObject {
                            len,
                            elems: new_elems,
                            phantom_data,
                            elem_type,
                        }))
                    }
                }
                Object::Object(NormalObject { objinfo: ObjectFieldsAndClass { fields, class_pointer }, obj_ptr }) => {
                    let new_fields = unsafe { slice::from_raw_parts_mut(allocated.cast::<NativeJavaValue<'gc_life>>().as_ptr(), fields.read().unwrap().len()) };
                    for (i, field) in fields.read().unwrap().iter().enumerate() {
                        new_fields[i] = *field;
                    }
                    Arc::new(Object::Object(NormalObject { objinfo: ObjectFieldsAndClass { fields: RwLock::new(new_fields), class_pointer }, obj_ptr }))
                }
            },
            raw_ptr: allocated,
            gc: self,
            jvm,
        }
    }

    fn gc_recurse(obj: NonNull<c_void>, visited: &mut HashSet<NonNull<c_void>>) {
        if visited.contains(&obj) {
            return;
        }
        unsafe {
            if obj.as_ptr() == transmute(0xDEADDEADDEADDEADusize) {
                //todo need better handling of top, but is fine for now
                return;
            }
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

    fn gc_recurse_super(obj: &NormalObject<'gc_life, 'l>, class_class: &RuntimeClassClass, visited: &mut HashSet<NonNull<c_void>>) {
        for (_name, (index, rtype)) in class_class.field_numbers.iter() {
            if let CPDType::Ref(_) = rtype {
                Self::gc_recurse(match NonNull::new(todo!()/*unsafe { obj.objinfo.fields[*index].get().as_ref().unwrap().object }*/) {
                    None => continue,
                    Some(ptr) => ptr
                }, visited)
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
        if !jvm.vm_live() {
            return;
        }
        let interpreter_states = jvm.thread_state.all_java_threads.read().unwrap().values().map(|jt| {
            unsafe { jt.gc_suspend(); }
            let guard = InterpreterStateGuard {
                int_state: Some(jt.interpreter_state.write().unwrap()),
                thread: jt.clone(),
                registered: false,
            };
            (guard.cloned_stack_snapshot(jvm), guard.throw())
        }).collect_vec();
        let mut roots = HashSet::new();
        unsafe {
            // dbg!(interpreter_states.len());
            for (stack, throw) in interpreter_states {
                // dbg!(stack.len());
                if let Some(throw) = throw {
                    roots.insert(throw.raw_ptr);
                }
                for stack_entry in stack {
                    // dbg!(stack_entry.operand_stack.len());
                    // dbg!(stack_entry.local_vars().len());
                    for local_var in stack_entry.local_vars().iter().cloned().chain(
                        stack_entry.operand_stack().iter().cloned()).chain(
                        stack_entry.native_local_refs.iter().flat_map(|hashet| hashet.iter().map(|raw| JavaValue::Object(from_object(jvm, *raw))))) {
                        if let JavaValue::Object(Some(gc_managed)) = local_var {
                            // dbg!(gc_managed.raw_ptr);
                            roots.insert(gc_managed.raw_ptr);
                        }
                    }
                }
            }
        }
        for (root, _) in self.vm_temp_owned_roots.read().unwrap().iter() {
            roots.insert(todo!()/*root.clone()*/);
        }
        for root in jvm.classes.read().unwrap().class_object_pool.left_values().map(|by_address| by_address.0.clone()).chain(
            jvm.classes.read().unwrap().anon_class_live_object_ldc_pool.read().unwrap().iter().cloned()).chain(
            jvm.class_loaders.read().unwrap().right_values().map(|by_address| by_address.0.clone())).chain(
            jvm.protection_domains.read().unwrap().right_values().map(|by_address| by_address.0.clone())).chain(
            jvm.string_internment.read().unwrap().strings.values().cloned()).chain(
            jvm.thread_state.system_thread_group.read().unwrap().as_ref().map(|thread_group| thread_group.clone().object()).iter().cloned()).chain(
            jvm.thread_state.all_java_threads.read().unwrap().values().map(|jt| jt.thread_object().object())).chain(
            jvm.classes.read().unwrap().initiating_loaders.values()
                .flat_map(|(_loader, class)| { class.try_unwrap_class_class() })
                .flat_map(|class| class.static_vars.read().unwrap().values().flat_map(|jv| jv.try_unwrap_object()).flatten().collect_vec())) {
            roots.insert(root.raw_ptr);
        }
        self.gc_with_roots(todo!()/*roots*/)
    }

    pub fn new(regions: early_startup::Regions) -> Self {
        Self {
            memory_region: Mutex::new(MemoryRegions::new(regions)),
            vm_temp_owned_roots: RwLock::new(Default::default()),
            all_allocated_object: Default::default(),
            phantom: PhantomData::default(),
        }
    }
}


pub struct ByAddressGcManagedObject<'gc_life>(pub GcManagedObject<'gc_life>);

impl Hash for ByAddressGcManagedObject<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.raw_ptr_usize())
    }
}

impl Eq for ByAddressGcManagedObject<'_> {}

impl PartialEq for ByAddressGcManagedObject<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.raw_ptr_usize() == other.0.raw_ptr_usize()
    }
}

pub struct GcManagedObject<'gc_life> {
    obj: Arc<Object<'gc_life, 'gc_life>>,
    raw_ptr: NonNull<c_void>,
    //allocated from a box
    gc: &'gc_life GC<'gc_life>,
    jvm: &'gc_life JVMState<'gc_life>,
}

impl<'gc_life> GcManagedObject<'gc_life> {
    pub fn from_native(raw_ptr: NonNull<c_void>, jvm: &'gc_life JVMState<'gc_life>) -> Self {
        jvm.gc.register_root_reentrant(raw_ptr);
        let guard = jvm.gc.memory_region.lock().unwrap();
        let allocated_type = guard.find_object_allocated_type(raw_ptr);
        let obj = match allocated_type {
            AllocatedObjectType::Class { size, name, loader } => {
                let classes = jvm.classes.read().unwrap();
                let runtime_class = classes.loaded_classes_by_type.get(loader).unwrap().get(&(*name).into()).unwrap();
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
            AllocatedObjectType::ObjectArray { .. } => todo!(),
            AllocatedObjectType::PrimitiveArray { .. } => todo!(),
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
}

impl<'gc_life> Deref for GcManagedObject<'gc_life> {
    type Target = Object<'gc_life, 'gc_life>;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}


impl<'gc_life> Clone for GcManagedObject<'gc_life> {
    fn clone(&self) -> Self {
        //this doesn't leak b/c if we ever try to create a cycle we put into a field and deregister as a root.
        self.gc.register_root_reentrant(self.raw_ptr);
        Self {
            obj: self.obj.clone(),
            raw_ptr: self.raw_ptr,
            gc: self.gc,
            jvm: self.jvm,
        }
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

    Top,//should never be interacted with by the bytecode
}


impl<'gc_life> JavaValue<'gc_life> {
    pub(crate) fn self_check(&self) {
        if let JavaValue::Object(Some(obj)) = self {
            obj.self_check()
        }
    }


    pub fn to_native(&self) -> NativeJavaValue<'gc_life> {
        match self.clone() {
            JavaValue::Long(val_) => {
                NativeJavaValue { long: val_ }
            }
            JavaValue::Int(val_) => {
                NativeJavaValue { int: val_ }
            }
            JavaValue::Short(val_) => {
                NativeJavaValue { int: val_ as i32 }
            }
            JavaValue::Byte(val_) => {
                NativeJavaValue { int: val_ as i32 }
            }
            JavaValue::Boolean(val_) => {
                NativeJavaValue { int: val_ as i32 }
            }
            JavaValue::Char(val_) => {
                NativeJavaValue { int: val_ as i32 }
            }
            JavaValue::Float(val_) => {
                NativeJavaValue { float: val_ }
            }
            JavaValue::Double(val_) => {
                NativeJavaValue { double: val_ }
            }
            JavaValue::Object(val_) => {
                NativeJavaValue {
                    object: match val_ {
                        None => null_mut(),
                        Some(gc_managed) => {
                            // gc_managed.obj
                            todo!()
                        }
                    }
                }
            }
            JavaValue::Top => unsafe { transmute(0xDEADDEADDEADDEADusize) }
        }
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
                writeln!(f, "Long:{}", elem)
            }
            JavaValue::Int(elem) => {
                writeln!(f, "Int:{}", elem)
            }
            JavaValue::Short(elem) => {
                writeln!(f, "Short:{}", elem)
            }
            JavaValue::Byte(elem) => {
                writeln!(f, "Byte:{}", elem)
            }
            JavaValue::Boolean(elem) => {
                writeln!(f, "Boolean:{}", elem)
            }
            JavaValue::Char(elem) => {
                writeln!(f, "Char:{}", elem)
            }
            JavaValue::Float(elem) => {
                writeln!(f, "Float:{}", elem)
            }
            JavaValue::Double(elem) => {
                writeln!(f, "Double:{}", elem)
            }
            JavaValue::Object(_) => {
                writeln!(f, "obj")
            }
            JavaValue::Top => writeln!(f, "top")
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
            JavaValue::Int(i) => {
                *i
            }
            JavaValue::Byte(i) => {
                *i as i32
            }
            JavaValue::Boolean(i) => {
                *i as i32
            }
            JavaValue::Char(c) => {
                *c as i32
            }
            JavaValue::Short(i) => {
                *i as i32
            }
            _ => {
                return None;
            }
        }.into()
    }

    pub fn unwrap_float(&self) -> f32 {
        self.try_unwrap_float().unwrap()
    }
    pub fn try_unwrap_float(&self) -> Option<f32> {
        match self {
            JavaValue::Float(f) => {
                (*f).into()
            }
            _ => None
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
            JavaValue::Double(f) => {
                (*f).into()
            }
            _ => None
        }
    }

    pub fn try_unwrap_long(&self) -> Option<i64> {
        match self {
            JavaValue::Long(l) => {
                (*l).into()
            }
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
            JavaValue::Object(o) => {
                o.as_ref().unwrap().unwrap_array()
            }
            _ => panic!()
        }
    }


    pub fn try_unwrap_object(&self) -> Option<Option<GcManagedObject<'gc_life>>> {
        match self {
            JavaValue::Object(o) => {
                Some(o.clone())
            }
            _ => {
                // dbg!(other);
                None
            }
        }
    }

    pub fn deep_clone(&self, jvm: &'gc_life JVMState<'gc_life>) -> Self {
        match &self {
            JavaValue::Object(o) => {
                JavaValue::Object(match o {
                    None => None,
                    Some(o) => {
                        jvm.allocate_object(o.deref().deep_clone(jvm)).into()
                    }
                })
            }
            JavaValue::Top => panic!(),
            jv => (*jv).clone()
        }
    }
    pub fn empty_byte_array(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<JavaValue<'gc_life>, WasException> {
        Ok(JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            vec![],
            CPDType::ByteType,
            jvm.thread_state.new_monitor("".to_string()),
        )?)))))
    }

    fn new_object_impl(runtime_class: &Arc<RuntimeClass<'gc_life>>) -> ObjectFieldsAndClass<'gc_life, 'gc_life> {
        let fields = repeat_n(JavaValue::Top, runtime_class.unwrap_class_class().num_vars()).map(|jv| UnsafeCell::new(jv.to_native())).collect_vec();
        ObjectFieldsAndClass {
            fields: todo!(),
            class_pointer: runtime_class.clone(),
        }
    }

    pub fn new_object(jvm: &'gc_life JVMState<'gc_life>, runtime_class: Arc<RuntimeClass<'gc_life>>) -> Option<GcManagedObject<'gc_life>> {
        assert!(!runtime_class.view().is_abstract());

        let class_class = runtime_class.unwrap_class_class();
        let mut fields = (0..class_class.recursive_num_fields).map(|_| NativeJavaValue { object: null_mut() }).collect_vec();

        jvm.allocate_object(Object::Object(NormalObject {
            objinfo: ObjectFieldsAndClass { fields: RwLock::new(&mut fields), class_pointer: runtime_class },
            obj_ptr: None,
        })).into()
    }

    pub fn new_vec(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, len: usize, val: JavaValue<'gc_life>, elem_type: CPDType) -> Result<Option<GcManagedObject<'gc_life>>, WasException> {
        let mut buf: Vec<JavaValue<'gc_life>> = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(val.clone());
        }
        Ok(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            buf,
            elem_type,
            jvm.thread_state.new_monitor("array object monitor".to_string()),
        )?))))
    }

    pub fn new_vec_from_vec(jvm: &'gc_life JVMState<'gc_life>, vals: Vec<JavaValue<'gc_life>>, elem_type: CPDType) -> JavaValue<'gc_life> {
        JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject {
            len: todo!(),
            elems: todo!(),
            phantom_data: Default::default(),
            elem_type,
        }))))
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
            _ => None
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
            JavaValue::Object(obj) => {
                RuntimeType::Ref(match obj {
                    None => RuntimeRefType::NullType,
                    Some(not_null) => {
                        match not_null.deref() {
                            Object::Array(array) => {
                                RuntimeRefType::Array(array.elem_type.clone().into())
                            }
                            Object::Object(obj) => {
                                RuntimeRefType::Class(obj.objinfo.class_pointer.cpdtype().unwrap_class_type())
                            }
                        }
                    }
                })
            }
            JavaValue::Top => RuntimeType::TopType
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
            JavaValue::Long(x) => {
                match other {
                    JavaValue::Long(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Int(x) => {
                match other {
                    JavaValue::Int(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Short(x) => {
                match other {
                    JavaValue::Short(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Byte(x) => {
                match other {
                    JavaValue::Byte(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Boolean(x) => {
                match other {
                    JavaValue::Boolean(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Char(x) => {
                match other {
                    JavaValue::Char(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Float(x) => {
                match other {
                    JavaValue::Float(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Double(x) => {
                match other {
                    JavaValue::Double(x1) => x == x1,
                    _ => false
                }
            }
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
                    _ => false
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
            Some(res) => res
        };
        // unsafe { self.unwrap_normal_object().objinfo.fields[*field_number].get().as_ref() }.unwrap().to_java_value(rtype.clone(), jvm)
        todo!()
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
                // dbg!(obj.objinfo.class_pointer.view().name());
                // dbg!(obj.objinfo.class_pointer.unwrap_class_class().class_view.name());
                panic!()
            }
        }
    }

    pub fn deep_clone(&self, jvm: &'gc_life JVMState<'gc_life>) -> Self {
        match &self {
            Object::Array(a) => {
                // let sub_array = a.array_iterator(jvm).map(|x| x.deep_clone(jvm).to_native()).collect();//todo
                todo!();
                Object::Array(ArrayObject { len: todo!(), elems: todo!(), phantom_data: Default::default(), elem_type: a.elem_type.clone() })
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
            Object::Array(a) => todo!()/*&a.monitor*/,
            Object::Object(o) => todo!()/*&o.monitor*/,
        }
    }

    pub fn monitor_unlock(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>) {
        self.monitor().unlock(jvm, int_state).unwrap();
    }

    pub fn monitor_lock<'k>(&'_ self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'k>) {
        let monitor_to_lock = self.monitor();
        monitor_to_lock.lock(jvm, int_state).unwrap();
    }
}

pub struct ArrayObject<'gc_life, 'l> {
    // pub elems: UnsafeCell<Vec<NativeJavaValue<'gc_life>>>,
    pub len: jint,
    pub elems: &'l mut [NativeJavaValue<'gc_life>],
    pub phantom_data: PhantomData<&'gc_life ()>,
    pub elem_type: CPDType,
    // pub monitor: Arc<Monitor2>,
}

pub struct ArrayIterator<'gc_life, 'l, 'k> {
    elems: &'l ArrayObject<'gc_life, 'k>,
    jvm: &'gc_life JVMState<'gc_life>,
    i: usize,
}

impl<'gc_life> Iterator for ArrayIterator<'gc_life, '_, '_> {
    type Item = JavaValue<'gc_life>;

    fn next(&mut self) -> Option<Self::Item> {
        let elems = todo!()/*unsafe { todo!()/*self.elems.elems.get().as_ref()*/ }.unwrap()*/;
        /*let res = match elems.get(self.i) {
            None => None,
            Some(item) => {
                Some(item.to_java_value(self.elems.elem_type.clone(), self.jvm))
            }
        };*/
        todo!();
        self.i += 1;
        todo!()
    }
}

impl<'gc_life> ArrayObject<'gc_life, '_> {
    pub fn get_i(&self, jvm: &'gc_life JVMState<'gc_life>, i: i32) -> JavaValue<'gc_life> {
        /*unsafe { todo!()/*self.elems.get().as_ref()*/ }.unwrap()[i as usize].to_java_value(self.elem_type.clone(), jvm)*/
        todo!()
    }

    pub fn set_i(&self, jvm: &'gc_life JVMState<'gc_life>, i: i32, jv: JavaValue<'gc_life>) {
        /*unsafe { self.elems.get().as_mut() }.unwrap()[i as usize] = jv.to_native();*/
        todo!()
    }

    pub fn array_iterator(&'l self, jvm: &'gc_life JVMState<'gc_life>) -> ArrayIterator<'gc_life, 'l, '_> {
        ArrayIterator {
            elems: self,
            jvm,
            i: 0,
        }
    }

    pub fn len(&self) -> i32 {
        /*unsafe { self.elems.get().as_ref() }.unwrap().len() as i32*/
        todo!()
    }

    pub fn new_array(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, elems: Vec<JavaValue<'gc_life>>, type_: CPDType, monitor: Arc<Monitor2>) -> Result<Self, WasException> {
        check_resolved_class(jvm, int_state, CPDType::Ref(CPRefType::Array(box type_.clone())))?;
        Ok(Self {
            len: todo!(),
            elems: todo!(),
            phantom_data: Default::default(),
            elem_type: type_,
        })
    }
}

#[derive(Copy, Clone)]
pub union NativeJavaValue<'gc_life> {
    byte: i8,
    boolean: u8,
    short: i16,
    char: u16,
    int: i32,
    long: i64,
    float: f32,
    double: f64,
    pub(crate) object: *mut c_void,
    phantom_data: PhantomData<&'gc_life ()>,
}

impl<'gc_life> NativeJavaValue<'gc_life> {
    pub fn to_java_value(&self, ptype: CPDType, jvm: &'gc_life JVMState<'gc_life>) -> JavaValue<'gc_life> {
        unsafe {
            match ptype {
                CPDType::ByteType => {
                    JavaValue::Byte(self.byte)
                }
                CPDType::CharType => {
                    JavaValue::Char(self.char)
                }
                CPDType::DoubleType => {
                    JavaValue::Double(self.double)
                }
                CPDType::FloatType => {
                    JavaValue::Float(self.float)
                }
                CPDType::IntType => {
                    JavaValue::Int(self.int)
                }
                CPDType::LongType => {
                    JavaValue::Long(self.long)
                }
                CPDType::Ref(_) => {
                    match NonNull::new(self.object) {
                        None => {
                            JavaValue::Object(None)
                        }
                        Some(nonnull) => {
                            JavaValue::Object(Some(GcManagedObject::from_native(nonnull, jvm)))
                        }
                    }
                }
                CPDType::ShortType => {
                    JavaValue::Short(self.short)
                }
                CPDType::BooleanType => {
                    JavaValue::Boolean(self.boolean)
                }
                CPDType::VoidType => panic!()
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
    phantom_data: PhantomData<&'gc_life ()>,
}

impl<'gc_life> StackNativeJavaValue<'gc_life> {
    pub fn to_java_value(&self, rtype: RuntimeType, jvm: &'gc_life JVMState<'gc_life>) -> JavaValue<'gc_life> {
        unsafe {
            match rtype {
                RuntimeType::DoubleType => {
                    JavaValue::Double(self.double)
                }
                RuntimeType::FloatType => {
                    JavaValue::Float(self.float)
                }
                RuntimeType::IntType => {
                    JavaValue::Int(self.int)
                }
                RuntimeType::LongType => {
                    JavaValue::Long(self.long)
                }
                RuntimeType::Ref(_) => {
                    match NonNull::new(self.object) {
                        None => {
                            JavaValue::Object(None)
                        }
                        Some(nonnull) => {
                            JavaValue::Object(Some(GcManagedObject::from_native(nonnull, jvm)))
                        }
                    }
                }
                RuntimeType::TopType => panic!()
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
                    self.objinfo.fields.write().unwrap().get_mut(*field_index).map(|set| *set = jv.to_native());
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

pub fn default_value<'gc_life>(type_: CPDType) -> JavaValue<'gc_life> {
    match type_ {
        CPDType::ByteType => JavaValue::Byte(0),
        CPDType::CharType => JavaValue::Char('\u{000000}' as u16),
        CPDType::DoubleType => JavaValue::Double(0.0),
        CPDType::FloatType => JavaValue::Float(0.0),
        CPDType::IntType => JavaValue::Int(0),
        CPDType::LongType => JavaValue::Long(0),
        CPDType::Ref(_) => JavaValue::Object(None),
        CPDType::ShortType => JavaValue::Short(0),
        CPDType::BooleanType => JavaValue::Boolean(0),
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
        self.array_iterator(jvm).map(|x| { x.unwrap_byte() }).collect()
    }
    pub fn unwrap_char_array(&self, jvm: &'gc_life JVMState<'gc_life>) -> String {
        assert_eq!(self.elem_type, CPDType::CharType);
        let mut res = String::new();
        self.array_iterator(jvm).for_each(|x| { res.push(x.unwrap_int() as u8 as char) });//todo fix this
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
