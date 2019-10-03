use verification::prolog_info_defs::get_access_flags;
use classfile::ACC_NATIVE;
use classfile::ACC_ABSTRACT;
use verification::code_verification::ParseCodeAttribute;
use std::prelude::v1::Vec;
use classfile::code::Instruction;
use verification::code_verification::StackMap;

pub fn is_bootstrap_loader(loader: &String) -> bool {
    return loader == &"bl".to_string();//todo  what if someone defines a Loader class called bl
}

pub fn get_class_methods(class: PrologClass) -> Vec<PrologClassMethod> {
    unimplemented!();
}

pub fn class_is_type_safe(class: &PrologClass) -> bool {
    if class.name == "java/lang/Object" {
        if !is_bootstrap_loader(class.loader) {
            return false;
        }
    } else {
        //class must have a superclass or be 'java/lang/Object'
        unimplemented!();
    }
    let mut method = get_class_methods(class);
    method.iter().all(|m| {
        method_is_type_safe(class, m)
    })
}

pub fn does_not_override_final_method(class: &PrologClass, method: &PrologClassMethod) -> bool {}

pub fn method_is_type_safe(class: &PrologClass, method: &PrologClassMethod) -> bool {
    let access_flags = get_access_flags(class, method);
    return does_not_override_final_method(class, method) &&
        if access_flags & ACC_NATIVE {
            true
        } else if access_flags & ACC_ABSTRACT {
            true
        } else {
            let attributes = get_attributes(class, method);
            attributes.iter().any(|_| {
                unimplemented!()
            }) && method_with_code_is_type_safe(class, method)
        };
}

struct Environment<'l> {
    class: PrologClass,
    method: PrologClassMethod,
    frame_size: u16,
    max_stack: u16,
    merged_code: Vec<MergedCodeInstruction<'l>>,

}

enum MergedCodeInstruction<'l> {
    Instruction(&'l Instruction),
    StackMap(&'l StackMap),
}

/**
assumes that stackmaps and instructions are ordered
*/
fn merge_stack_map_and_code<'l>(instruction: Vec<Instruction>, stack_maps: Vec<StackMap>) -> Vec<MergedCodeInstruction<'l>> {
    let mut res = vec![];

    loop {
        let (instruction, instruction_offset) = match instruction.first() {
            None => { (None, -1) },//todo hacky
            Some(i) => { (Some(i), i.offset as i32) },
        };
        let (stack_map, stack_map_offset) = match stack_maps.first() {
            None => { (None, -1) },
            Some(s) => { (Some(s), s.offset as i32) },
        };
        if stack_map_offset >= instruction_offset {
            res.push(MergedCodeInstruction::StackMap(stack_map.unwrap()))//todo
        } else {
            let instr = match instruction {
                None => { break },
                Some(i) => { i },
            };
            res.push(MergedCodeInstruction::Instruction(instr))//todo
        }
    }
    return res;
}

pub fn method_with_code_is_type_safe(class: &PrologClass, method: &PrologClassMethod) -> bool {
    let parsed_code: ParseCodeAttribute = get_parsed_code_attribute(class, method);
    let frame_size = parsed_code.frame_size;
    let max_stack = parsed_code.max_stack;
    let code = parsed_code.code;
    let handlers = parsed_code.exception_table;
    let stack_map = parsed_code.stackmap_frames;
    let merged = merge_stack_map_and_code(code, stack_map);
    let initial_stack_frame = method_initial_stack_frame(class,method,frame_size,unimplemented!(),unimplemented!())
}