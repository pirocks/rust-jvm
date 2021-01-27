use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use classfile_view::loading::LoaderName;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::parse_field_descriptor;
use rust_jvm_common::classfile::{ACC_STATIC, Classfile};

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::instructions::ldc::from_constant_pool_entry;
use crate::interpreter::run_function;
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
}

#[derive(Debug)]
pub struct RuntimeClassArray {
    pub sub_class: Arc<RuntimeClass>,
}


pub struct RuntimeClassClass {
    pub(crate) class_view: Arc<ClassView>,
    pub(crate) static_vars: RwLock<HashMap<String, JavaValue>>,
    //class may not be prepared
    pub(crate) status: RwLock<ClassStatus>,
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
                PTypeView::Ref(ReferenceTypeView::Class(o.class_view.name()))
            }
        }
    }
    pub fn view(&self) -> &Arc<ClassView> {
        match self {
            RuntimeClass::Byte => unimplemented!(),
            RuntimeClass::Boolean => unimplemented!(),
            RuntimeClass::Short => unimplemented!(),
            RuntimeClass::Char => unimplemented!(),
            RuntimeClass::Int => unimplemented!(),
            RuntimeClass::Long => unimplemented!(),
            RuntimeClass::Float => unimplemented!(),
            RuntimeClass::Double => unimplemented!(),
            RuntimeClass::Void => unimplemented!(),
            RuntimeClass::Array(_) => unimplemented!(),
            RuntimeClass::Object(o) => &o.class_view,
        }
    }

    pub fn try_view(&self) -> Option<&Arc<ClassView>> {
        match self {
            RuntimeClass::Byte => None,
            RuntimeClass::Boolean => None,
            RuntimeClass::Short => None,
            RuntimeClass::Char => None,
            RuntimeClass::Int => None,
            RuntimeClass::Long => None,
            RuntimeClass::Float => None,
            RuntimeClass::Double => None,
            RuntimeClass::Void => None,
            RuntimeClass::Array(_) => None,
            RuntimeClass::Object(o) => (&o.class_view).into(),
        }
    }

    pub fn static_vars(&self) -> RwLockWriteGuard<'_, HashMap<String, JavaValue>> {
        match self {
            RuntimeClass::Byte => unimplemented!(),
            RuntimeClass::Boolean => unimplemented!(),
            RuntimeClass::Short => unimplemented!(),
            RuntimeClass::Char => unimplemented!(),
            RuntimeClass::Int => unimplemented!(),
            RuntimeClass::Long => unimplemented!(),
            RuntimeClass::Float => unimplemented!(),
            RuntimeClass::Double => unimplemented!(),
            RuntimeClass::Void => unimplemented!(),
            RuntimeClass::Array(_) => unimplemented!(),
            RuntimeClass::Object(o) => o.static_vars.write().unwrap(),
        }
    }


    pub fn status(&self) -> ClassStatus {
        match self {
            RuntimeClass::Byte => todo!(),
            RuntimeClass::Boolean => todo!(),
            RuntimeClass::Short => todo!(),
            RuntimeClass::Char => todo!(),
            RuntimeClass::Int => todo!(),
            RuntimeClass::Long => todo!(),
            RuntimeClass::Float => todo!(),
            RuntimeClass::Double => todo!(),
            RuntimeClass::Void => todo!(),
            RuntimeClass::Array(a) => a.sub_class.status(),
            RuntimeClass::Object(o) => *o.status.read().unwrap()
        }
    }

    pub fn set_status(&self, status: ClassStatus) {
        match self {
            RuntimeClass::Byte => todo!(),
            RuntimeClass::Boolean => todo!(),
            RuntimeClass::Short => todo!(),
            RuntimeClass::Char => todo!(),
            RuntimeClass::Int => todo!(),
            RuntimeClass::Long => todo!(),
            RuntimeClass::Float => todo!(),
            RuntimeClass::Double => todo!(),
            RuntimeClass::Void => todo!(),
            RuntimeClass::Array(a) => a.sub_class.set_status(status),
            RuntimeClass::Object(o) => *o.status.write().unwrap() = status,
        }
    }
}

impl Debug for RuntimeClassClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}:{:?}", self.class_view.name(), self.static_vars)
    }
}

// impl Hash for RuntimeClassClass {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.classfile.hash(state);
//
//     }
// }
//
// impl PartialEq for RuntimeClassClass {
//     fn eq(&self, other: &Self) -> bool {
//         self.loader == other.loader &&
//             self.classfile == other.classfile && *self.static_vars.read().unwrap() == *other.static_vars.read().unwrap()
//     }
// }

// impl Eq for RuntimeClass {}

pub fn prepare_class(_jvm: &JVMState, classfile: Arc<Classfile>, res: &mut HashMap<String, JavaValue>) {
    for field in &classfile.fields {
        if (field.access_flags & ACC_STATIC) > 0 {
            let name = classfile.constant_pool[field.name_index as usize].extract_string_from_utf8();
            let field_descriptor_string = classfile.constant_pool[field.descriptor_index as usize].extract_string_from_utf8();
            let parsed = parse_field_descriptor(field_descriptor_string.as_str()).unwrap();//todo we should really have two pass parsing
            let val = default_value(PTypeView::from_ptype(&parsed.field_type));
            res.insert(name, val);
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
) -> Option<Arc<RuntimeClass>> {
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
        None => return runtime_class.into(),
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
    run_function(jvm, interpreter_state);
    interpreter_state.pop_frame(new_function_frame);
    if interpreter_state.throw().is_some() || *interpreter_state.terminate() {
        interpreter_state.print_stack_trace();
        return None;
    }
    let function_return = interpreter_state.function_return_mut();
    if *function_return {
        *function_return = false;
        return runtime_class.into();
    }
    panic!()
}

