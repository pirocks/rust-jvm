use crate::{InterpreterState, StackEntry};
use classfile_parser::code::CodeParserContext;
use classfile_parser::code::parse_instruction;
use rust_jvm_common::classfile::{InstructionInfo, ACC_STATIC, Classfile};
use crate::runtime_class::prepare_class;
use crate::runtime_class::initialize_class;
use std::sync::Arc;
use rust_jvm_common::classnames::{ClassName, class_name};
use rust_jvm_common::loading::LoaderArc;
use std::rc::Rc;
use crate::instructions::invoke::{run_invoke_static, invoke_special, invoke_virtual};
use runtime_common::java_values::{JavaValue, default_value, Object};
use runtime_common::runtime_class::RuntimeClass;
use classfile_parser::types::parse_field_descriptor;
use crate::instructions::load::{aload, fload, iload, aaload};
use crate::instructions::store::{astore, castore, aastore};
use crate::instructions::fields::{get_field, get_static, putfield, putstatic};
use crate::instructions::cmp::{fcmpg, fcmpl};
use crate::instructions::conversion::{i2l, i2f, f2i};
use crate::instructions::new::{new, anewarray, newarray};
use crate::instructions::return_::{return_, areturn, dreturn, freturn, ireturn};
use crate::instructions::arithmetic::{ladd, land, lshl, fmul, iand, irem, iadd, ishl, isub};
use crate::instructions::constant::{fconst_0, sipush, bipush, aconst_null};
use crate::instructions::ldc::{ldc, ldc2_w};
use crate::instructions::dup::dup;
use crate::instructions::branch::{goto_, iconst_0, iconst_1, iconst_2, iconst_3, iconst_4, iconst_5, if_icmpgt, ifeq, ifne, iflt, ifge, ifgt, ifle, ifnonnull, ifnull, if_icmplt, if_icmpne, if_acmpne};
use crate::instructions::special::{arraylength, invoke_instanceof};
use log::trace;

//todo jni should really live in interpreter state
pub fn check_inited_class(
    state: &mut InterpreterState,
    class_name: &ClassName,
    current_frame: Option<Rc<StackEntry>>,
    loader_arc: LoaderArc,
) -> Arc<RuntimeClass> {
    //todo racy/needs sychronization
    if !state.initialized_classes.read().unwrap().contains_key(&class_name) {
        let bl = state.bootstrap_loader.clone();
        let target_classfile = loader_arc.clone().load_class(loader_arc.clone(), &class_name, bl).unwrap();
        let prepared = Arc::new(prepare_class(target_classfile.clone(), loader_arc.clone()));
        state.initialized_classes.write().unwrap().insert(class_name.clone(), prepared.clone());//must be before, otherwise infinite recurse
        let inited_target = initialize_class(prepared, state, current_frame);
        state.initialized_classes.write().unwrap().insert(class_name.clone(), inited_target);
    }
    let res = state.initialized_classes.read().unwrap().get(class_name).unwrap().clone();
//    dbg!(&res.static_vars.borrow().get("savedProps"));
    res
}



pub fn run_function(
    state: &mut InterpreterState,
    current_frame: Rc<StackEntry>,
) {
    let methods = &current_frame.class_pointer.classfile.methods;
    let method = &methods[current_frame.method_i as usize];
    let code = method.code_attribute().unwrap();
    let meth_name = method.method_name(&current_frame.class_pointer.classfile);
    if meth_name == "storeToXML" && class_name(&current_frame.class_pointer.classfile) == ClassName::Str("java/util/Properties".to_string()){
        dbg!(&current_frame.local_vars);
        dbg!("here");
    }
    trace!("CALL BEGIN:{} {} {}", class_name(&current_frame.class_pointer.classfile).get_referred_name(), meth_name, current_frame.depth());
    assert!(!state.function_return);
    while !state.terminate && !state.function_return && !state.throw {
        let (instruct, instruction_size) = {
            let current = &code.code_raw[*current_frame.pc.borrow()..];
            let mut context = CodeParserContext { offset: 0, iter: current.iter() };
            (parse_instruction(&mut context).unwrap().clone(), context.offset)
        };
        current_frame.pc_offset.replace(instruction_size as isize);
//        dbg!(instruct.clone());
        match instruct {
            InstructionInfo::aaload => aaload(&current_frame),
            InstructionInfo::aastore => aastore(&current_frame),
            InstructionInfo::aconst_null => aconst_null(&current_frame),
            InstructionInfo::aload(n) => aload(&current_frame, n as usize),
            InstructionInfo::aload_0 => aload(&current_frame, 0),
            InstructionInfo::aload_1 => aload(&current_frame, 1),
            InstructionInfo::aload_2 => aload(&current_frame, 2),
            InstructionInfo::aload_3 => aload(&current_frame, 3),
            InstructionInfo::anewarray(cp) => anewarray(state, current_frame.clone(), cp),
            InstructionInfo::areturn => areturn(state, &current_frame),
            InstructionInfo::arraylength => arraylength(&current_frame),
            InstructionInfo::astore(n) => astore(&current_frame, n as usize),
            InstructionInfo::astore_0 => astore(&current_frame, 0),
            InstructionInfo::astore_1 => astore(&current_frame, 1),
            InstructionInfo::astore_2 => astore(&current_frame, 2),
            InstructionInfo::astore_3 => astore(&current_frame, 3),
            InstructionInfo::athrow => unimplemented!(),
            InstructionInfo::baload => unimplemented!(),
            InstructionInfo::bastore => unimplemented!(),
            InstructionInfo::bipush(b) => bipush(&current_frame, b),
            InstructionInfo::caload => unimplemented!(),
            InstructionInfo::castore => castore(&current_frame),
            InstructionInfo::checkcast(_) => unimplemented!(),
            InstructionInfo::d2f => unimplemented!(),
            InstructionInfo::d2i => unimplemented!(),
            InstructionInfo::d2l => unimplemented!(),
            InstructionInfo::dadd => unimplemented!(),
            InstructionInfo::daload => unimplemented!(),
            InstructionInfo::dastore => unimplemented!(),
            InstructionInfo::dcmpg => unimplemented!(),
            InstructionInfo::dcmpl => unimplemented!(),
            InstructionInfo::dconst_0 => unimplemented!(),
            InstructionInfo::dconst_1 => unimplemented!(),
            InstructionInfo::ddiv => unimplemented!(),
            InstructionInfo::dload(_) => unimplemented!(),
            InstructionInfo::dload_0 => unimplemented!(),
            InstructionInfo::dload_1 => unimplemented!(),
            InstructionInfo::dload_2 => unimplemented!(),
            InstructionInfo::dload_3 => unimplemented!(),
            InstructionInfo::dmul => unimplemented!(),
            InstructionInfo::dneg => unimplemented!(),
            InstructionInfo::drem => unimplemented!(),
            InstructionInfo::dreturn => dreturn(state, &current_frame),
            InstructionInfo::dstore(_) => unimplemented!(),
            InstructionInfo::dstore_0 => unimplemented!(),
            InstructionInfo::dstore_1 => unimplemented!(),
            InstructionInfo::dstore_2 => unimplemented!(),
            InstructionInfo::dstore_3 => unimplemented!(),
            InstructionInfo::dsub => unimplemented!(),
            InstructionInfo::dup => dup(&current_frame),
            InstructionInfo::dup_x1 => unimplemented!(),
            InstructionInfo::dup_x2 => unimplemented!(),
            InstructionInfo::dup2 => unimplemented!(),
            InstructionInfo::dup2_x1 => unimplemented!(),
            InstructionInfo::dup2_x2 => unimplemented!(),
            InstructionInfo::f2d => unimplemented!(),
            InstructionInfo::f2i => f2i(&current_frame),
            InstructionInfo::f2l => unimplemented!(),
            InstructionInfo::fadd => unimplemented!(),
            InstructionInfo::faload => unimplemented!(),
            InstructionInfo::fastore => unimplemented!(),
            InstructionInfo::fcmpg => fcmpg(&current_frame),
            InstructionInfo::fcmpl => fcmpl(&current_frame),
            InstructionInfo::fconst_0 => fconst_0(&current_frame),
            InstructionInfo::fconst_1 => unimplemented!(),
            InstructionInfo::fconst_2 => unimplemented!(),
            InstructionInfo::fdiv => unimplemented!(),
            InstructionInfo::fload(_) => unimplemented!(),
            InstructionInfo::fload_0 => fload(&current_frame, 0),
            InstructionInfo::fload_1 => fload(&current_frame, 1),
            InstructionInfo::fload_2 => fload(&current_frame, 2),
            InstructionInfo::fload_3 => unimplemented!(),
            InstructionInfo::fmul => fmul(current_frame.clone()),
            InstructionInfo::fneg => unimplemented!(),
            InstructionInfo::frem => unimplemented!(),
            InstructionInfo::freturn => freturn(state, &current_frame),
            InstructionInfo::fstore(_) => unimplemented!(),
            InstructionInfo::fstore_0 => unimplemented!(),
            InstructionInfo::fstore_1 => unimplemented!(),
            InstructionInfo::fstore_2 => unimplemented!(),
            InstructionInfo::fstore_3 => unimplemented!(),
            InstructionInfo::fsub => unimplemented!(),
            InstructionInfo::getfield(cp) => get_field(&current_frame, cp),
            InstructionInfo::getstatic(cp) => get_static(state, &current_frame, cp),
            InstructionInfo::goto_(target) => goto_(&current_frame, target),
            InstructionInfo::goto_w(_) => unimplemented!(),
            InstructionInfo::i2b => unimplemented!(),
            InstructionInfo::i2c => unimplemented!(),
            InstructionInfo::i2d => unimplemented!(),
            InstructionInfo::i2f => i2f(&current_frame),
            InstructionInfo::i2l => i2l(&current_frame),
            InstructionInfo::i2s => unimplemented!(),
            InstructionInfo::iadd => iadd(&current_frame),
            InstructionInfo::iaload => unimplemented!(),
            InstructionInfo::iand => iand(&current_frame),
            InstructionInfo::iastore => unimplemented!(),
            InstructionInfo::iconst_m1 => unimplemented!(),
            InstructionInfo::iconst_0 => iconst_0(&current_frame),
            InstructionInfo::iconst_1 => iconst_1(&current_frame),
            InstructionInfo::iconst_2 => iconst_2(&current_frame),
            InstructionInfo::iconst_3 => iconst_3(&current_frame),
            InstructionInfo::iconst_4 => iconst_4(&current_frame),
            InstructionInfo::iconst_5 => iconst_5(&current_frame),
            InstructionInfo::idiv => unimplemented!(),
            InstructionInfo::if_acmpeq(_) => unimplemented!(),
            InstructionInfo::if_acmpne(offset) => if_acmpne(&current_frame, offset),
            InstructionInfo::if_icmpeq(_) => unimplemented!(),
            InstructionInfo::if_icmpne(offset) => if_icmpne(&current_frame, offset),
            InstructionInfo::if_icmplt(offset) => if_icmplt(&current_frame, offset),
            InstructionInfo::if_icmpge(_) => unimplemented!(),
            InstructionInfo::if_icmpgt(offset) => if_icmpgt(&current_frame, offset),
            InstructionInfo::if_icmple(_) => unimplemented!(),
            InstructionInfo::ifeq(offset) => ifeq(&current_frame, offset),
            InstructionInfo::ifne(offset) => ifne(&current_frame, offset),
            InstructionInfo::iflt(offset) => iflt(&current_frame, offset),
            InstructionInfo::ifge(offset) => ifge(&current_frame, offset),
            InstructionInfo::ifgt(offset) => ifgt(&current_frame, offset),
            InstructionInfo::ifle(offset) => ifle(&current_frame, offset),
            InstructionInfo::ifnonnull(offset) => ifnonnull(&current_frame, offset),
            InstructionInfo::ifnull(offset) => ifnull(&current_frame, offset),
            InstructionInfo::iinc(iinc) => {
                let val = current_frame.local_vars.borrow()[iinc.index as usize].unwrap_int();
                let res = val + iinc.const_ as i32;
                current_frame.local_vars.borrow_mut()[iinc.index as usize] = JavaValue::Int(res);
            },
            InstructionInfo::iload(n) => iload(&current_frame, n as usize),
            InstructionInfo::iload_0 => iload(&current_frame, 0),
            InstructionInfo::iload_1 => iload(&current_frame, 1),
            InstructionInfo::iload_2 => iload(&current_frame, 2),
            InstructionInfo::iload_3 => iload(&current_frame, 3),
            InstructionInfo::imul => unimplemented!(),
            InstructionInfo::ineg => unimplemented!(),
            InstructionInfo::instanceof(cp) => invoke_instanceof(state,&current_frame,cp),
            InstructionInfo::invokedynamic(_) => unimplemented!(),
            InstructionInfo::invokeinterface(_) => unimplemented!(),
            InstructionInfo::invokespecial(cp) => invoke_special(state, &current_frame, cp),
            InstructionInfo::invokestatic(cp) => run_invoke_static(state, current_frame.clone(), cp),
            InstructionInfo::invokevirtual(cp) => invoke_virtual(state, current_frame.clone(), cp),
            InstructionInfo::ior => unimplemented!(),
            InstructionInfo::irem => irem(&current_frame),
            InstructionInfo::ireturn => ireturn(state, &current_frame),
            InstructionInfo::ishl => ishl(&current_frame),
            InstructionInfo::ishr => unimplemented!(),
            InstructionInfo::istore(n) => istore(&current_frame, n),
            InstructionInfo::istore_0 => istore(&current_frame, 0),
            InstructionInfo::istore_1 => istore(&current_frame, 1),
            InstructionInfo::istore_2 => istore(&current_frame, 2),
            InstructionInfo::istore_3 => istore(&current_frame, 3),
            InstructionInfo::isub => isub(&current_frame),
            InstructionInfo::iushr => unimplemented!(),
            InstructionInfo::ixor => unimplemented!(),
            InstructionInfo::jsr(_) => unimplemented!(),
            InstructionInfo::jsr_w(_) => unimplemented!(),
            InstructionInfo::l2d => unimplemented!(),
            InstructionInfo::l2f => unimplemented!(),
            InstructionInfo::l2i => unimplemented!(),
            InstructionInfo::ladd => ladd(current_frame.clone()),
            InstructionInfo::laload => unimplemented!(),
            InstructionInfo::land => land(current_frame.clone()),
            InstructionInfo::lastore => unimplemented!(),
            InstructionInfo::lcmp => unimplemented!(),
            InstructionInfo::lconst_0 => unimplemented!(),
            InstructionInfo::lconst_1 => unimplemented!(),
            InstructionInfo::ldc(cp) => ldc(state, current_frame.clone(), cp),
            InstructionInfo::ldc_w(_) => unimplemented!(),
            InstructionInfo::ldc2_w(cp) => ldc2_w(current_frame.clone(), cp),
            InstructionInfo::ldiv => unimplemented!(),
            InstructionInfo::lload(_) => unimplemented!(),
            InstructionInfo::lload_0 => unimplemented!(),
            InstructionInfo::lload_1 => unimplemented!(),
            InstructionInfo::lload_2 => unimplemented!(),
            InstructionInfo::lload_3 => unimplemented!(),
            InstructionInfo::lmul => unimplemented!(),
            InstructionInfo::lneg => unimplemented!(),
            InstructionInfo::lookupswitch(_) => unimplemented!(),
            InstructionInfo::lor => unimplemented!(),
            InstructionInfo::lrem => unimplemented!(),
            InstructionInfo::lreturn => unimplemented!(),
            InstructionInfo::lshl => lshl(current_frame.clone()),
            InstructionInfo::lshr => unimplemented!(),
            InstructionInfo::lstore(_) => unimplemented!(),
            InstructionInfo::lstore_0 => unimplemented!(),
            InstructionInfo::lstore_1 => unimplemented!(),
            InstructionInfo::lstore_2 => unimplemented!(),
            InstructionInfo::lstore_3 => unimplemented!(),
            InstructionInfo::lsub => unimplemented!(),
            InstructionInfo::lushr => unimplemented!(),
            InstructionInfo::lxor => unimplemented!(),
            InstructionInfo::monitorenter => { /*unimplemented for now todo*/ }
            InstructionInfo::monitorexit => { /*unimplemented for now todo*/ }
            InstructionInfo::multianewarray(_) => unimplemented!(),
            InstructionInfo::new(cp) => new(state, &current_frame, cp as usize),
            InstructionInfo::newarray(a_type) => newarray(&current_frame, a_type),
            InstructionInfo::nop => unimplemented!(),
            InstructionInfo::pop => {current_frame.pop();},
            InstructionInfo::pop2 => unimplemented!(),
            InstructionInfo::putfield(cp) => putfield(state, &current_frame, cp),
            InstructionInfo::putstatic(cp) => putstatic(state, &current_frame, cp),
            InstructionInfo::ret(_) => unimplemented!(),
            InstructionInfo::return_ => return_(state),
            InstructionInfo::saload => unimplemented!(),
            InstructionInfo::sastore => unimplemented!(),
            InstructionInfo::sipush(val) => sipush(&current_frame, val),
            InstructionInfo::swap => unimplemented!(),
            InstructionInfo::tableswitch(_) => unimplemented!(),
            InstructionInfo::wide(_) => unimplemented!(),
            InstructionInfo::EndOfCode => unimplemented!(),
        }
        //todo need to figure out where return res ends up on next stack
        let offset = *current_frame.pc_offset.borrow();
        let mut pc = *current_frame.pc.borrow();
        if offset > 0 {
            pc += offset as usize;
        } else {
            pc -= (-offset) as usize;//todo perhaps i don't have to do this bs if I use u64 instead of usize
        }
        current_frame.pc.replace(pc);
//        dbg!(&current_frame.class_pointer.static_vars.borrow().get("savedProps"));
    }
//    dbg!(&current_frame.class_pointer.static_vars.borrow().get("savedProps"));
    trace!("CALL END:{} {} {}", class_name(&current_frame.class_pointer.classfile).get_referred_name(), meth_name, current_frame.depth());
}

fn istore(current_frame: &Rc<StackEntry>, n: u8) -> () {
    let object_ref = current_frame.pop();
    match object_ref.clone() {
        JavaValue::Int(_) => {}
        _ => {
            dbg!(&object_ref);
            panic!()
        }
    }
    current_frame.local_vars.borrow_mut()[n as usize] = object_ref;
}


pub fn push_new_object(current_frame: Rc<StackEntry>, target_classfile: &Arc<RuntimeClass>) {
    let loader_arc = &current_frame.class_pointer.loader.clone();
    let object_pointer = JavaValue::new_object(target_classfile.clone());
    let new_obj = JavaValue::Object(object_pointer.clone());
    default_init_fields(loader_arc.clone(), object_pointer, &target_classfile.classfile, loader_arc.clone());
    current_frame.push(new_obj);
}

fn default_init_fields(loader_arc: LoaderArc, object_pointer: Option<Arc<Object>>, classfile: &Arc<Classfile>, bl: LoaderArc) {
    if classfile.super_class != 0 {
        let super_name = classfile.super_class_name();
        let loaded_super = loader_arc.load_class(loader_arc.clone(), &super_name, bl.clone()).unwrap();
        default_init_fields(loader_arc.clone(), object_pointer.clone(), &loaded_super, bl);
    }
    for field in &classfile.fields {
        if field.access_flags & ACC_STATIC == 0 {
            //todo should I look for constant val attributes?
            let _value_i = match field.constant_value_attribute_i() {
                None => {}
                Some(_i) => unimplemented!(),
            };
            let name = classfile.constant_pool[field.name_index as usize].extract_string_from_utf8();
            let descriptor_str = classfile.constant_pool[field.descriptor_index as usize].extract_string_from_utf8();
            let descriptor = parse_field_descriptor(&loader_arc, descriptor_str.as_str()).unwrap();
            let type_ = descriptor.field_type;
            let val = default_value(type_);
            {
                object_pointer.clone().unwrap().fields.borrow_mut().insert(name, val);
            }
        }
    }
}

