use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use classfile_view::view::{ArrayView, ClassView, HasAccessFlags, PrimitiveView};
use gc_memory_layout_common::NativeJavaValue;
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{FieldName, MethodName};
use rust_jvm_common::method_shape::MethodShape;

use crate::instructions::ldc::from_constant_pool_entry;
use crate::interpreter::{run_function, WasException};
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::{default_value, native_to_new_java_value};
use crate::jit::MethodResolver;
use crate::jvm_state::{ClassStatus, JVMState};
use crate::new_java_values::NewJavaValueHandle;
use crate::NewJavaValue;
use crate::stack_entry::{StackEntryPush};

#[derive(Debug)]
pub enum RuntimeClass<'gc> {
    Byte,
    Boolean,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Void,
    Array(RuntimeClassArray<'gc>),
    Object(RuntimeClassClass<'gc>),
    Top,
}

impl<'gc> RuntimeClass<'gc> {
    pub fn cpdtype(&self) -> CPDType {
        match self {
            RuntimeClass::Byte => CPDType::ByteType,
            RuntimeClass::Boolean => CPDType::BooleanType,
            RuntimeClass::Short => CPDType::ShortType,
            RuntimeClass::Char => CPDType::CharType,
            RuntimeClass::Int => CPDType::IntType,
            RuntimeClass::Long => CPDType::LongType,
            RuntimeClass::Float => CPDType::FloatType,
            RuntimeClass::Double => CPDType::DoubleType,
            RuntimeClass::Void => CPDType::VoidType,
            RuntimeClass::Array(arr) => CPDType::array(arr.sub_class.cpdtype()),
            RuntimeClass::Object(o) => CPDType::Ref(CPRefType::Class(o.class_view.name().unwrap_name())),
            RuntimeClass::Top => panic!(),
        }
    }
    pub fn view(&self) -> Arc<dyn ClassView> {
        match self {
            RuntimeClass::Byte => Arc::new(PrimitiveView::Byte),
            RuntimeClass::Boolean => Arc::new(PrimitiveView::Boolean),
            RuntimeClass::Short => Arc::new(PrimitiveView::Short),
            RuntimeClass::Char => Arc::new(PrimitiveView::Char),
            RuntimeClass::Int => Arc::new(PrimitiveView::Int),
            RuntimeClass::Long => Arc::new(PrimitiveView::Long),
            RuntimeClass::Float => Arc::new(PrimitiveView::Float),
            RuntimeClass::Double => Arc::new(PrimitiveView::Double),
            RuntimeClass::Void => Arc::new(PrimitiveView::Void),
            RuntimeClass::Array(arr) => Arc::new(ArrayView { sub: arr.sub_class.view() }),
            RuntimeClass::Object(o) => o.class_view.clone(),
            RuntimeClass::Top => panic!(),
        }
    }
    pub fn status(&self) -> ClassStatus {
        match self {
            RuntimeClass::Byte => ClassStatus::INITIALIZED,
            RuntimeClass::Boolean => ClassStatus::INITIALIZED,
            RuntimeClass::Short => ClassStatus::INITIALIZED,
            RuntimeClass::Char => ClassStatus::INITIALIZED,
            RuntimeClass::Int => ClassStatus::INITIALIZED,
            RuntimeClass::Long => ClassStatus::INITIALIZED,
            RuntimeClass::Float => ClassStatus::INITIALIZED,
            RuntimeClass::Double => ClassStatus::INITIALIZED,
            RuntimeClass::Void => ClassStatus::INITIALIZED,
            RuntimeClass::Array(a) => a.sub_class.status(),
            RuntimeClass::Object(o) => *o.status.read().unwrap(),
            RuntimeClass::Top => panic!(),
        }
    }

    pub fn set_status(&self, status: ClassStatus) {
        match self {
            RuntimeClass::Array(a) => a.sub_class.set_status(status),
            RuntimeClass::Object(o) => *o.status.write().unwrap() = status,
            _ => {}
        }
    }

    pub fn unwrap_class_class(&self) -> &RuntimeClassClass<'gc> {
        self.try_unwrap_class_class().unwrap()
    }

    pub fn try_unwrap_class_class(&'_ self) -> Option<&'_ RuntimeClassClass<'gc>> {
        match self {
            RuntimeClass::Object(classclass) => Some(classclass),
            _ => None,
        }
    }
}

pub struct StaticVarGuard<'gc, 'l> {
    jvm: &'gc JVMState<'gc>,
    data_guard: RwLockWriteGuard<'l, HashMap<FieldName, NativeJavaValue<'gc>>>,
    types: &'l HashMap<FieldName, CPDType>,
}

impl<'gc, 'l> StaticVarGuard<'gc, 'l> {
    pub fn try_get(&self, name: FieldName) -> Option<NewJavaValueHandle<'gc>> {
        let cpd_type = self.types.get(&name)?;
        let native = *self.data_guard.get(&name)?;
        Some(native_to_new_java_value(native,cpd_type, self.jvm))
    }

    pub fn get(&self, name: FieldName) -> NewJavaValueHandle<'gc> {
        self.try_get(name).unwrap()
    }

    pub fn set(&mut self, name: FieldName, elem: NewJavaValueHandle<'gc>) {
        let cpd_type = self.types.get(&name).unwrap();
        self.data_guard.insert(name, elem.as_njv().to_native());
    }
}

impl<'gc> RuntimeClass<'gc> {
    pub fn static_vars<'l>(&'l self, jvm: &'gc JVMState<'gc>) -> StaticVarGuard<'gc, 'l> {
        match self {
            RuntimeClass::Byte => panic!(),
            RuntimeClass::Boolean => panic!(),
            RuntimeClass::Short => panic!(),
            RuntimeClass::Char => panic!(),
            RuntimeClass::Int => panic!(),
            RuntimeClass::Long => panic!(),
            RuntimeClass::Float => panic!(),
            RuntimeClass::Double => panic!(),
            RuntimeClass::Void => panic!(),
            RuntimeClass::Array(_) => panic!(),
            RuntimeClass::Object(o) => {
                StaticVarGuard {
                    jvm,
                    data_guard: o.static_vars.write().unwrap(),
                    types: &o.static_var_types,
                }
            }
            RuntimeClass::Top => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeClassArray<'gc> {
    pub sub_class: Arc<RuntimeClass<'gc>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FieldNumber(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MethodNumber(pub(crate) u32);

pub struct RuntimeClassClass<'gc> {
    pub class_view: Arc<dyn ClassView>,
    pub field_numbers: HashMap<FieldName, (FieldNumber, CPDType)>,
    pub field_numbers_reverse: HashMap<FieldNumber, (FieldName, CPDType)>,
    pub method_numbers: HashMap<MethodShape, MethodNumber>,
    pub method_numbers_reverse: HashMap<MethodNumber, MethodShape>,
    pub recursive_num_fields: usize,
    pub static_var_types: HashMap<FieldName, CPDType>,
    pub static_vars: RwLock<HashMap<FieldName, NativeJavaValue<'gc>>>,
    pub parent: Option<Arc<RuntimeClass<'gc>>>,
    pub interfaces: Vec<Arc<RuntimeClass<'gc>>>,
    //class may not be prepared
    pub status: RwLock<ClassStatus>,
}

//todo refactor to make it impossible to create RuntimeClassClass without registering to array, box leak jvm state to static

impl<'gc> RuntimeClassClass<'gc> {
    pub fn new(class_view: Arc<dyn ClassView>, field_numbers: HashMap<FieldName, (FieldNumber, CPDType)>, method_numbers: HashMap<MethodShape, MethodNumber>, recursive_num_fields: usize, static_vars: RwLock<HashMap<FieldName, NativeJavaValue<'gc>>>, parent: Option<Arc<RuntimeClass<'gc>>>, interfaces: Vec<Arc<RuntimeClass<'gc>>>, status: RwLock<ClassStatus>, static_var_types: HashMap<FieldName, CPDType>) -> Self {
        let field_numbers_reverse = field_numbers.iter()
            .map(|(field_name, (field_number, cpd_type))| (*field_number, (*field_name, cpd_type.clone())))
            .collect();

        let method_numbers_reverse = method_numbers.iter()
            .map(|(method_shape, method_number)| (method_number.clone(), method_shape.clone()))
            .collect();
        Self {
            class_view,
            field_numbers,
            field_numbers_reverse,
            method_numbers,
            method_numbers_reverse,
            recursive_num_fields,
            static_var_types,
            static_vars,
            parent,
            interfaces,
            status,
        }
    }

    pub fn num_vars(&self) -> usize {
        self.class_view.fields().filter(|field| !field.is_static()).count() + self.parent.as_ref().map(|parent| parent.unwrap_class_class().num_vars()).unwrap_or(0)
    }

    pub fn num_virtual_methods(&self) -> usize {
        self.class_view.methods().filter(|method| !method.is_static()).count() + self.parent.as_ref().map(|parent| parent.unwrap_class_class().num_virtual_methods()).unwrap_or(0)
    }
}

impl<'gc> Debug for RuntimeClassClass<'gc> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}:{:?}", self.class_view.name(), todo!()/*self.static_vars*/)
    }
}

pub fn prepare_class<'vm_life, 'l, 'k>(jvm: &'vm_life JVMState<'vm_life>, int_state: &'_ mut InterpreterStateGuard<'vm_life, 'l>, classfile: Arc<dyn ClassView>, res: &mut StaticVarGuard<'vm_life, 'k>) {
    if let Some(jvmti) = jvm.jvmti_state() {
        if let CPDType::Ref(ref_) = classfile.type_() {
            if let CPRefType::Class(cn) = ref_ {
                jvmti.built_in_jdwp.class_prepare(jvm, &cn, int_state)
            }
        }
    }

    for field in classfile.fields() {
        if field.is_static() {
            let val = default_value(&field.field_type());
            res.set(field.field_name(), val);
        }
    }
}

impl<'gc> std::convert::From<RuntimeClassClass<'gc>> for RuntimeClass<'gc> {
    fn from(rcc: RuntimeClassClass<'gc>) -> Self {
        Self::Object(rcc)
    }
}

pub fn initialize_class<'gc, 'l>(runtime_class: Arc<RuntimeClass<'gc>>, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<Arc<RuntimeClass<'gc>>, WasException> {
    // assert!(int_state.throw().is_none());
    //todo make sure all superclasses are iniited first
    //todo make sure all interfaces are initted first
    //todo create a extract string which takes index. same for classname
    {
        let view = &runtime_class.view();
        for field in view.fields() {
            if field.is_static() && field.is_final() {
                //todo do I do this for non-static? Should I?
                let constant_info_view = match field.constant_value_attribute() {
                    None => continue,
                    Some(i) => i,
                };
                let constant_value = from_constant_pool_entry(&constant_info_view, jvm, int_state);
                let name = field.field_name();
                runtime_class.static_vars(jvm).set(name, constant_value);
            }
        }
    }
    //todo detecting if assertions are enabled?
    let view = &runtime_class.view();
    let lookup_res = view.lookup_method_name(MethodName::constructor_clinit()); // todo constant for clinit
    assert!(lookup_res.len() <= 1);
    let clinit = match lookup_res.get(0) {
        None => return Ok(runtime_class),
        Some(x) => x,
    };
    //todo should I really be manipulating the interpreter state like this

    let mut locals = vec![];
    let locals_n = clinit.code_attribute().unwrap().max_locals;
    for _ in 0..locals_n {
        locals.push(NewJavaValue::Top);
    }

    let method_i = clinit.method_i() as u16;
    let method_id = jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method_i);
    jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolver { jvm, loader: int_state.current_loader(jvm) }, method_id);


    let new_stack = StackEntryPush::new_java_frame(jvm, runtime_class.clone(), method_i, locals);

    //todo these java frames may have to be converted to native?
    let mut new_function_frame = int_state.push_frame(new_stack);
    return match run_function(jvm, int_state, &mut new_function_frame) {
        Ok(res) => {
            assert!(res.is_none());
            int_state.pop_frame(jvm, new_function_frame, true);
            if !jvm.config.compiled_mode_active {}
            // if int_state.function_return() {
            //     int_state.set_function_return(false);
            Ok(runtime_class)
            // }
            // panic!()
        }
        Err(WasException {}) => {
            int_state.pop_frame(jvm, new_function_frame, false);
            // dbg!(JavaValue::Object(todo!()/*interpreter_state.throw().clone()*/).cast_object().to_string(jvm, interpreter_state).unwrap().unwrap().to_rust_string(jvm));
            int_state.debug_print_stack_trace(jvm);
            Err(WasException)
        }
    };
}