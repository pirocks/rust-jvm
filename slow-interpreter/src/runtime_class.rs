use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use classfile_view::loading::LoaderArc;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::parse_field_descriptor;
use rust_jvm_common::classfile::{ACC_STATIC, Classfile};

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::instructions::ldc::from_constant_pool_entry;
use crate::interpreter::run_function;
use crate::java_values::{default_value, JavaValue};

#[derive(Debug, PartialEq, Hash)]
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

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct RuntimeClassArray {
    pub sub_class: Arc<RuntimeClass>
}


pub struct RuntimeClassClass {
    classfile: Arc<Classfile>,
    class_view: Arc<ClassView>,
    loader: LoaderArc,
    static_vars: RwLock<HashMap<String, JavaValue>>,
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

    pub fn loader(&self, jvm: &JVMState) -> LoaderArc {
        match self {
            RuntimeClass::Byte => jvm.bootstrap_loader.clone(),
            RuntimeClass::Boolean => jvm.bootstrap_loader.clone(),
            RuntimeClass::Short => jvm.bootstrap_loader.clone(),
            RuntimeClass::Char => jvm.bootstrap_loader.clone(),
            RuntimeClass::Int => jvm.bootstrap_loader.clone(),
            RuntimeClass::Long => jvm.bootstrap_loader.clone(),
            RuntimeClass::Float => jvm.bootstrap_loader.clone(),
            RuntimeClass::Double => jvm.bootstrap_loader.clone(),
            RuntimeClass::Void => jvm.bootstrap_loader.clone(),
            RuntimeClass::Array(a) => a.sub_class.loader(jvm),//todo technically this is wrong
            RuntimeClass::Object(o) => o.loader.clone(),
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
}

impl Debug for RuntimeClassClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}:{:?}", self.class_view.name(), self.static_vars)
    }
}

impl Hash for RuntimeClassClass {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.classfile.hash(state);
        self.loader.name().to_string().hash(state)
    }
}

impl PartialEq for RuntimeClassClass {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.loader, &other.loader) && self.classfile == other.classfile && *self.static_vars.read().unwrap() == *other.static_vars.read().unwrap()
    }
}

impl Eq for RuntimeClass {}

pub fn prepare_class(_jvm: &JVMState, classfile: Arc<Classfile>, loader: LoaderArc) -> RuntimeClass {
    let mut res = HashMap::new();
    for field in &classfile.fields {
        if (field.access_flags & ACC_STATIC) > 0 {
            let name = classfile.constant_pool[field.name_index as usize].extract_string_from_utf8();
            let field_descriptor_string = classfile.constant_pool[field.descriptor_index as usize].extract_string_from_utf8();
            let parsed = parse_field_descriptor(field_descriptor_string.as_str()).unwrap();//todo we should really have two pass parsing
            let val = default_value(PTypeView::from_ptype(&parsed.field_type));
            res.insert(name, val);
        }
    }
    let res = RuntimeClassClass {
        class_view: Arc::new(ClassView::from(classfile.clone())),
        classfile,
        loader,
        static_vars: RwLock::new(res),
    }.into();
    res
}


impl std::convert::From<RuntimeClassClass> for RuntimeClass {
    fn from(rcc: RuntimeClassClass) -> Self {
        Self::Object(rcc)
    }
}

pub fn initialize_class<'l>(
    runtime_class: Arc<RuntimeClass>,
    jvm: &'static JVMState,
    interpreter_state: &mut InterpreterStateGuard,
) -> Arc<RuntimeClass> {
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
                let constant_value = from_constant_pool_entry(&constant_info_view, jvm, interpreter_state);
                let name = field.field_name();
                runtime_class.static_vars().insert(name, constant_value);
            }
        }
    }
    //todo detecting if assertions are enabled?
    let class_arc = runtime_class;
    let view = &class_arc.view();
    let lookup_res = view.lookup_method_name(&"<clinit>".to_string());
    assert!(lookup_res.len() <= 1);
    let clinit = match lookup_res.iter().nth(0) {
        None => return class_arc,
        Some(x) => x,
    };
    //todo should I really be manipulating the interpreter state like this

    let mut locals = vec![];
    let locals_n = clinit.code_attribute().unwrap().max_locals;
    for _ in 0..locals_n {
        locals.push(JavaValue::Top);
    }

    let new_stack = StackEntry {
        class_pointer: class_arc.clone(),
        method_i: Option::from(clinit.method_i() as u16),
        local_vars: locals,
        operand_stack: vec![],
        pc: 0,
        pc_offset: 0,
    };
    interpreter_state.push_frame(new_stack);
    run_function(jvm, interpreter_state);
    interpreter_state.pop_frame();
    if interpreter_state.throw().is_some() || *interpreter_state.terminate() {
        interpreter_state.print_stack_trace();
        unimplemented!()
        //need to clear status after
    }
    let function_return = interpreter_state.function_return_mut();
    if *function_return {
        *function_return = false;
        return class_arc;
    }
    panic!()
}

