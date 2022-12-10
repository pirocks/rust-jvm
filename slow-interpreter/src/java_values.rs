use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::ptr::{NonNull, null, null_mut};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};

use itertools::Itertools;

use add_only_static_vec::AddOnlyVec;
use array_memory_layout::layout::ArrayMemoryLayout;
use gc_memory_layout_common::early_startup::Regions;
use gc_memory_layout_common::memory_regions::MemoryRegions;
use jvmti_jni_bindings::{jbyte, jfieldID, jint, jmethodID, jobject, jvalue};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};
use rust_jvm_common::StackNativeJavaValue;

use crate::{AllocatedHandle, check_initing_or_inited_class};
use crate::accessor_ext::AccessorExt;
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::class_loading::{assert_inited_or_initing_class};
use crate::exceptions::WasException;
use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::jvm_state::JVMState;
use crate::new_java_values::{NewJavaValue, NewJavaValueHandle};
use crate::new_java_values::allocated_objects::{AllocatedArrayObjectHandle, AllocatedNormalObjectHandle};
use crate::new_java_values::unallocated_objects::{ObjectFields, UnAllocatedObject, UnAllocatedObjectArray, UnAllocatedObjectObject};
use crate::threading::safepoints::Monitor2;

pub struct GC<'gc> {
    pub memory_region: Mutex<MemoryRegions>,
    //doesn't really need to be atomic usize
    vm_temp_owned_roots: RwLock<HashMap<NonNull<c_void>, AtomicUsize>>,
    pub objects_that_live_for_gc_life: AddOnlyVec<AllocatedNormalObjectHandle<'gc>>,
}

impl<'gc> GC<'gc> {
    #[must_use]
    pub fn register_root_reentrant(&'gc self, jvm: &'gc JVMState<'gc>, ptr: NonNull<c_void>) -> AllocatedHandle<'gc> {
        let mut guard = self.vm_temp_owned_roots.write().unwrap();
        let count = guard.entry(ptr).or_insert(AtomicUsize::new(0));
        count.fetch_add(1, Ordering::SeqCst);
        let guard = self.memory_region.lock().unwrap();
        let cpdtype = guard.find_object_allocated_type(ptr).as_cpdtype();
        if cpdtype.is_array() {
            AllocatedHandle::Array(AllocatedArrayObjectHandle { jvm, ptr })
        } else {
            AllocatedHandle::NormalObject(AllocatedNormalObjectHandle { jvm, ptr })
        }
    }

    pub fn deregister_root_reentrant(&'gc self, ptr: NonNull<c_void>) {
        let mut guard = self.vm_temp_owned_roots.write().unwrap();
        let count = guard.get(&ptr).unwrap();
        count.fetch_sub(1, Ordering::SeqCst);
        if count.load(Ordering::SeqCst) == 0 {
            guard.remove(&ptr);
        }
    }

    pub fn handle_lives_for_gc_life(&'gc self, handle: AllocatedNormalObjectHandle<'gc>) -> &'gc AllocatedNormalObjectHandle<'gc> {
        let index = self.objects_that_live_for_gc_life.len();
        self.objects_that_live_for_gc_life.push(handle);
        let handle_ref: &'gc AllocatedNormalObjectHandle<'gc> = &self.objects_that_live_for_gc_life[index];
        handle_ref
    }

    pub fn allocate_object<'l>(&'gc self, jvm: &'gc JVMState<'gc>, object: UnAllocatedObject<'gc, 'l>) -> AllocatedHandle<'gc> {
        let allocated_object_type = match &object {
            UnAllocatedObject::Array(arr) => {
                assert!(arr.whole_array_runtime_class.cpdtype().is_array());
                runtime_class_to_allocated_object_type(jvm, arr.whole_array_runtime_class.clone(), LoaderName::BootstrapLoader, Some(arr.elems.len() as jint))
            }//todo loader name nonsense
            UnAllocatedObject::Object(obj) => runtime_class_to_allocated_object_type(
                jvm,
                obj.object_rc.clone(),
                LoaderName::BootstrapLoader,
                None,
            ),
        };
        let mut guard = self.memory_region.lock().unwrap();
        let (allocated, allocated_size) = guard.allocate_with_size(&allocated_object_type);
        unsafe { libc::memset(allocated.as_ptr(), 0, allocated_size.get()); }
        drop(guard);
        jvm.thread_state.debug_assert(jvm);
        let handle = self.register_root_reentrant(jvm, allocated);//should register before putting in all objects so can't be gced
        Self::init_allocated(object, allocated);
        jvm.thread_state.debug_assert(jvm);
        handle
    }

    fn init_allocated(object: UnAllocatedObject, allocated: NonNull<c_void>) {
        match object {
            UnAllocatedObject::Object(UnAllocatedObjectObject { object_rc, object_fields }) => {
                let ObjectFields { fields, hidden_fields } = object_fields;
                for (field_number, field) in fields.into_iter().chain(hidden_fields.into_iter()) {
                    let object_rc_class_class = object_rc.unwrap_class_class();
                    let object_layout = &object_rc_class_class.object_layout;
                    let field_type = object_layout.field_entry_type(field_number);
                    let accessor = object_layout.field_entry_pointer(allocated, field_number);
                    accessor.write_njv(field, field_type)
                }
            }
            UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems }) => {
                unsafe {
                    let sub_type = whole_array_runtime_class.cpdtype().unwrap_array_type();
                    let array_layout = ArrayMemoryLayout::from_cpdtype(sub_type);
                    array_layout.calculate_len_address(allocated).as_ptr().write(elems.len() as jint);
                    for (i, elem) in elems.into_iter().enumerate() {
                        //todo fix all ub usages of NativeJavaValue
                        let accessor = array_layout.calculate_index_address(allocated, i as i32);
                        accessor.write_njv(elem, sub_type)
                    }
                }
            }
        }
    }


    pub fn new(regions: Regions) -> Self {
        Self {
            memory_region: Mutex::new(MemoryRegions::new(regions)),
            vm_temp_owned_roots: RwLock::new(Default::default()),
            objects_that_live_for_gc_life: AddOnlyVec::new(),
        }
    }
}

#[derive(Clone)]
pub enum ByAddressAllocatedObject<'gc> {
    Owned(AllocatedNormalObjectHandle<'gc>),
    LookupOnly(usize),
}

impl<'gc> ByAddressAllocatedObject<'gc> {
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

    pub fn owned_inner(self) -> AllocatedNormalObjectHandle<'gc> {
        match self {
            ByAddressAllocatedObject::Owned(owned) => owned,
            ByAddressAllocatedObject::LookupOnly(_) => panic!()
        }
    }

    pub fn owned_inner_ref<'l>(&'l self) -> &'l AllocatedNormalObjectHandle<'gc> {
        match self {
            ByAddressAllocatedObject::Owned(owned) => owned,
            ByAddressAllocatedObject::LookupOnly(_) => panic!()
        }
    }
}

impl Hash for ByAddressAllocatedObject<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.raw_ptr_usize())
    }
}

impl Eq for ByAddressAllocatedObject<'_> {}

impl PartialEq for ByAddressAllocatedObject<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_ptr_usize() == other.raw_ptr_usize()
    }
}

pub struct GcManagedObject<'gc> {
    obj: Arc<Object<'gc, 'gc>>,
    //todo this double gc life thing is kinda unsafe
    raw_ptr: NonNull<c_void>,
    //allocated from a box
    gc: &'gc GC<'gc>,
    jvm: &'gc JVMState<'gc>,
}

impl<'gc> GcManagedObject<'gc> {

}

impl<'gc> Deref for GcManagedObject<'gc> {
    type Target = Object<'gc, 'gc>;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}


impl<'gc> DerefMut for GcManagedObject<'gc> {
    #[allow(mutable_transmutes)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { transmute(self.obj.deref()) }//todo mega unsafe this whole model needs a rethink
    }
}


impl<'gc> Clone for GcManagedObject<'gc> {
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

impl<'gc> GcManagedObject<'gc> {

    pub fn raw_ptr_usize(&self) -> usize {
        self.raw_ptr.as_ptr() as usize
    }
}

// #[derive(Copy)]
pub enum JavaValue<'gc> {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),

    Float(f32),
    Double(f64),
    Object(Option<GcManagedObject<'gc>>),

    Top, //should never be interacted with by the bytecode
}

impl<'gc> JavaValue<'gc> {
    pub fn to_new<'anything>(&self) -> NewJavaValue<'gc, 'anything> {
        todo!()
    }
}

impl<'gc> Debug for JavaValue<'gc> {
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

impl<'gc> JavaValue<'gc> {

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

    pub fn unwrap_object(&self) -> Option<GcManagedObject<'gc>> {
        self.try_unwrap_object().unwrap()
    }

    pub fn unwrap_array<'l>(&'l self) -> &'l ArrayObject<'gc, 'gc> {
        match self {
            JavaValue::Object(o) => o.as_ref().unwrap().unwrap_array(),
            _ => panic!(),
        }
    }

    pub fn try_unwrap_object(&self) -> Option<Option<GcManagedObject<'gc>>> {
        match self {
            JavaValue::Object(o) => Some(o.clone()),
            _ => {
                // dbg!(other);
                None
            }
        }
    }

    pub fn byte_array<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, bytes: Vec<u8>) -> Result<AllocatedHandle<'gc>, WasException<'gc>> {
        let byte_array = check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::ByteType))?;
        let elems = bytes.into_iter().map(|byte| NewJavaValue::Byte(byte as i8)).collect_vec();
        Ok(jvm.allocate_object(UnAllocatedObject::new_array(byte_array, elems)))
    }

    pub fn new_object(jvm: &'gc JVMState<'gc>, runtime_class: Arc<RuntimeClass<'gc>>, will_apply_intrinsic_data: bool) -> AllocatedNormalObjectHandle<'gc> {
        assert!(!runtime_class.view().is_abstract());

        let class_class = runtime_class.unwrap_class_class();


        let object_layout = &class_class.object_layout;

        let object_fields = if will_apply_intrinsic_data {
            ObjectFields::new_default_with_hidden_fields(object_layout)
        } else {
            ObjectFields::new_default_init_fields(object_layout)
        };
        jvm.allocate_object(UnAllocatedObject::Object(UnAllocatedObjectObject {
            object_rc: runtime_class,
            object_fields,
        })).unwrap_normal_object()
    }

    pub fn new_vec<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, len: usize, val: NewJavaValue<'gc, '_>, elem_type: CPDType) -> Result<AllocatedHandle<'gc>, WasException<'gc>> {
        let mut buf: Vec<NewJavaValue<'gc, '_>> = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(val.clone());
        }
        Ok(jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class: check_initing_or_inited_class(jvm, int_state, CPDType::array(elem_type)).unwrap(), elems: buf })/*Object::Array(ArrayObject::new_array(jvm, int_state, buf, elem_type, jvm.thread_state.new_monitor("array object monitor".to_string()))?)*/))
    }

    pub fn new_vec_from_vec(jvm: &'gc JVMState<'gc>, vals: Vec<NewJavaValue<'gc, '_>>, elem_type: CPDType) -> AllocatedHandle<'gc> {
        let whole_array_runtime_class = assert_inited_or_initing_class(jvm, CPDType::array(elem_type));
        jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems: vals }))
    }

    pub fn unwrap_char(&self) -> u16 {
        self.unwrap_int() as u16
    }
}

impl<'gc> Clone for JavaValue<'gc> {
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

pub enum Object<'gc, 'underlying_data> {
    Array(ArrayObject<'gc, 'underlying_data>),
    Object(NormalObject<'gc, 'underlying_data>),
}

//todo should really fix this
unsafe impl<'gc> Send for Object<'gc, '_> {}

unsafe impl<'gc> Sync for Object<'gc, '_> {}

impl<'gc, 'l> Object<'gc, 'l> {
    pub fn unwrap_normal_object(&self) -> &NormalObject<'gc, 'l> {
        match self {
            Object::Array(_) => panic!(),
            Object::Object(o) => o,
        }
    }

    pub fn unwrap_normal_object_mut(&mut self) -> &mut NormalObject<'gc, 'l> {
        match self {
            Object::Array(_) => panic!(),
            Object::Object(o) => o,
        }
    }

    pub fn try_unwrap_normal_object(&self) -> Option<&NormalObject<'gc, 'l>> {
        match self {
            Object::Array(_) => None,
            Object::Object(o) => Some(o),
        }
    }

    pub fn unwrap_array(&self) -> &ArrayObject<'gc, 'l> {
        match self {
            Object::Array(a) => a,
            Object::Object(obj) => {
                dbg!(obj.objinfo.class_pointer.view().name().unwrap_name());
                dbg!(obj.objinfo.class_pointer.unwrap_class_class().class_view.name());
                panic!()
            }
        }
    }

    pub fn unwrap_array_mut(&mut self) -> &mut ArrayObject<'gc, 'l> {
        match self {
            Object::Array(a) => a,
            Object::Object(obj) => {
                dbg!(obj.objinfo.class_pointer.view().name().unwrap_name());
                dbg!(obj.objinfo.class_pointer.unwrap_class_class().class_view.name());
                panic!()
            }
        }
    }
    pub fn is_array(&self) -> bool {
        match self {
            Object::Array(_) => true,
            Object::Object(_) => false,
        }
    }

    pub fn monitor(&self) -> &Monitor2 {
        match self {
            Object::Array(_a) => todo!(),  /*&a.monitor*/
            Object::Object(_o) => todo!(), /*&o.monitor*/
        }
    }

    pub fn monitor_unlock<'k>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>) {
        self.monitor().unlock(jvm, int_state).unwrap();
    }

    pub fn monitor_lock<'k>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>) {
        let monitor_to_lock = self.monitor();
        monitor_to_lock.lock(jvm, int_state).unwrap();
    }
}

pub struct ArrayObject<'gc, 'l> {
    pub whole_array_runtime_class: Arc<RuntimeClass<'gc>>,
    pub loader: LoaderName,
    pub len: jint,
    pub elems_base: *mut !,
    //pointer to elems bas
    pub phantom_data: PhantomData<&'l ()>,
    pub elem_type: CPDType,
}


// pub fn native_to_new_java_value<'gc>(native: NativeJavaValue<'gc>, ptype: CPDType, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
//     unsafe {
//         match ptype {
//             CPDType::ByteType => NewJavaValueHandle::Byte(native.byte),
//             CPDType::CharType => NewJavaValueHandle::Char(native.char),
//             CPDType::DoubleType => NewJavaValueHandle::Double(native.double),
//             CPDType::FloatType => NewJavaValueHandle::Float(native.float),
//             CPDType::IntType => NewJavaValueHandle::Int(native.int),
//             CPDType::LongType => NewJavaValueHandle::Long(native.long),
//             CPDType::Class(_) | CPDType::Array { .. } => {
//                 match NonNull::new(native.object) {
//                     None => {
//                         NewJavaValueHandle::Null
//                     }
//                     Some(ptr) => {
//                         NewJavaValueHandle::Object(jvm.gc.register_root_reentrant(jvm, ptr))
//                     }
//                 }
//             }
//             CPDType::ShortType => NewJavaValueHandle::Short(native.short),
//             CPDType::BooleanType => NewJavaValueHandle::Boolean(native.boolean),
//             CPDType::VoidType => panic!(),
//         }
//     }
// }


// pub fn array_native_to_new_java_value<'gc>(native: &ArrayNativeJV, ptype: CPDType, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
//     unsafe {
//         match ptype {
//             CPDType::ByteType => NewJavaValueHandle::Byte(native.byte),
//             CPDType::CharType => NewJavaValueHandle::Char(native.char),
//             CPDType::DoubleType => NewJavaValueHandle::Double(native.double),
//             CPDType::FloatType => NewJavaValueHandle::Float(native.float),
//             CPDType::IntType => NewJavaValueHandle::Int(native.int),
//             CPDType::LongType => NewJavaValueHandle::Long(native.long),
//             CPDType::Class(_) | CPDType::Array { .. } => {
//                 match NonNull::new(native.obj) {
//                     None => {
//                         NewJavaValueHandle::Null
//                     }
//                     Some(ptr) => {
//                         NewJavaValueHandle::Object(jvm.gc.register_root_reentrant(jvm, ptr))
//                     }
//                 }
//             }
//             CPDType::ShortType => NewJavaValueHandle::Short(native.short),
//             CPDType::BooleanType => NewJavaValueHandle::Boolean(native.bool),
//             CPDType::VoidType => panic!(),
//         }
//     }
// }


pub fn native_to_new_java_value_rtype<'gc>(native: StackNativeJavaValue<'gc>, rtype: RuntimeType, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
    unsafe {
        match rtype {
            RuntimeType::DoubleType => NewJavaValueHandle::Double(native.double),
            RuntimeType::FloatType => NewJavaValueHandle::Float(native.float),
            RuntimeType::IntType => NewJavaValueHandle::Int(native.int),
            RuntimeType::LongType => NewJavaValueHandle::Long(native.long),
            RuntimeType::Ref(ref_) => {
                match ref_ {
                    RuntimeRefType::Array(_) |
                    RuntimeRefType::Class(_) => {
                        match NonNull::new(native.object) {
                            Some(ptr) => {
                                NewJavaValueHandle::Object(jvm.gc.register_root_reentrant(jvm, ptr))
                            }
                            None => {
                                NewJavaValueHandle::Null
                            }
                        }
                    }
                    RuntimeRefType::NullType => {
                        assert_eq!(native.as_u64, 0);
                        NewJavaValueHandle::Null
                    }
                }
            }
            RuntimeType::TopType => panic!(),
        }
    }
}

pub fn native_to_new_java_value_cpdtype<'gc>(native: StackNativeJavaValue<'gc>, cpdtype: CPDType, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
    unsafe {
        match cpdtype {
            CPDType::DoubleType => NewJavaValueHandle::Double(native.double),
            CPDType::FloatType => NewJavaValueHandle::Float(native.float),
            CPDType::IntType => NewJavaValueHandle::Int(native.int),
            CPDType::LongType => NewJavaValueHandle::Long(native.long),
            CPDType::Class(_) | CPDType::Array { .. } => {
                match NonNull::new(native.object) {
                    Some(ptr) => {
                        NewJavaValueHandle::Object(jvm.gc.register_root_reentrant(jvm, ptr))
                    }
                    None => {
                        NewJavaValueHandle::Null
                    }
                }
            }
            CPDType::VoidType => panic!(),
            CPDType::BooleanType => {
                NewJavaValueHandle::Boolean(native.int as u8)
            }
            CPDType::ByteType => {
                NewJavaValueHandle::Byte(native.int as i8)
            }
            CPDType::ShortType => {
                NewJavaValueHandle::Short(native.int as i16)
            }
            CPDType::CharType => {
                NewJavaValueHandle::Char(native.int as u16)
            }
        }
    }
}

pub struct ObjectFieldsAndClass<'gc, 'l> {
    //ordered by alphabetical and super first
    pub fields: RwLock<&'l mut [!]>,
    pub class_pointer: Arc<RuntimeClass<'gc>>,
}

pub struct NormalObject<'gc, 'l> {
    pub objinfo: ObjectFieldsAndClass<'gc, 'l>,
    pub obj_ptr: Option<NonNull<!>>, //None means we have no object allocated backing this
}


pub fn default_value<'gc>(type_: CPDType) -> NewJavaValueHandle<'gc> {
    match type_ {
        CPDType::ByteType => NewJavaValueHandle::Byte(0),
        CPDType::CharType => NewJavaValueHandle::Char('\u{000000}' as u16),
        CPDType::DoubleType => NewJavaValueHandle::Double(0.0),
        CPDType::FloatType => NewJavaValueHandle::Float(0.0),
        CPDType::IntType => NewJavaValueHandle::Int(0),
        CPDType::LongType => NewJavaValueHandle::Long(0),
        CPDType::Class(_) => NewJavaValueHandle::Null,
        CPDType::Array { .. } => NewJavaValueHandle::Null,
        CPDType::ShortType => NewJavaValueHandle::Short(0),
        CPDType::BooleanType => NewJavaValueHandle::Boolean(0),
        CPDType::VoidType => panic!(),
    }
}

pub fn default_value_njv<'gc, 'any>(type_: &CPDType) -> NewJavaValue<'gc, 'any> {
    match type_ {
        CPDType::ByteType => NewJavaValue::Byte(0),
        CPDType::CharType => NewJavaValue::Char('\u{000000}' as u16),
        CPDType::DoubleType => NewJavaValue::Double(0.0),
        CPDType::FloatType => NewJavaValue::Float(0.0),
        CPDType::IntType => NewJavaValue::Int(0),
        CPDType::LongType => NewJavaValue::Long(0),
        CPDType::Class(_) => NewJavaValue::Null,
        CPDType::Array { .. } => NewJavaValue::Null,
        CPDType::ShortType => NewJavaValue::Short(0),
        CPDType::BooleanType => NewJavaValue::Boolean(0),
        CPDType::VoidType => panic!(),
    }
}

impl<'gc> ArrayObject<'gc, '_> {
}

impl<'gc> From<Option<GcManagedObject<'gc>>> for JavaValue<'gc> {
    fn from(f: Option<GcManagedObject<'gc>>) -> Self {
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

impl<'gc> ExceptionReturn for JavaValue<'gc> {
    fn invalid_default() -> Self {
        JavaValue::Top
    }
}

impl<'gc> ExceptionReturn for NewJavaValueHandle<'gc> {
    fn invalid_default() -> Self {
        NewJavaValueHandle::Top
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

impl ExceptionReturn for jvalue {
    fn invalid_default() -> Self {
        jvalue { j: 0 }
    }
}