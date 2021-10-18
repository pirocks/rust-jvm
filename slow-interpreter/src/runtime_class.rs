use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use classfile_view::view::{ArrayView, ClassView, HasAccessFlags, PrimitiveView};
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{FieldName, MethodName};

use crate::instructions::ldc::from_constant_pool_entry;
use crate::interpreter::{run_function, WasException};
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::{default_value, JavaValue};
use crate::jvm_state::{ClassStatus, JVMState};
use crate::stack_entry::StackEntry;

#[derive(Debug)]
pub enum RuntimeClass<'gc_life> {
    Byte,
    Boolean,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Void,
    Array(RuntimeClassArray<'gc_life>),
    Object(RuntimeClassClass<'gc_life>),
    Top,
}

impl<'gc_life> RuntimeClass<'gc_life> {
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
            RuntimeClass::Array(arr) => {
                CPDType::Ref(CPRefType::Array(box arr.sub_class.cpdtype()))
            }
            RuntimeClass::Object(o) => {
                CPDType::Ref(CPRefType::Class(o.class_view.name().unwrap_name()))
            }
            RuntimeClass::Top => panic!()
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
            RuntimeClass::Array(arr) => {
                Arc::new(ArrayView {
                    sub: arr.sub_class.view()
                })
            }
            RuntimeClass::Object(o) => o.class_view.clone(),
            RuntimeClass::Top => panic!()
        }
    }

    pub fn static_vars(&self) -> RwLockWriteGuard<'_, HashMap<FieldName, JavaValue<'gc_life>>> {
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
            RuntimeClass::Object(o) => o.static_vars.write().unwrap(),
            RuntimeClass::Top => panic!()
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
            RuntimeClass::Top => panic!()
        }
    }

    pub fn set_status(&self, status: ClassStatus) {
        match self {
            RuntimeClass::Array(a) => a.sub_class.set_status(status),
            RuntimeClass::Object(o) => *o.status.write().unwrap() = status,
            _ => {}
        }
    }

    pub fn unwrap_class_class(&self) -> &RuntimeClassClass<'gc_life> {
        self.try_unwrap_class_class().unwrap()
    }

    pub fn try_unwrap_class_class(&self) -> Option<&RuntimeClassClass<'gc_life>> {
        match self {
            RuntimeClass::Object(classclass) => Some(classclass),
            _ => None
        }
    }
}


#[derive(Debug)]
pub struct RuntimeClassArray<'gc_life> {
    pub sub_class: Arc<RuntimeClass<'gc_life>>,
}

pub struct RuntimeClassClass<'gc_life> {
    pub class_view: Arc<dyn ClassView>,
    pub field_numbers: HashMap<FieldName, (usize, CPDType)>,
    pub recursive_num_fields: usize,
    pub static_vars: RwLock<HashMap<FieldName, JavaValue<'gc_life>>>,
    pub parent: Option<Arc<RuntimeClass<'gc_life>>>,
    pub interfaces: Vec<Arc<RuntimeClass<'gc_life>>>,
    //class may not be prepared
    pub status: RwLock<ClassStatus>,
}

//todo refactor to make it impossible to create RuntimeClassClass without registering to array, box leak jvm state to static 

impl<'gc_life> RuntimeClassClass<'gc_life> {
    pub fn new(class_view: Arc<dyn ClassView>,
               field_numbers: HashMap<FieldName, (usize, CPDType)>,
               recursive_num_fields: usize,
               static_vars: RwLock<HashMap<FieldName, JavaValue<'gc_life>>>,
               parent: Option<Arc<RuntimeClass<'gc_life>>>,
               interfaces: Vec<Arc<RuntimeClass<'gc_life>>>,
               status: RwLock<ClassStatus>) -> Self {
        Self {
            class_view,
            field_numbers,
            recursive_num_fields,
            static_vars,
            parent,
            interfaces,
            status,
        }
    }

    pub fn num_vars(&self) -> usize {
        self.class_view.fields().filter(|field| !field.is_static()).count() + self.parent.as_ref().map(|parent| parent.unwrap_class_class().num_vars()).unwrap_or(0)
    }
}

impl<'gc_life> Debug for RuntimeClassClass<'gc_life> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}:{:?}", self.class_view.name(), self.static_vars)
    }
}

pub fn prepare_class<'vm_life>(jvm: &'vm_life JVMState<'vm_life>, int_state: &'_ mut InterpreterStateGuard<'vm_life, '_>, classfile: Arc<dyn ClassView>, res: &mut HashMap<FieldName, JavaValue<'vm_life>>) {
    if let Some(jvmti) = jvm.jvmti_state() {
        if let CPDType::Ref(ref_) = classfile.type_() {
            if let CPRefType::Class(cn) = ref_ {
                jvmti.built_in_jdwp.class_prepare(jvm, &cn, int_state)
            }
        }
    }

    for field in classfile.fields() {
        if field.is_static() {
            let val = default_value(field.field_type());
            res.insert(field.field_name(), val);
        }
    }
}


impl<'gc_life> std::convert::From<RuntimeClassClass<'gc_life>> for RuntimeClass<'gc_life> {
    fn from(rcc: RuntimeClassClass<'gc_life>) -> Self {
        Self::Object(rcc)
    }
}

pub fn initialize_class(
    runtime_class: Arc<RuntimeClass<'gc_life>>,
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
    assert!(int_state.throw().is_none());
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
                runtime_class.static_vars().insert(name, constant_value);
            }
        }
    }
    //todo detecting if assertions are enabled?
    let view = &runtime_class.view();
    let lookup_res = view.lookup_method_name(MethodName::constructor_clinit());// todo constant for clinit
    assert!(lookup_res.len() <= 1);
    let clinit = match lookup_res.get(0) {
        None => return Ok(runtime_class),
        Some(x) => x,
    };
    //todo should I really be manipulating the interpreter state like this

    let mut locals = vec![];
    let locals_n = clinit.code_attribute().unwrap().max_locals;
    for _ in 0..locals_n {
        locals.push(JavaValue::Top);
    }

    let new_stack = StackEntry::new_java_frame(jvm, runtime_class.clone(), clinit.method_i() as u16, locals);
    //todo these java frames may have to be converted to native?
    let new_function_frame = int_state.push_frame(new_stack, jvm);
    match run_function(jvm, int_state) {
        Ok(()) => {
            if !jvm.config.compiled_mode_active {
                int_state.pop_frame(jvm, new_function_frame, true);
            }
            if int_state.function_return() {
                int_state.set_function_return(false);
                return Ok(runtime_class);
            }
            panic!()
        }
        Err(WasException {}) => {
            int_state.pop_frame(jvm, new_function_frame, false);
            // dbg!(JavaValue::Object(todo!()/*interpreter_state.throw().clone()*/).cast_object().to_string(jvm, interpreter_state).unwrap().unwrap().to_rust_string(jvm));
            int_state.debug_print_stack_trace(jvm);
            return Err(WasException);
        }
    };
}

