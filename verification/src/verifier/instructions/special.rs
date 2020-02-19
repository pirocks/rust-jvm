use crate::verifier::codecorrectness::{Environment, valid_type_transition};
use crate::verifier::{Frame, standard_exception_frame};
use crate::verifier::instructions::{InstructionTypeSafe, AfterGotoFrames, exception_stack_frame};
use crate::verifier::TypeSafetyError;
use crate::verifier::get_class;
use rust_jvm_common::classfile::{ConstantKind, UninitializedVariableInfo};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::classnames::NameReference;
use std::sync::Arc;
use crate::verifier::codecorrectness::can_pop;
use crate::verifier::passes_protected_check;
use rust_jvm_common::classfile::CPIndex;
use crate::verifier::instructions::branches::{substitute, possibly_array_to_type};
use crate::OperandStack;
use rust_jvm_common::unified_types::PType;
use crate::verifier::instructions::type_transition;
use crate::verifier::instructions::target_is_type_safe;
use rust_jvm_common::classfile::Classfile;
use descriptor_parser::{Descriptor, FieldDescriptor, parse_field_type, parse_field_descriptor};
use rust_jvm_common::vtype::VType;
use rust_jvm_common::loading::ClassWithLoader;
use rust_jvm_common::view::ptype_view::PTypeView;

pub fn instruction_is_type_safe_instanceof(_cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    //todo verify that cp is valid
    let bl = &env.vf.bootstrap_loader.clone();
    let object = VType::Class(ClassWithLoader { class_name: ClassName::object(), loader: bl.clone() });
    type_transition(env, stack_frame, vec![object], VType::IntType)
}


pub fn instruction_is_type_safe_getfield(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (field_class_name, field_name, field_descriptor) = extract_field_descriptor(cp, get_class(&env.vf, env.method.class));
    let field_type = &field_descriptor.field_type.to_verification_type(&env.class_loader);
    passes_protected_check(env, &field_class_name.clone(), field_name, Descriptor::Field(&field_descriptor), stack_frame)?;
    let current_loader = env.class_loader.clone();
    let expected_types_on_stack = vec![VType::Class(ClassWithLoader { class_name: field_class_name, loader: current_loader })];
    type_transition(env, stack_frame, expected_types_on_stack, field_type.clone())
}

pub fn instruction_is_type_safe_getstatic(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (_field_class_name, _field_name, field_descriptor) = extract_field_descriptor(cp, get_class(&env.vf, env.method.class));
    let field_type = &field_descriptor.field_type.to_verification_type(&env.class_loader);
    type_transition(env, stack_frame, vec![], field_type.clone())
}


pub fn instruction_is_type_safe_tableswitch(targets: Vec<usize>, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let branch_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType])?;
    for t in targets {
        target_is_type_safe(env, &branch_frame, t)?;
    }
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}


pub fn instruction_is_type_safe_anewarray(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let sub_type = extract_constant_pool_entry_as_type(cp, &env);
    let expected_types_on_stack = vec![VType::IntType];
    let res_type = VType::ArrayReferenceType(sub_type);
    type_transition(env, stack_frame, expected_types_on_stack, res_type)
}

fn extract_constant_pool_entry_as_type(cp: CPIndex, env: &Environment) -> PType {
    let class = get_class(&env.vf, &env.method.class);
    let class_name = match &class.constant_pool[cp as usize].kind {
        ConstantKind::Class(c) => {
            class.constant_pool[c.name_index as usize].extract_string_from_utf8()
        }
        _ => panic!()
    };
    let subtype = possibly_array_to_type(class_name);
    PType::Ref(subtype)
}

pub fn instruction_is_type_safe_arraylength(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(1, stack_frame)?;
    array_component_type(array_type)?;
    type_transition(env, stack_frame, vec![VType::TopType], VType::IntType)
}

pub fn array_component_type(type_: VType) -> Result<PTypeView, TypeSafetyError> {
    Result::Ok(match type_ {
        VType::ArrayReferenceType(a) => a.clone(),
        VType::NullType => PTypeView::NullType,
        _ => panic!()
    })
}

pub fn nth1_operand_stack_is(i: usize, frame: &Frame) -> Result<VType, TypeSafetyError> {
    Result::Ok(nth1(i, &frame.stack_map))
}

fn nth1(i: usize, o: &OperandStack) -> VType {
    o.data[i - 1].clone()
}

pub fn instruction_is_type_safe_athrow(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let to_pop = ClassWithLoader { class_name: ClassName::throwable(), loader: env.vf.bootstrap_loader.clone() };
    can_pop(&env.vf, stack_frame, vec![VType::Class(to_pop)])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}

//todo duplication with class name parsing and array logic
pub fn instruction_is_type_safe_checkcast(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let object_class = ClassWithLoader { class_name: ClassName::object(), loader: env.vf.bootstrap_loader.clone() };
    let class = get_class(&env.vf, env.method.class);
    let result_type = match &class.constant_pool[index].kind {
        ConstantKind::Class(c) => {
            let name = class.constant_pool[c.name_index as usize].extract_string_from_utf8();
            PType::Ref(possibly_array_to_type(name)).to_verification_type(&env.class_loader)
        }
        _ => panic!()
    };
    let expected_types_on_stack = vec![VType::Class(object_class)];
    type_transition(env, stack_frame, expected_types_on_stack, result_type)
}


pub fn instruction_is_type_safe_putfield(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let method_classfile = get_class(&env.vf, env.method.class);
    if method_classfile.methods[env.method.method_index].method_name(&method_classfile) == "<init>" {
        match instruction_is_type_safe_putfield_second_case(cp, env, stack_frame) {
            Ok(res) => return Result::Ok(res),
            Err(_) => {}
        };
    }
    match instruction_is_type_safe_putfield_first_case(cp, env, stack_frame) {
        Ok(res) => Result::Ok(res),
        Err(_) => instruction_is_type_safe_putfield_second_case(cp, env, stack_frame),
    }
}

fn instruction_is_type_safe_putfield_second_case(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    //todo duplication
    let (field_class_name, _field_name, field_descriptor) = extract_field_descriptor(cp, get_class(&env.vf, env.method.class));
    let field_type = (&field_descriptor.field_type).to_verification_type(&env.class_loader);
    if env.method.class.class_name != field_class_name {
        return Result::Err(unknown_error_verifying!());
    }
    //todo is this equivalent to isInit
    let method_classfile = get_class(&env.vf, env.method.class);
    if method_classfile.methods[env.method.method_index].method_name(&method_classfile) != "<init>" {
        return Result::Err(unknown_error_verifying!());
    }
    let next_frame = can_pop(&env.vf, stack_frame, vec![field_type, VType::UninitializedThis])?;
    standard_exception_frame(stack_frame, next_frame)
}

fn instruction_is_type_safe_putfield_first_case(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (field_class_name, field_name, field_descriptor) = extract_field_descriptor(cp, get_class(&env.vf, env.method.class));
    let field_type = (&field_descriptor.field_type).to_verification_type(&env.class_loader);
    let _popped_frame = can_pop(&env.vf, stack_frame, vec![field_type.clone()])?;
    passes_protected_check(env, &field_class_name.clone(), field_name, Descriptor::Field(&field_descriptor), stack_frame)?;
    let current_loader = env.class_loader.clone();
//    dbg!(&stack_frame);
    let next_frame = can_pop(&env.vf, stack_frame, vec![field_type, VType::Class(ClassWithLoader { loader: current_loader, class_name: field_class_name })])?;
    standard_exception_frame(stack_frame, next_frame)
}

//todo maybe move to impl
pub fn extract_field_descriptor(cp: CPIndex, class: Arc<Classfile>) -> (ClassName, String, FieldDescriptor) {
//    dbg!(cp);
    let current_class = class;
    let field_entry: &ConstantKind = &current_class.constant_pool[cp as usize].kind;
    let (class_index, name_and_type_index) = match field_entry {
        ConstantKind::Fieldref(f) => {
            (f.class_index, f.name_and_type_index)
        }
        _ => {
            dbg!(&field_entry);
            panic!()
        }
    };
    let field_class_name = match &current_class.constant_pool[class_index as usize].kind {
        ConstantKind::Class(c) => {
            ClassName::Ref(NameReference { class_file: Arc::downgrade(&current_class), index: c.name_index })
        }
        _ => panic!()
    };
//    dbg!(&field_class_name);
    let (field_name_index, descriptor_index) = match &current_class.constant_pool[name_and_type_index as usize].kind {
        ConstantKind::NameAndType(nt) => {
            (nt.name_index, nt.descriptor_index)
        }
        _ => panic!()
    };
    let field_name = current_class.constant_pool[field_name_index as usize].extract_string_from_utf8();
    let descriptor_string = current_class.constant_pool[descriptor_index as usize].extract_string_from_utf8();
    let field_descriptor = parse_field_descriptor(descriptor_string.as_ref()).unwrap();
    (field_class_name, field_name, field_descriptor)
}

pub fn instruction_is_type_safe_putstatic(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (_field_class_name, _field_name, field_descriptor) = extract_field_descriptor(cp, get_class(&env.vf, env.method.class));
    let field_type = (&field_descriptor.field_type).to_verification_type(&env.class_loader);
    let next_frame = can_pop(&env.vf, stack_frame, vec![field_type])?;
    standard_exception_frame(stack_frame, next_frame)
}


pub fn instruction_is_type_safe_monitorenter(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::Reference])?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_multianewarray(cp: usize, dim: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let classfile = get_class(&env.vf, env.method.class);
    let expected_type = parse_field_type(classfile.extract_class_from_constant_pool_name(cp as u16).as_str()).unwrap().1;
    if class_dimension(env,&expected_type.to_verification_type(&env.class_loader)) != dim {
        return Result::Err(unknown_error_verifying!());
    }
    let dim_list = dim_list(dim);
    type_transition(env, stack_frame, dim_list, expected_type.to_verification_type(&env.class_loader))
}

fn dim_list(dim: usize) -> Vec<VType> {
    let mut res = vec![];
    for _ in 0..dim {
        res.push(VType::IntType)
    }
    res
}

fn class_dimension(env: &Environment,v: &VType) -> usize {
    match v {
        VType::ArrayReferenceType(sub) => {
            class_dimension(env,&sub.to_verification_type(&env.class_loader)) + 1
        }
        _ => 0,
//        _ => unimplemented!()
    }
}

pub fn instruction_is_type_safe_new(cp: usize, offset: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let operand_stack = &stack_frame.stack_map;
    let flags = stack_frame.flag_this_uninit;
    match &get_class(&env.vf, env.method.class).constant_pool[cp].kind {
        ConstantKind::Class(_) => {}
        _ => panic!()
    };
    let new_item = VType::Uninitialized(UninitializedVariableInfo { offset: offset as u16 });
    match operand_stack.iter().find(|x| {
        x == &&new_item
    }) {
        None => {}
        Some(_) => return Result::Err(unknown_error_verifying!()),
    };
    let new_locals = substitute(&new_item, &VType::TopType, locals.as_slice());
    let next_frame = valid_type_transition(env, vec![], &new_item, &Frame {
        locals: new_locals,
        stack_map: operand_stack.clone(),
        flag_this_uninit: flags,
    })?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_newarray(type_code: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let element_type = primitive_array_info(type_code);
    type_transition(env, stack_frame, vec![VType::IntType], VType::ArrayReferenceType(element_type))
}

fn primitive_array_info(type_code: usize) -> PType {
    match type_code {
        4 => PType::BooleanType,
        5 => PType::CharType,
        6 => PType::FloatType,
        7 => PType::DoubleType,
        8 => PType::ByteType,
        9 => PType::ShortType,
        10 => PType::IntType,
        11 => PType::LongType,
        _ => panic!()
    }
}

//impl Vec<usize> {
//todo replace with is_sorted when that becomes stable
fn sorted(nums: &Vec<i32>) -> bool {
    let mut old_x: i32 = std::i32::MIN;
    nums.iter().all(|x| {
        let res = old_x <= *x;
        old_x = *x;
        res
    })
}
//}

pub fn instruction_is_type_safe_lookupswitch(targets: Vec<usize>, keys: Vec<i32>, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    if !sorted(&keys) {
        return Result::Err(unknown_error_verifying!());
    }
    let branch_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType])?;
    for t in targets {
        target_is_type_safe(env, &branch_frame, t)?;
    }
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}
