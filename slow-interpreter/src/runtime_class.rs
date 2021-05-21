use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use classfile_view::view::{ArrayView, ClassView, HasAccessFlags, PrimitiveView};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::instructions::ldc::from_constant_pool_entry;
use crate::interpreter::{run_function, WasException};
use crate::java_values::{default_value, JavaValue};
use crate::jvm_state::ClassStatus;

#[derive(Debug)]
pub enum RuntimeClass {
    Byte,
    Boolean,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Void,
    Array(RuntimeClassArray),
    Object(RuntimeClassClass),
    Top,
}

#[derive(Debug)]
pub struct RuntimeClassArray {
    pub sub_class: Arc<RuntimeClass>,
}


pub struct RuntimeClassClass {
    pub class_view: Arc<dyn ClassView>,
    pub field_numbers: HashMap<String, usize>,
    pub static_vars: RwLock<HashMap<String, JavaValue>>,
    pub parent: Option<Arc<RuntimeClass>>,
    pub interfaces: Vec<Arc<RuntimeClass>>,
    //class may not be prepared
    pub status: RwLock<ClassStatus>,
}

impl RuntimeClass {
    pub fn ptypeview(&self) -> PTypeView {
        match self {
            RuntimeClass::Byte => PTypeView::ByteType,
            RuntimeClass::Boolean => PTypeView::BooleanType,
            RuntimeClass::Short => PTypeView::ShortType,
            RuntimeClass::Char => PTypeView::CharType,
            RuntimeClass::Int => PTypeView::IntType,
            RuntimeClass::Long => PTypeView::LongType,
            RuntimeClass::Float => PTypeView::FloatType,
            RuntimeClass::Double => PTypeView::DoubleType,
            RuntimeClass::Void => PTypeView::VoidType,
            RuntimeClass::Array(arr) => {
                PTypeView::Ref(ReferenceTypeView::Array(box arr.sub_class.ptypeview()))
            }
            RuntimeClass::Object(o) => {
                PTypeView::Ref(ReferenceTypeView::Class(o.class_view.name().unwrap_name()))
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

    pub fn static_vars(&self) -> RwLockWriteGuard<'_, HashMap<String, JavaValue>> {
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

    pub fn unwrap_class_class(&self) -> &RuntimeClassClass {
        match self {
            RuntimeClass::Object(classclass) => classclass,
            _ => panic!()
        }
    }
}

impl Debug for RuntimeClassClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}:{:?}", self.class_view.name(), self.static_vars)
    }
}

impl RuntimeClassClass {
    pub fn num_vars(&self) -> usize {
        self.class_view.fields().filter(|field| !field.is_static()).count() + self.parent.as_ref().map(|parent| parent.unwrap_class_class().num_vars()).unwrap_or(0)
    }
}

pub fn prepare_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, classfile: Arc<dyn ClassView>, res: &mut HashMap<String, JavaValue>) {
    if let Some(jvmti) = jvm.jvmti_state.as_ref() {
        if let PTypeView::Ref(ref_) = classfile.type_() {
            if let ReferenceTypeView::Class(cn) = ref_ {
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


impl std::convert::From<RuntimeClassClass> for RuntimeClass {
    fn from(rcc: RuntimeClassClass) -> Self {
        Self::Object(rcc)
    }
}

pub fn initialize_class(
    runtime_class: Arc<RuntimeClass>,
    jvm: &JVMState,
    interpreter_state: &mut InterpreterStateGuard,
) -> Result<Arc<RuntimeClass>, WasException> {
    assert!(interpreter_state.throw().is_none());
    //todo make sure all superclasses are iniited first
    //todo make sure all interfaces are initted first
    //todo create a extract string which takes index. same for classname
    {
        let view = &runtime_class.view();
        // dbg!(view.name());
        for field in view.fields() {
            if field.is_static() && field.is_final() {
                //todo do I do this for non-static? Should I?
                let constant_info_view = match field.constant_value_attribute() {
                    None => continue,
                    Some(i) => i,
                };
                let constant_value = from_constant_pool_entry(&constant_info_view, jvm, interpreter_state);
                let name = field.field_name();
                runtime_class.static_vars().insert(name, constant_value);
            }
        }
    }
    //todo detecting if assertions are enabled?
    let view = &runtime_class.view();
    let lookup_res = view.lookup_method_name(&"<clinit>".to_string());
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
    let new_function_frame = interpreter_state.push_frame(new_stack);
    match run_function(jvm, interpreter_state) {
        Ok(()) => {
            interpreter_state.pop_frame(jvm, new_function_frame, true);
            let function_return = interpreter_state.function_return_mut();
            if *function_return {
                *function_return = false;
                return Ok(runtime_class);
            }
            panic!()
        }
        Err(WasException {}) => {
            interpreter_state.pop_frame(jvm, new_function_frame, false);
            dbg!(JavaValue::Object(interpreter_state.throw().clone()).cast_object().to_string(jvm, interpreter_state).unwrap().unwrap().to_rust_string());
            interpreter_state.debug_print_stack_trace();
            return Err(WasException);
        }
    };
}

