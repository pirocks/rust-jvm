use std::collections::HashMap;

use classfile::{Classfile, code_attribute, MethodInfo};
use interpreter::{do_instruction, InterpreterState};
use verification::prolog_info_defs::extract_string_from_utf8;

pub enum JavaValue {
    Long(i64),
    Int(i32),
    Char(u16),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Reference(Box<ClassInstance>),
    ArrayReference(Box<ClassInstance>),
}

pub struct ClassInstance {
    pub loaded_classfile: Box<LoadedClassFile>,
    pub fields: HashMap<String, JavaValue>,
}

pub struct LoadedClassFile {
    pub classfile: Classfile,
    pub static_fields: HashMap<String, JavaValue>,
}

pub fn init_locals_static_no_args(class_file:&Classfile, method: &MethodInfo) -> Vec<u32>{
//    let code = code_attribute(method).expect("Error finding code in method");
//    let descriptor_str = extract_string_from_utf8(&class_file.constant_pool[method_info.descriptor_index as usize]);
//    let parsed_descriptor = parse_method_descriptor(descriptor_str.as_str()).expect("Error parsing method descriptor");

//    let this_pointer = if method_info.access_flags & ACC_STATIC > 0{
//        None
//    }else {
//        Some(Type::ReferenceType(Reference {class_name:class_name(class_file) }))
//    };

//    let frame: Frame = init_frame(parsed_descriptor.parameter_types, this_pointer, code.max_locals);

    let res = Vec::new();

////    match this_pointer{
////        None => {},
////        Some(this) => {
////            res.push()
////        },
////    }
//
//    for local in frame.locals.iter(){
//
//    }

    res
}

pub fn run_static_method_no_args(classfile: &Classfile, method: &MethodInfo) {
    dbg!("{}",extract_string_from_utf8(&classfile.constant_pool[method.name_index as usize]));
    let code = code_attribute(method).expect("Error finding code in method");
    let local_vars = init_locals_static_no_args(classfile,method);

    let mut interpreter_state = InterpreterState {
        local_vars,
        operand_stack: vec![],
        pc: 0,
        pc_offset: 0,
        terminate: false,
//        current_class: Box::new(classfile)
    };

    while !interpreter_state.terminate {
        do_instruction(code.code_raw.as_slice(), &mut interpreter_state, panic!());
    }
}