use crate::verifier::codecorrectness::{Environment, valid_type_transition, translate_types_to_vm_types};
use crate::verifier::Frame;
use crate::verifier::instructions::{InstructionTypeSafe, AfterGotoFrames};
use crate::verifier::TypeSafetyError;
use crate::verifier::get_class;
use rust_jvm_common::classfile::{ConstantKind, UninitializedVariableInfo};
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::classnames::NameReference;
use std::sync::Arc;
use crate::verifier::codecorrectness::can_pop;
use crate::verifier::passes_protected_check;
use rust_jvm_common::unified_types::UnifiedType;
use rust_jvm_common::unified_types::ClassWithLoader;
use crate::verifier::instructions::exception_stack_frame;
use crate::verifier::instructions::ResultFrames;
use rust_jvm_common::classnames::get_referred_name;
use rust_jvm_common::classfile::CPIndex;
use crate::verifier::instructions::branches::{substitute, possibly_array_to_type};
use crate::OperandStack;
use classfile_parser::types::Descriptor;
use classfile_parser::types::FieldDescriptor;
use classfile_parser::types::parse_field_descriptor;
use rust_jvm_common::unified_types::ArrayType;

pub fn instruction_is_type_safe_instanceof(_cp: CPIndex, env: &Environment, _offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    let type_ = extract_constant_pool_entry_as_type(cp,env);//todo verify that cp is valid
    let bl = &env.vf.bootstrap_loader.clone();
    let object = UnifiedType::Class(ClassWithLoader { class_name: ClassName::Str("java/lang/Object".to_string()), loader: bl.clone() });
    let next_frame= valid_type_transition(env, vec![object], &UnifiedType::IntType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}


pub fn instruction_is_type_safe_getfield(cp: CPIndex, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (field_class_name, field_name, field_descriptor) = extract_field_descriptor(cp, env);
    let field_type = translate_types_to_vm_types(&field_descriptor.field_type);
    passes_protected_check(env, &field_class_name.clone(), field_name, Descriptor::Field(&field_descriptor), stack_frame)?;
    let current_loader = env.class_loader.clone();
    let next_frame = valid_type_transition(env, vec![UnifiedType::Class(ClassWithLoader { class_name: field_class_name, loader: current_loader })], &field_type, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_getstatic(cp: CPIndex, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (_field_class_name, _field_name, field_descriptor) = extract_field_descriptor(cp, env);
    let field_type = translate_types_to_vm_types(&field_descriptor.field_type);
    let next_frame = valid_type_transition(env, vec![], &field_type, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}


//#[allow(unused)]
//pub fn instruction_is_type_safe_tableswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}


pub fn instruction_is_type_safe_anewarray(cp: CPIndex, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let sub_type = Box::new(extract_constant_pool_entry_as_type(cp, &env));
    let next_frame = valid_type_transition(env, vec![UnifiedType::IntType], &UnifiedType::ArrayReferenceType(ArrayType { sub_type }), stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

fn extract_constant_pool_entry_as_type(cp: CPIndex, env: &Environment) -> UnifiedType {
    let class = get_class(&env.vf, &env.method.class);
    let class_name = match &class.constant_pool[cp as usize].kind {
        ConstantKind::Class(c) => {
            extract_string_from_utf8(&class.constant_pool[c.name_index as usize])
        }
        _ => panic!()
    };
    let subtype = possibly_array_to_type(env, class_name);
    subtype
}

pub fn instruction_is_type_safe_arraylength(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    dbg!(stack_frame);
    let array_type = nth1_operand_stack_is(1, stack_frame)?;
    array_component_type(array_type)?;
    let next_frame = valid_type_transition(env, vec![UnifiedType::TopType], &UnifiedType::IntType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn array_component_type(type_: UnifiedType) -> Result<UnifiedType, TypeSafetyError> {
    use std::ops::Deref;
    Result::Ok(match type_ {
        UnifiedType::ArrayReferenceType(a) => a.sub_type.deref().clone(),
        UnifiedType::NullType => UnifiedType::NullType,
        _ => panic!()
    })
}

pub fn nth1_operand_stack_is(i: usize, frame: &Frame) -> Result<UnifiedType, TypeSafetyError> {
    Result::Ok(nth1(i, &frame.stack_map))
}

fn nth1(i: usize, o: &OperandStack) -> UnifiedType {
    o.data[i - 1].clone()
}

pub fn instruction_is_type_safe_athrow(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let to_pop = ClassWithLoader { class_name: ClassName::Str("java/lang/Throwable".to_string()), loader: env.vf.bootstrap_loader.clone() };
    can_pop(&env.vf, stack_frame, vec![UnifiedType::Class(to_pop)])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}

//todo duplication with class name parsing and array logic
pub fn instruction_is_type_safe_checkcast(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let object_class = ClassWithLoader { class_name: ClassName::Str("java/lang/Object".to_string()), loader: env.vf.bootstrap_loader.clone() };
    let class = get_class(&env.vf, env.method.class);
    let result_type = match &class.constant_pool[index].kind {
        ConstantKind::Class(c) => {
            let name = extract_string_from_utf8(&class.constant_pool[c.name_index as usize]);
            possibly_array_to_type(env, name)
        }
        _ => panic!()
    };
    dbg!(&result_type);
    let next_frame = valid_type_transition(env, vec![UnifiedType::Class(object_class)], &result_type, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}


pub fn instruction_is_type_safe_putfield(cp: CPIndex, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    match instruction_is_type_safe_putfield_first_case(cp, env, offset, stack_frame) {
        Ok(res) => Result::Ok(res),
        Err(_) => instruction_is_type_safe_putfield_second_case(cp, env, offset, stack_frame),
    }
}

fn instruction_is_type_safe_putfield_second_case(cp: CPIndex, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    //todo duplication
    let (field_class_name, _field_name, field_descriptor) = extract_field_descriptor(cp, env);
    let field_type = translate_types_to_vm_types(&field_descriptor.field_type);
    if env.method.class.class_name != field_class_name {
        return Result::Err(unknown_error_verifying!());
    }
    //todo is this equivalent to isInit
    if get_referred_name(&env.method.class.class_name) != "<init>" {
        return Result::Err(unknown_error_verifying!());
    }
    let next_stack_frame = can_pop(&env.vf, stack_frame, vec![field_type, UnifiedType::UninitializedThis])?;
    let exception_frame = exception_stack_frame(&stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { exception_frame, next_frame: next_stack_frame }))
}

fn instruction_is_type_safe_putfield_first_case(cp: CPIndex, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (field_class_name, field_name, field_descriptor) = extract_field_descriptor(cp, env);
    let field_type = translate_types_to_vm_types(&field_descriptor.field_type);
    let _popped_frame = can_pop(&env.vf, stack_frame, vec![field_type.clone()])?;
    passes_protected_check(env, &field_class_name.clone(), field_name, Descriptor::Field(&field_descriptor), stack_frame)?;
    let current_loader = env.class_loader.clone();
    let next_stack_frame = can_pop(&env.vf, stack_frame, vec![field_type, UnifiedType::Class(ClassWithLoader { loader: current_loader, class_name: field_class_name })])?;
    let exception_frame = exception_stack_frame(&stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { exception_frame, next_frame: next_stack_frame }))
}


fn extract_field_descriptor(cp: CPIndex, env: &Environment) -> (ClassName, String, FieldDescriptor) {
    let current_class = get_class(&env.vf, env.method.class);
    let field_entry: &ConstantKind = &current_class.constant_pool[cp as usize].kind;
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

pub fn instruction_is_type_safe_putstatic(cp: CPIndex, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (_field_class_name, _field_name, field_descriptor) = extract_field_descriptor(cp, env);
    let field_type = translate_types_to_vm_types(&field_descriptor.field_type);
    let next_frame = can_pop(&env.vf, stack_frame, vec![field_type])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}


pub fn instruction_is_type_safe_monitorenter(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = can_pop(&env.vf, stack_frame, vec![UnifiedType::Reference])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_multianewarray(cp: usize, dim: usize, env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//
//
pub fn instruction_is_type_safe_new(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let operand_stack = &stack_frame.stack_map;
    let flags = stack_frame.flag_this_uninit;
    match &get_class(&env.vf, env.method.class).constant_pool[cp].kind {
        ConstantKind::Class(_) => {}
        _ => panic!()
    };
    let new_item = UnifiedType::Uninitialized(UninitializedVariableInfo { offset: offset as u16 });
    match operand_stack.iter().find(|x| {
        x == &&new_item
    }) {
        None => {}
        Some(_) => return Result::Err(unknown_error_verifying!()),
    };
    let new_locals = substitute(&new_item, &UnifiedType::TopType, locals.as_slice());
    let next_frame = valid_type_transition(env, vec![], &new_item, &Frame {
        locals: new_locals,
        stack_map: operand_stack.clone(),
        flag_this_uninit: flags,
    })?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_newarray(type_code: usize, env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}

//
//#[allow(unused)]
//pub fn instruction_is_type_safe_lookupswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//