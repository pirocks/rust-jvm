use crate::{InterpreterState, StackEntry};
use classfile_parser::code::CodeParserContext;
use classfile_parser::code::parse_instruction;
use rust_jvm_common::classfile::{InstructionInfo, ACC_STATIC, Classfile};
use crate::runtime_class::prepare_class;
use crate::runtime_class::initialize_class;
use std::sync::Arc;
use rust_jvm_common::classnames::{ClassName, class_name};

use std::rc::Rc;
use runtime_common::java_values::{JavaValue, default_value, Object};
use runtime_common::runtime_class::RuntimeClass;
use crate::instructions::load::*;
use crate::instructions::store::*;
use crate::instructions::fields::*;
use crate::instructions::cmp::{fcmpg, fcmpl};
use crate::instructions::conversion::*;
use crate::instructions::new::*;
use crate::instructions::return_::*;
use crate::instructions::arithmetic::*;
use crate::instructions::constant::{fconst_0, sipush, bipush, aconst_null, fconst_1};
use crate::instructions::ldc::{ldc_w, ldc2_w};
use crate::instructions::dup::*;
use crate::instructions::branch::*;
use crate::instructions::special::{arraylength, invoke_instanceof, invoke_checkcast, inherits_from};
use crate::instructions::switch::{invoke_lookupswitch, tableswitch};

use crate::instructions::invoke::interface::invoke_interface;
use crate::instructions::invoke::special::invoke_special;
use crate::instructions::invoke::static_::run_invoke_static;
use crate::instructions::invoke::virtual_::{invoke_virtual_instruction, invoke_virtual_method_i};
use crate::instructions::invoke::dynamic::invoke_dynamic;
use crate::instructions::pop::{pop2, pop};
use classfile_view::view::descriptor_parser::{parse_field_descriptor, parse_method_descriptor};
use classfile_view::loading::LoaderArc;


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
        let prepared = Arc::new(prepare_class(target_classfile.backing_class(), loader_arc.clone()));
        state.initialized_classes.write().unwrap().insert(class_name.clone(), prepared.clone());//must be before, otherwise infinite recurse
        let inited_target = initialize_class(prepared, state, current_frame);
        state.initialized_classes.write().unwrap().insert(class_name.clone(), inited_target);
    }
    let res = state.initialized_classes.read().unwrap().get(class_name).unwrap().clone();
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
    let class_name__ = class_name(&current_frame.class_pointer.classfile);


    let class_name_ = class_name__.get_referred_name();
    let method_desc = method.descriptor_str(&current_frame.class_pointer.classfile);
    let current_depth = current_frame.depth();
    println!("CALL BEGIN:{} {} {} {}", &class_name_, &meth_name, method_desc, current_depth);
    assert!(!state.function_return);
    if &meth_name == "make" && &method_desc == "(Ljava/lang/invoke/MemberName;)Ljava/lang/invoke/DirectMethodHandle;"{
        //ion resolve
        // dbg!(&current_frame.local_vars.borrow()[0].unwrap_object_nonnull().lookup_field("debugName"));
        dbg!(&current_frame.local_vars.borrow()[0].unwrap_object_nonnull());
        // dbg!(&current_frame.local_vars.borrow()[0].unwrap_object_nonnull().lookup_field("name"))
        // dbg!(&current_frame.last_call_stack.as_ref().unwrap().operand_stack.borrow().last());
    }
    if &meth_name == "resolve" && class_name_ == "java/lang/invoke/LambdaForm/NamedFunction" {
        //ion resolve
        // dbg!(&current_frame.local_vars.borrow()[0].unwrap_object_nonnull().lookup_field("debugName"));
        dbg!(&current_frame.local_vars.borrow()[0]);
        // dbg!(&current_frame.local_vars.borrow()[0].unwrap_object_nonnull().lookup_field("name"))
        // dbg!(&current_frame.last_call_stack.as_ref().unwrap().operand_stack.borrow().last());
    }
    while !state.terminate && !state.function_return && !state.throw.is_some() {
        let (instruct, instruction_size) = {
            let current = &code.code_raw[*current_frame.pc.borrow()..];
            let mut context = CodeParserContext { offset: *current_frame.pc.borrow(), iter: current.iter() };
            (parse_instruction(&mut context).unwrap().clone(), context.offset - *current_frame.pc.borrow())
        };
        current_frame.pc_offset.replace(instruction_size as isize);
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
            InstructionInfo::athrow => {
                println!("EXCEPTION:");
                current_frame.print_stack_trace();
                let exception_obj = current_frame.pop().unwrap_object_nonnull();
                dbg!(exception_obj.lookup_field("detailMessage"));
                state.throw = exception_obj.into();
            }
            InstructionInfo::baload => baload(&current_frame),
            InstructionInfo::bastore => bastore(&current_frame),
            InstructionInfo::bipush(b) => bipush(&current_frame, b),
            InstructionInfo::caload => caload(state, &current_frame),
            InstructionInfo::castore => castore(&current_frame),
            InstructionInfo::checkcast(cp) => invoke_checkcast(state, &current_frame, cp),
            InstructionInfo::d2f => unimplemented!(),
            InstructionInfo::d2i => d2i(&current_frame),
            InstructionInfo::d2l => d2l(&current_frame),
            InstructionInfo::dadd => dadd(&current_frame),
            InstructionInfo::daload => unimplemented!(),
            InstructionInfo::dastore => unimplemented!(),
            InstructionInfo::dcmpg => unimplemented!(),
            InstructionInfo::dcmpl => unimplemented!(),
            InstructionInfo::dconst_0 => dconst_0(&current_frame),
            InstructionInfo::dconst_1 => dconst_1(&current_frame),
            InstructionInfo::ddiv => unimplemented!(),
            InstructionInfo::dload(i) => dload(&current_frame, i as usize),
            InstructionInfo::dload_0 => dload(&current_frame, 0),
            InstructionInfo::dload_1 => dload(&current_frame, 1),
            InstructionInfo::dload_2 => dload(&current_frame, 2),
            InstructionInfo::dload_3 => dload(&current_frame, 3),
            InstructionInfo::dmul => dmul(&current_frame),
            InstructionInfo::dneg => unimplemented!(),
            InstructionInfo::drem => unimplemented!(),
            InstructionInfo::dreturn => dreturn(state, &current_frame),
            InstructionInfo::dstore(i) => dstore(&current_frame, i as usize),
            InstructionInfo::dstore_0 => dstore(&current_frame, 0 as usize),
            InstructionInfo::dstore_1 => dstore(&current_frame, 1 as usize),
            InstructionInfo::dstore_2 => dstore(&current_frame, 2 as usize),
            InstructionInfo::dstore_3 => dstore(&current_frame, 3 as usize),
            InstructionInfo::dsub => unimplemented!(),
            InstructionInfo::dup => dup(&current_frame),
            InstructionInfo::dup_x1 => dup_x1(&current_frame),
            InstructionInfo::dup_x2 => dup_x2(&current_frame),
            InstructionInfo::dup2 => dup2(&current_frame),
            InstructionInfo::dup2_x1 => dup2_x1(&current_frame),
            InstructionInfo::dup2_x2 => unimplemented!(),
            InstructionInfo::f2d => f2d(&current_frame),
            InstructionInfo::f2i => f2i(&current_frame),
            InstructionInfo::f2l => unimplemented!(),
            InstructionInfo::fadd => unimplemented!(),
            InstructionInfo::faload => unimplemented!(),
            InstructionInfo::fastore => unimplemented!(),
            InstructionInfo::fcmpg => fcmpg(&current_frame),
            InstructionInfo::fcmpl => fcmpl(&current_frame),
            InstructionInfo::fconst_0 => fconst_0(&current_frame),
            InstructionInfo::fconst_1 => fconst_1(&current_frame),
            InstructionInfo::fconst_2 => unimplemented!(),
            InstructionInfo::fdiv => fdiv(&current_frame),
            InstructionInfo::fload(_) => unimplemented!(),
            InstructionInfo::fload_0 => fload(&current_frame, 0),
            InstructionInfo::fload_1 => fload(&current_frame, 1),
            InstructionInfo::fload_2 => fload(&current_frame, 2),
            InstructionInfo::fload_3 => fload(&current_frame, 3),
            InstructionInfo::fmul => fmul(&current_frame),
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
            InstructionInfo::i2b => i2b(&current_frame),
            InstructionInfo::i2c => i2c(&current_frame),
            InstructionInfo::i2d => i2d(&current_frame),
            InstructionInfo::i2f => i2f(&current_frame),
            InstructionInfo::i2l => i2l(&current_frame),
            InstructionInfo::i2s => i2s(&current_frame),
            InstructionInfo::iadd => iadd(&current_frame),
            InstructionInfo::iaload => iaload(&current_frame),
            InstructionInfo::iand => iand(&current_frame),
            InstructionInfo::iastore => iastore(&current_frame),
            InstructionInfo::iconst_m1 => iconst_m1(&current_frame),
            InstructionInfo::iconst_0 => iconst_0(&current_frame),
            InstructionInfo::iconst_1 => iconst_1(&current_frame),
            InstructionInfo::iconst_2 => iconst_2(&current_frame),
            InstructionInfo::iconst_3 => iconst_3(&current_frame),
            InstructionInfo::iconst_4 => iconst_4(&current_frame),
            InstructionInfo::iconst_5 => iconst_5(&current_frame),
            InstructionInfo::idiv => idiv(&current_frame),
            InstructionInfo::if_acmpeq(offset) => if_acmpeq(&current_frame, offset),
            InstructionInfo::if_acmpne(offset) => if_acmpne(&current_frame, offset),
            InstructionInfo::if_icmpeq(offset) => if_icmpeq(&current_frame, offset),
            InstructionInfo::if_icmpne(offset) => if_icmpne(&current_frame, offset),
            InstructionInfo::if_icmplt(offset) => if_icmplt(&current_frame, offset),
            InstructionInfo::if_icmpge(offset) => if_icmpge(&current_frame, offset),
            InstructionInfo::if_icmpgt(offset) => if_icmpgt(&current_frame, offset),
            InstructionInfo::if_icmple(offset) => if_icmple(&current_frame, offset),
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
            }
            InstructionInfo::iload(n) => iload(&current_frame, n as usize),
            InstructionInfo::iload_0 => iload(&current_frame, 0),
            InstructionInfo::iload_1 => iload(&current_frame, 1),
            InstructionInfo::iload_2 => iload(&current_frame, 2),
            InstructionInfo::iload_3 => iload(&current_frame, 3),
            InstructionInfo::imul => imul(&current_frame),
            InstructionInfo::ineg => unimplemented!(),
            InstructionInfo::instanceof(cp) => invoke_instanceof(state, &current_frame, cp),
            InstructionInfo::invokedynamic(cp) => {
                // current_frame.print_stack_trace();
                invoke_dynamic(state, current_frame.clone(), cp)
            }
            InstructionInfo::invokeinterface(invoke_i) => invoke_interface(state, current_frame.clone(), invoke_i),
            InstructionInfo::invokespecial(cp) => invoke_special(state, &current_frame, cp),
            InstructionInfo::invokestatic(cp) => run_invoke_static(state, current_frame.clone(), cp),
            InstructionInfo::invokevirtual(cp) => invoke_virtual_instruction(state, current_frame.clone(), cp),
            InstructionInfo::ior => ior(&current_frame),
            InstructionInfo::irem => irem(&current_frame),
            InstructionInfo::ireturn => ireturn(state, &current_frame),
            InstructionInfo::ishl => ishl(&current_frame),
            InstructionInfo::ishr => ishr(&current_frame),
            InstructionInfo::istore(n) => istore(&current_frame, n),
            InstructionInfo::istore_0 => istore(&current_frame, 0),
            InstructionInfo::istore_1 => istore(&current_frame, 1),
            InstructionInfo::istore_2 => istore(&current_frame, 2),
            InstructionInfo::istore_3 => istore(&current_frame, 3),
            InstructionInfo::isub => isub(&current_frame),
            InstructionInfo::iushr => iushr(&current_frame),
            InstructionInfo::ixor => ixor(&current_frame),
            InstructionInfo::jsr(_) => unimplemented!(),
            InstructionInfo::jsr_w(_) => unimplemented!(),
            InstructionInfo::l2d => unimplemented!(),
            InstructionInfo::l2f => l2f(&current_frame),
            InstructionInfo::l2i => l2i(&current_frame),
            InstructionInfo::ladd => ladd(&current_frame),
            InstructionInfo::laload => unimplemented!(),
            InstructionInfo::land => land(current_frame.clone()),
            InstructionInfo::lastore => unimplemented!(),
            InstructionInfo::lcmp => lcmp(&current_frame),
            InstructionInfo::lconst_0 => lconst(&current_frame, 0),
            InstructionInfo::lconst_1 => lconst(&current_frame, 1),
            InstructionInfo::ldc(cp) => ldc_w(state, current_frame.clone(), cp as u16),
            InstructionInfo::ldc_w(cp) => ldc_w(state, current_frame.clone(), cp),
            InstructionInfo::ldc2_w(cp) => ldc2_w(current_frame.clone(), cp),
            InstructionInfo::ldiv => unimplemented!(),
            InstructionInfo::lload(i) => lload(&current_frame, i as usize),
            InstructionInfo::lload_0 => lload(&current_frame, 0),
            InstructionInfo::lload_1 => lload(&current_frame, 1),
            InstructionInfo::lload_2 => lload(&current_frame, 2),
            InstructionInfo::lload_3 => lload(&current_frame, 3),
            InstructionInfo::lmul => unimplemented!(),
            InstructionInfo::lneg => unimplemented!(),
            InstructionInfo::lookupswitch(ls) => invoke_lookupswitch(&ls, &current_frame),
            InstructionInfo::lor => lor(&current_frame),
            InstructionInfo::lrem => unimplemented!(),
            InstructionInfo::lreturn => lreturn(state, &current_frame),
            InstructionInfo::lshl => lshl(current_frame.clone()),
            InstructionInfo::lshr => lshr(current_frame.clone()),
            InstructionInfo::lstore(n) => lstore(&current_frame, n as usize),
            InstructionInfo::lstore_0 => lstore(&current_frame, 0),
            InstructionInfo::lstore_1 => lstore(&current_frame, 1),
            InstructionInfo::lstore_2 => lstore(&current_frame, 2),
            InstructionInfo::lstore_3 => lstore(&current_frame, 3),
            InstructionInfo::lsub => lsub(&current_frame),
            InstructionInfo::lushr => unimplemented!(),
            InstructionInfo::lxor => unimplemented!(),
            InstructionInfo::monitorenter => {
                current_frame.pop().unwrap_object_nonnull();
                /*unimplemented for now todo*/
            }
            InstructionInfo::monitorexit => {
                current_frame.pop().unwrap_object_nonnull();
                /*unimplemented for now todo*/
            }
            InstructionInfo::multianewarray(cp) => multi_a_new_array(state, &current_frame, cp),
            InstructionInfo::new(cp) => new(state, &current_frame, cp as usize),
            InstructionInfo::newarray(a_type) => newarray(&current_frame, a_type),
            InstructionInfo::nop => {},
            InstructionInfo::pop => pop(&current_frame),
            InstructionInfo::pop2 => pop2(&current_frame),
            InstructionInfo::putfield(cp) => putfield(state, &current_frame, cp),
            InstructionInfo::putstatic(cp) => putstatic(state, &current_frame, cp),
            InstructionInfo::ret(_) => unimplemented!(),
            InstructionInfo::return_ => return_(state),
            InstructionInfo::saload => unimplemented!(),
            InstructionInfo::sastore => unimplemented!(),
            InstructionInfo::sipush(val) => sipush(&current_frame, val),
            InstructionInfo::swap => unimplemented!(),
            InstructionInfo::tableswitch(switch) => tableswitch(switch, &current_frame),
            InstructionInfo::wide(_) => unimplemented!(),
            InstructionInfo::EndOfCode => unimplemented!(),
        }
        if state.throw.is_some() {
            let throw_class = state.throw.as_ref().unwrap().unwrap_normal_object().class_pointer.clone();
            for excep_table in &code.exception_table {
                if excep_table.start_pc as usize <= *current_frame.pc.borrow() && *current_frame.pc.borrow() < (excep_table.end_pc as usize) {//todo exclusive
                    if excep_table.catch_type == 0 {
                        //todo dup
                        current_frame.push(JavaValue::Object(state.throw.clone()));
                        state.throw = None;
                        current_frame.pc.replace(excep_table.handler_pc as usize);
                        println!("Caught Exception:{}", class_name(&throw_class.classfile).get_referred_name());
                        break;
                    } else {
                        let catch_runtime_name = current_frame.class_pointer.classfile.extract_class_from_constant_pool_name(excep_table.catch_type);
                        let catch_class = check_inited_class(state, &ClassName::Str(catch_runtime_name), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
                        if inherits_from(state, &throw_class, &catch_class) {
                            current_frame.push(JavaValue::Object(state.throw.clone()));
                            state.throw = None;
                            current_frame.pc.replace(excep_table.handler_pc as usize);
                            println!("Caught Exception:{}", class_name(&throw_class.classfile).get_referred_name());
                            break;
                        }
                    }
                }
            }
            if state.throw.is_some() {
                //need to propogate to caller
                break;
            }
        } else {

            //todo need to figure out where return res ends up on next stack
            let offset = *current_frame.pc_offset.borrow();
            let mut pc = *current_frame.pc.borrow();
            if offset > 0 {
                pc += offset as usize;
            } else {
                pc -= (-offset) as usize;//todo perhaps i don't have to do this bs if I use u64 instead of usize
            }
            current_frame.pc.replace(pc);
        }
    }
    if &meth_name == "getMethodOrFieldType"{
        dbg!(&current_frame.last_call_stack.as_ref().unwrap().operand_stack.borrow().last().unwrap());
    }
    println!("CALL END:{} {} {}", &class_name_, meth_name, current_depth);
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
        let loaded_super = loader_arc.load_class(loader_arc.clone(), &super_name.unwrap(), bl.clone()).unwrap();
        default_init_fields(loader_arc.clone(), object_pointer.clone(), &loaded_super.backing_class(), bl);
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
            let descriptor = parse_field_descriptor(descriptor_str.as_str()).unwrap();
            let type_ = descriptor.field_type;
            let val = default_value(type_);
            {
                object_pointer.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert(name, val);
            }
        }
    }
}

pub fn run_constructor(state: &mut InterpreterState, frame: Rc<StackEntry>, target_classfile: Arc<RuntimeClass>, mut full_args: Vec<JavaValue>, descriptor: String) {
    let (i, m) = target_classfile.classfile.lookup_method("<init>".to_string(), descriptor.clone()).unwrap();
    let md = parse_method_descriptor(descriptor.as_str()).unwrap();
    let this_ptr = full_args[0].clone();
    let actual_args = &mut full_args[1..];
    frame.push(this_ptr);
    for arg in actual_args {
        frame.push(arg.clone());
    }
    //todo this should be invoke special
    invoke_virtual_method_i(state, frame, md, target_classfile.clone(), i, m);
}
