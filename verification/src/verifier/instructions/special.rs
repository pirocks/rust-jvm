use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::instructions::InstructionIsTypeSafeResult;
use crate::verifier::TypeSafetyError;
use crate::verifier::get_class;
use rust_jvm_common::classfile::ConstantKind;
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::classnames::NameReference;
use std::sync::Arc;
use crate::types::parse_field_descriptor;
use crate::types::FieldDescriptor;
use crate::verifier::codecorrectness::can_pop;
use crate::verifier::passes_protected_check;
use rust_jvm_common::unified_types::UnifiedType;
use rust_jvm_common::unified_types::ClassWithLoader;
use crate::verifier::instructions::exception_stack_frame;
use crate::verifier::instructions::ResultFrames;
use rust_jvm_common::classnames::get_referred_name;
use crate::verifier::codecorrectness::stackmapframes::copy_recurse;


//#[allow(unused)]
//fn instruction_is_type_safe_instanceof(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//

//#[allow(unused)]
//fn instruction_is_type_safe_getfield(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_getstatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//

//#[allow(unused)]
//fn instruction_is_type_safe_tableswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}


//
//#[allow(unused)]
//fn instruction_is_type_safe_anewarray(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_arraylength(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_athrow(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_checkcast(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//

pub fn instruction_is_type_safe_putfield(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    match instruction_is_type_safe_putfield_first_case(cp, env, offset, stack_frame) {
        Ok(res) => Result::Ok(res),
        Err(_) => instruction_is_type_safe_putfield_second_case(cp, env, offset, stack_frame),
    }
}

fn instruction_is_type_safe_putfield_second_case(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    //todo duplication
    let (field_class_name, _field_name, field_descriptor) = extract_field_descriptor(cp, env);
    let field_type = field_descriptor.field_type;
    if env.method.prolog_class.class_name != field_class_name {
        return Result::Err(TypeSafetyError::NotSafe(format!("todo message:{}:{}", file!(), line!())));
    }
    //todo is this equivalent to
    if get_referred_name(&env.method.prolog_class.class_name) != "<init>" {
        //todo need a macro for this
        return Result::Err(TypeSafetyError::NotSafe(format!("todo message:{}:{}", file!(), line!())));
    }
    let next_stack_frame = can_pop(stack_frame, vec![field_type, UnifiedType::UninitializedThis])?;
    let exception_frame = exception_stack_frame(&stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame, next_frame: next_stack_frame }))
}

fn instruction_is_type_safe_putfield_first_case(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let (field_class_name, field_name, field_descriptor) = extract_field_descriptor(cp, env);
    let field_type = field_descriptor.field_type;
    let _popped_frame = can_pop(stack_frame, vec![copy_recurse(&field_type)])?;
    passes_protected_check(env, get_referred_name(&field_class_name), field_name,/* &field_descriptor, */stack_frame)?;
    let current_loader = env.class_loader.clone();
    let next_stack_frame = can_pop(stack_frame, vec![field_type, UnifiedType::Class(ClassWithLoader { loader: current_loader, class_name: field_class_name })])?;
    let exception_frame = exception_stack_frame(&stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame, next_frame: next_stack_frame }))
}


fn extract_field_descriptor(cp: usize, env: &Environment) -> (ClassName, String, FieldDescriptor) {
    let current_class = get_class(env.method.prolog_class);
    let field_entry: &ConstantKind = &current_class.constant_pool[cp].kind;
    let (class_index, name_and_type_index) = match field_entry {
        ConstantKind::Fieldref(f) => {
            (f.class_index, f.name_and_type_index)
        }
        _ => panic!()
    };
    let field_class_name = match &current_class.constant_pool[class_index as usize].kind {
        ConstantKind::Class(c) => {
            ClassName::Ref(NameReference { class_file: Arc::downgrade(&current_class), index: c.name_index })
        }
        _ => panic!()
    };
    let (field_name_index, descriptor_index) = match &current_class.constant_pool[name_and_type_index as usize].kind {
        ConstantKind::NameAndType(nt) => {
            (nt.name_index, nt.descriptor_index)
        }
        _ => panic!()
    };
    let field_name = extract_string_from_utf8(&current_class.constant_pool[field_name_index as usize]);
    let descriptor_string = extract_string_from_utf8(&current_class.constant_pool[descriptor_index as usize]);
    let field_descriptor = parse_field_descriptor(&env.class_loader, descriptor_string.as_ref()).unwrap();
    (field_class_name, field_name, field_descriptor)
}

//#[allow(unused)]
//fn instruction_is_type_safe_putstatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//

//#[allow(unused)]
//fn instruction_is_type_safe_monitorenter(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_multianewarray(cp: usize, dim: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
////todo start using CPIndex instead of usize
//
//#[allow(unused)]
//fn instruction_is_type_safe_new(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_newarray(type_code: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}

//
//#[allow(unused)]
//fn instruction_is_type_safe_lookupswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//