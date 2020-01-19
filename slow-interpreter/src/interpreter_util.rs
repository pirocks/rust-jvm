use crate::{InterpreterState, CallStackEntry};
use rust_jvm_common::utils::{code_attribute, extract_string_from_utf8, method_name};
use classfile_parser::code::CodeParserContext;
use classfile_parser::code::parse_instruction;
use rust_jvm_common::classfile::{InstructionInfo, ConstantKind, ACC_STATIC, ConstantInfo, String_, Class};
use verification::verifier::instructions::special::extract_field_descriptor;
use crate::runtime_class::prepare_class;
use crate::runtime_class::initialize_class;
use std::sync::Arc;
use rust_jvm_common::classnames::{ClassName, class_name};
use rust_jvm_common::loading::Loader;
use std::rc::Rc;
use crate::instructions::invoke::{run_invoke_static, invoke_special, find_target_method, invoke_virtual};
use runtime_common::java_values::{JavaValue, VecPointer, ObjectPointer, default_value, Object};
use runtime_common::runtime_class::RuntimeClass;
use rust_jni::LibJavaLoading;
use classfile_parser::types::{parse_field_descriptor, MethodDescriptor};
use rust_jvm_common::unified_types::{ArrayType, ParsedType};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::hash;

//todo jni should really live in interpreter state
pub fn check_inited_class(
    state: &mut InterpreterState,
    class_name: &ClassName,
    current_frame: Rc<CallStackEntry>,
    loader_arc: Arc<dyn Loader + Sync + Send>,
    jni: &LibJavaLoading,
) -> Arc<RuntimeClass> {
    //todo racy/needs sychronization
    if !state.initialized_classes.read().unwrap().contains_key(&class_name) {
        let bl = state.bootstrap_loader.clone();
        let target_classfile = loader_arc.clone().load_class(loader_arc.clone(), &class_name, bl).unwrap();
        let prepared = prepare_class(target_classfile.clone(), loader_arc.clone());
        state.initialized_classes.write().unwrap().insert(class_name.clone(), Arc::new(prepared));//must be before, otherwise infinite recurse
        let inited_target = initialize_class(prepare_class(target_classfile, loader_arc.clone()), state, current_frame, jni);
        state.initialized_classes.write().unwrap().insert(class_name.clone(), inited_target);
    }
    state.initialized_classes.read().unwrap().get(class_name).unwrap().clone()
}

pub fn run_function(
    state: &mut InterpreterState,
    current_frame: Rc<CallStackEntry>,
    jni: &LibJavaLoading,
) {
    let methods = &current_frame.class_pointer.classfile.methods;
    let method = &methods[current_frame.method_i as usize];
    let code = code_attribute(method).unwrap();

    assert!(!state.function_return);
    while !state.terminate && !state.function_return && !state.throw {
        let (instruct, instruction_size) = {
            let current = &code.code_raw[*current_frame.pc.borrow()..];
            let mut context = CodeParserContext { offset: 0, iter: current.iter() };
//            dbg!(context.offset);
//            dbg!(&current);
            (parse_instruction(&mut context).unwrap().clone(), context.offset)
        };
        current_frame.pc_offset.replace(instruction_size as isize);
        match instruct {
            InstructionInfo::aaload => unimplemented!(),
            InstructionInfo::aastore => unimplemented!(),
            InstructionInfo::aconst_null => {
                current_frame.operand_stack.borrow_mut().push(JavaValue::Object(None))
            }
            InstructionInfo::aload(_) => unimplemented!(),
            InstructionInfo::aload_0 => aload(&current_frame, 0),
            InstructionInfo::aload_1 => aload(&current_frame, 1),
            InstructionInfo::aload_2 => aload(&current_frame, 2),
            InstructionInfo::aload_3 => aload(&current_frame, 3),
            InstructionInfo::anewarray(_cp) => {
                let len = match current_frame.operand_stack.borrow_mut().pop().unwrap() {
                    JavaValue::Int(i) => i,
                    _ => panic!()
                };
                if len == 0 {
                    current_frame.operand_stack.borrow_mut().push(JavaValue::Array(Some(VecPointer::new(len as usize))))
                } else {
                    unimplemented!()
                }
            }
            InstructionInfo::areturn => {
                let res = current_frame.operand_stack.borrow_mut().pop().unwrap();
                state.function_return = true;
                current_frame.last_call_stack.as_ref().unwrap().operand_stack.borrow_mut().push(res);
            }
            InstructionInfo::arraylength => {
                let array = current_frame.operand_stack.borrow_mut().pop().unwrap();
                match array {
                    JavaValue::Array(a) => {
                        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(a.unwrap().object.len() as i32));
                    }
                    _ => panic!()
                }
            }
            InstructionInfo::astore(_) => unimplemented!(),
            InstructionInfo::astore_0 => unimplemented!(),
            InstructionInfo::astore_1 => astore(&current_frame, 1),
            InstructionInfo::astore_2 => astore(&current_frame, 2),
            InstructionInfo::astore_3 => unimplemented!(),
            InstructionInfo::athrow => unimplemented!(),
            InstructionInfo::baload => unimplemented!(),
            InstructionInfo::bastore => unimplemented!(),
            InstructionInfo::bipush(b) => {
                current_frame.operand_stack.borrow_mut().push(JavaValue::Int(b as i32))
            }
            InstructionInfo::caload => unimplemented!(),
            InstructionInfo::castore => unimplemented!(),
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
            InstructionInfo::dreturn => unimplemented!(),
            InstructionInfo::dstore(_) => unimplemented!(),
            InstructionInfo::dstore_0 => unimplemented!(),
            InstructionInfo::dstore_1 => unimplemented!(),
            InstructionInfo::dstore_2 => unimplemented!(),
            InstructionInfo::dstore_3 => unimplemented!(),
            InstructionInfo::dsub => unimplemented!(),
            InstructionInfo::dup => {
                let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
                current_frame.operand_stack.borrow_mut().push(val.clone());
                current_frame.operand_stack.borrow_mut().push(val.clone());
            }
            InstructionInfo::dup_x1 => unimplemented!(),
            InstructionInfo::dup_x2 => unimplemented!(),
            InstructionInfo::dup2 => unimplemented!(),
            InstructionInfo::dup2_x1 => unimplemented!(),
            InstructionInfo::dup2_x2 => unimplemented!(),
            InstructionInfo::f2d => unimplemented!(),
            InstructionInfo::f2i => unimplemented!(),
            InstructionInfo::f2l => unimplemented!(),
            InstructionInfo::fadd => unimplemented!(),
            InstructionInfo::faload => unimplemented!(),
            InstructionInfo::fastore => unimplemented!(),
            InstructionInfo::fcmpg => unimplemented!(),
            InstructionInfo::fcmpl => unimplemented!(),
            InstructionInfo::fconst_0 => unimplemented!(),
            InstructionInfo::fconst_1 => unimplemented!(),
            InstructionInfo::fconst_2 => unimplemented!(),
            InstructionInfo::fdiv => unimplemented!(),
            InstructionInfo::fload(_) => unimplemented!(),
            InstructionInfo::fload_0 => unimplemented!(),
            InstructionInfo::fload_1 => unimplemented!(),
            InstructionInfo::fload_2 => unimplemented!(),
            InstructionInfo::fload_3 => unimplemented!(),
            InstructionInfo::fmul => unimplemented!(),
            InstructionInfo::fneg => unimplemented!(),
            InstructionInfo::frem => unimplemented!(),
            InstructionInfo::freturn => unimplemented!(),
            InstructionInfo::fstore(_) => unimplemented!(),
            InstructionInfo::fstore_0 => unimplemented!(),
            InstructionInfo::fstore_1 => unimplemented!(),
            InstructionInfo::fstore_2 => unimplemented!(),
            InstructionInfo::fstore_3 => unimplemented!(),
            InstructionInfo::fsub => unimplemented!(),
            InstructionInfo::getfield(cp) => {
                let classfile = &current_frame.class_pointer.classfile;
                let loader_arc = &current_frame.class_pointer.loader;
                let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
                dbg!(&field_name);
                dbg!(method_name(classfile, &classfile.methods[current_frame.method_i as usize]));
                let object_ref = current_frame.operand_stack.borrow_mut().pop().unwrap();
                match object_ref {
                    JavaValue::Object(o) => {
                        dbg!(&o);
                        dbg!(&field_name);
//                        if(class_name(&o.clone().unwrap().object.class_pointer.classfile).get_referred_name()){
//
//                        }
                        let res = o.unwrap().object.fields.borrow().get(field_name.as_str()).unwrap().clone();
                        current_frame.operand_stack.borrow_mut().push(res);
                    }
                    _ => panic!(),
                }
            }
            InstructionInfo::getstatic(cp) => {
                //todo make sure class pointer is updated correctly

                let classfile = &current_frame.class_pointer.classfile;
                let loader_arc = &current_frame.class_pointer.loader;
                let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
                let target_classfile = check_inited_class(state, &field_class_name, current_frame.clone(), loader_arc.clone(), jni);
                let field_value = target_classfile.static_vars.borrow().get(&field_name).unwrap().clone();
                let mut stack = current_frame.operand_stack.borrow_mut();
                stack.push(field_value);
            }
            InstructionInfo::goto_(_) => unimplemented!(),
            InstructionInfo::goto_w(_) => unimplemented!(),
            InstructionInfo::i2b => unimplemented!(),
            InstructionInfo::i2c => unimplemented!(),
            InstructionInfo::i2d => unimplemented!(),
            InstructionInfo::i2f => unimplemented!(),
            InstructionInfo::i2l => unimplemented!(),
            InstructionInfo::i2s => unimplemented!(),
            InstructionInfo::iadd => unimplemented!(),
            InstructionInfo::iaload => unimplemented!(),
            InstructionInfo::iand => unimplemented!(),
            InstructionInfo::iastore => unimplemented!(),
            InstructionInfo::iconst_m1 => unimplemented!(),
            InstructionInfo::iconst_0 => {
                current_frame.operand_stack.borrow_mut().push(JavaValue::Int(0))
            }
            InstructionInfo::iconst_1 => {
                current_frame.operand_stack.borrow_mut().push(JavaValue::Int(1))
            }
            InstructionInfo::iconst_2 => unimplemented!(),
            InstructionInfo::iconst_3 => unimplemented!(),
            InstructionInfo::iconst_4 => unimplemented!(),
            InstructionInfo::iconst_5 => unimplemented!(),
            InstructionInfo::idiv => unimplemented!(),
            InstructionInfo::if_acmpeq(_) => unimplemented!(),
            InstructionInfo::if_acmpne(_) => unimplemented!(),
            InstructionInfo::if_icmpeq(_) => unimplemented!(),
            InstructionInfo::if_icmpne(_) => unimplemented!(),
            InstructionInfo::if_icmplt(_) => unimplemented!(),
            InstructionInfo::if_icmpge(_) => unimplemented!(),
            InstructionInfo::if_icmpgt(_) => unimplemented!(),
            InstructionInfo::if_icmple(_) => unimplemented!(),
            InstructionInfo::ifeq(_) => unimplemented!(),
            InstructionInfo::ifne(_) => unimplemented!(),
            InstructionInfo::iflt(_) => unimplemented!(),
            InstructionInfo::ifge(offset) => {
                let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
                let succeeds = match val {
                    JavaValue::Int(i) => i >= 0,
                    _ => panic!()
                };
                current_frame.pc_offset.replace(offset as isize);
                dbg!(&current_frame.pc);
                dbg!(&current_frame.pc_offset);
                dbg!(&offset);
            }
            InstructionInfo::ifgt(_) => unimplemented!(),
            InstructionInfo::ifle(_) => unimplemented!(),
            InstructionInfo::ifnonnull(offset) => {
                let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
                let succeeds = match val {
                    JavaValue::Object(o) => o.is_some(),
                    _ => panic!()
                };
                current_frame.pc_offset.replace(offset as isize);
            }
            InstructionInfo::ifnull(offset) => {
                let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
                let succeeds = match val {
                    JavaValue::Object(o) => o.is_none(),
                    _ => panic!()
                };
                current_frame.pc_offset.replace(offset as isize);
            }
            InstructionInfo::iinc(_) => unimplemented!(),
            InstructionInfo::iload(_) => unimplemented!(),
            InstructionInfo::iload_0 => unimplemented!(),
            InstructionInfo::iload_1 => iload(&current_frame, 1),
            InstructionInfo::iload_2 => iload(&current_frame, 2),
            InstructionInfo::iload_3 => unimplemented!(),
            InstructionInfo::imul => unimplemented!(),
            InstructionInfo::ineg => unimplemented!(),
            InstructionInfo::instanceof(_) => unimplemented!(),
            InstructionInfo::invokedynamic(_) => unimplemented!(),
            InstructionInfo::invokeinterface(_) => unimplemented!(),
            InstructionInfo::invokespecial(cp) => invoke_special(state, &current_frame, jni, cp),
            InstructionInfo::invokestatic(cp) => run_invoke_static(state, current_frame.clone(), cp, jni),
            InstructionInfo::invokevirtual(cp) => invoke_virtual(state, current_frame.clone(), cp, jni),
            InstructionInfo::ior => unimplemented!(),
            InstructionInfo::irem => unimplemented!(),
            InstructionInfo::ireturn => unimplemented!(),
            InstructionInfo::ishl => unimplemented!(),
            InstructionInfo::ishr => unimplemented!(),
            InstructionInfo::istore(_) => unimplemented!(),
            InstructionInfo::istore_0 => unimplemented!(),
            InstructionInfo::istore_1 => unimplemented!(),
            InstructionInfo::istore_2 => unimplemented!(),
            InstructionInfo::istore_3 => unimplemented!(),
            InstructionInfo::isub => unimplemented!(),
            InstructionInfo::iushr => unimplemented!(),
            InstructionInfo::ixor => unimplemented!(),
            InstructionInfo::jsr(_) => unimplemented!(),
            InstructionInfo::jsr_w(_) => unimplemented!(),
            InstructionInfo::l2d => unimplemented!(),
            InstructionInfo::l2f => unimplemented!(),
            InstructionInfo::l2i => unimplemented!(),
            InstructionInfo::ladd => unimplemented!(),
            InstructionInfo::laload => unimplemented!(),
            InstructionInfo::land => unimplemented!(),
            InstructionInfo::lastore => unimplemented!(),
            InstructionInfo::lcmp => unimplemented!(),
            InstructionInfo::lconst_0 => unimplemented!(),
            InstructionInfo::lconst_1 => unimplemented!(),
            InstructionInfo::ldc(cp) => {
                let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
                let pool_entry = &constant_pool[cp as usize];
                match &pool_entry.kind {
                    ConstantKind::String(s) => load_string_constant(state, &current_frame, jni, &constant_pool, &s),
                    ConstantKind::Class(c) => load_class_constant(state, &current_frame, jni, constant_pool, &c),
                    _ => unimplemented!()
                }
            }
            InstructionInfo::ldc_w(_) => unimplemented!(),
            InstructionInfo::ldc2_w(_) => unimplemented!(),
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
            InstructionInfo::lshl => unimplemented!(),
            InstructionInfo::lshr => unimplemented!(),
            InstructionInfo::lstore(_) => unimplemented!(),
            InstructionInfo::lstore_0 => unimplemented!(),
            InstructionInfo::lstore_1 => unimplemented!(),
            InstructionInfo::lstore_2 => unimplemented!(),
            InstructionInfo::lstore_3 => unimplemented!(),
            InstructionInfo::lsub => unimplemented!(),
            InstructionInfo::lushr => unimplemented!(),
            InstructionInfo::lxor => unimplemented!(),
            InstructionInfo::monitorenter => {/*unimplemented for now todo*/},
            InstructionInfo::monitorexit => {/*unimplemented for now todo*/},
            InstructionInfo::multianewarray(_) => unimplemented!(),
            InstructionInfo::new(cp) => new(state, jni, &current_frame, cp as usize),
            InstructionInfo::newarray(_) => unimplemented!(),
            InstructionInfo::nop => unimplemented!(),
            InstructionInfo::pop => unimplemented!(),
            InstructionInfo::pop2 => unimplemented!(),
            InstructionInfo::putfield(cp) => {
                let classfile = &current_frame.class_pointer.classfile;
                let loader_arc = &current_frame.class_pointer.loader;
                let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
                let target_classfile = check_inited_class(state, &field_class_name, current_frame.clone(), loader_arc.clone(), jni);
                let mut stack = current_frame.operand_stack.borrow_mut();
                let val = stack.pop().unwrap();
                let object_ref = stack.pop().unwrap();
                match object_ref {
                    JavaValue::Object(o) => {
                        o.unwrap().object.fields.borrow_mut().insert(field_name, val);
                    }
                    _ => {
                        dbg!(object_ref);
                        panic!()
                    }
                }
            }
            InstructionInfo::putstatic(cp) => {
                let classfile = &current_frame.class_pointer.classfile;
                let loader_arc = &current_frame.class_pointer.loader;
                let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
                let target_classfile = check_inited_class(state, &field_class_name, current_frame.clone(), loader_arc.clone(), jni);
                let mut stack = current_frame.operand_stack.borrow_mut();
                let field_value = stack.pop().unwrap();
                target_classfile.static_vars.borrow_mut().insert(field_name, field_value);
            }
            InstructionInfo::ret(_) => unimplemented!(),
            InstructionInfo::return_ => {
                state.function_return = true;
            }
            InstructionInfo::saload => unimplemented!(),
            InstructionInfo::sastore => unimplemented!(),
            InstructionInfo::sipush(_) => unimplemented!(),
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
    }
}

fn aload(current_frame: &Rc<CallStackEntry>, n: usize) -> () {
    let ref_ = current_frame.local_vars.borrow()[n].clone();
    match ref_.clone() {
        JavaValue::Object(_) | JavaValue::Array(_) => {}
        _ => {
            dbg!(ref_);
            panic!()
        }
    }
    current_frame.operand_stack.borrow_mut().push(ref_);
}

fn astore(current_frame: &Rc<CallStackEntry>, n: usize) -> () {
    let object_ref = current_frame.operand_stack.borrow_mut().pop().unwrap();
    match object_ref.clone() {
        JavaValue::Object(_) => {}
        _ => panic!(),
    }
    current_frame.local_vars.borrow_mut()[n] = object_ref;
}

fn iload(current_frame: &Rc<CallStackEntry>, n: usize) {
    let java_val = &current_frame.local_vars.borrow()[n];
    match java_val {
        JavaValue::Int(_) => {}
        _ => {
            panic!()
        }
    }
    current_frame.operand_stack.borrow_mut().push(java_val.clone())
}

thread_local! {

}

fn load_class_constant(state: &mut InterpreterState, current_frame: &Rc<CallStackEntry>, jni: &LibJavaLoading, constant_pool: &Vec<ConstantInfo>, c: &Class) {
    let res_class_name = extract_string_from_utf8(&constant_pool[c.name_index as usize]);
    let java_lang_class = ClassName::Str("java/lang/Class".to_string());
    let java_lang_class_loader = ClassName::Str("java/lang/ClassLoader".to_string());
    let current_loader = current_frame.class_pointer.loader.clone();
    let class_class = check_inited_class(state, &java_lang_class, current_frame.clone(), current_loader.clone(), jni);
    let class_loader_class = check_inited_class(state, &java_lang_class_loader, current_frame.clone(), current_loader.clone(), jni);
    //the above would only be required for higher jdks where a class loader obect is part of Class.
    //as it stands we can just push to operand stack
    push_new_object(current_frame.clone(), &class_class);
    let object = current_frame.operand_stack.borrow_mut().pop().unwrap();
    match object.clone() {
        JavaValue::Object(o) => {
            let boostrap_loader_object = Object {
                gc_reachable: true,
                fields: RefCell::new(HashMap::new()),
                class_pointer: class_loader_class.clone(),
                bootstrap_loader: true,
            };
            let bootstrap_arc = Arc::new(boostrap_loader_object);
            let bootstrap_loader_ptr = ObjectPointer { object: bootstrap_arc.clone() }.into();
            let bootstrap_class_loader = JavaValue::Object(bootstrap_loader_ptr);
            bootstrap_arc.fields.borrow_mut().insert("assertionLock".to_string(), bootstrap_class_loader.clone());//itself...
            bootstrap_arc.fields.borrow_mut().insert("classAssertionStatus".to_string(), JavaValue::Object(None));
//            fields.insert("assertionLock".to_string(),bootstrap_class_loader.clone());
            o.unwrap().object.fields.borrow_mut().insert("classLoader".to_string(), bootstrap_class_loader);
        }
        _ => panic!(),
    }
    current_frame.operand_stack.borrow_mut().push(object);
}

fn load_string_constant(state: &mut InterpreterState, current_frame: &Rc<CallStackEntry>, jni: &LibJavaLoading, constant_pool: &Vec<ConstantInfo>, s: &String_) {
    let res_string = extract_string_from_utf8(&constant_pool[s.string_index as usize]);
    let java_lang_string = ClassName::Str("java/lang/String".to_string());
    let current_loader = current_frame.class_pointer.loader.clone();
    let string_class = check_inited_class(state, &java_lang_string, current_frame.clone(), current_loader.clone(), jni);
    let str_as_vec = res_string.into_bytes().clone();
    let chars: Vec<JavaValue> = str_as_vec.iter().map(|x| { JavaValue::Char(*x as char) }).collect();
    push_new_object(current_frame.clone(), &string_class);
    let mut args = vec![current_frame.operand_stack.borrow_mut().pop().unwrap()];
    args.push(JavaValue::Array(Some(VecPointer { object: Arc::new(chars) })));
    let char_array_type = ParsedType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::CharType) });
    let expected_descriptor = MethodDescriptor { parameter_types: vec![char_array_type], return_type: ParsedType::VoidType };
    let (constructor_i, _constructor) = find_target_method(current_loader.clone(), "<init>".to_string(), &expected_descriptor, &string_class);
    let next_entry = CallStackEntry {
        last_call_stack: Some(current_frame.clone()),
        class_pointer: string_class,
        method_i: constructor_i as u16,
        local_vars: args.into(),
        operand_stack: vec![].into(),
        pc: 0.into(),
        pc_offset: 0.into(),
    };
    run_function(state, Rc::new(next_entry), jni);
    if state.terminate || state.throw {
        unimplemented!()
    }
    if state.function_return {
        state.function_return = false;
    }
    unimplemented!()
}

pub fn new(state: &mut InterpreterState, jni: &LibJavaLoading, current_frame: &Rc<CallStackEntry>, cp: usize) -> () {
    let loader_arc = &current_frame.class_pointer.loader;
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let class_name_index = match &constant_pool[cp as usize].kind {
        ConstantKind::Class(c) => c.name_index,
        _ => panic!()
    };
    let target_class_name = ClassName::Str(extract_string_from_utf8(&constant_pool[class_name_index as usize]));
    let target_classfile = check_inited_class(state, &target_class_name, current_frame.clone(), loader_arc.clone(), jni);
    push_new_object(current_frame.clone(), &target_classfile);
}

fn push_new_object(current_frame: Rc<CallStackEntry>, target_classfile: &Arc<RuntimeClass>) {
    let loader_arc = &current_frame.class_pointer.loader;
    let object_pointer = ObjectPointer::new(target_classfile.clone());
    let new_obj = JavaValue::Object(Some(object_pointer.clone()));
    let classfile = &target_classfile.classfile;
    for field in &classfile.fields {
        if field.access_flags & ACC_STATIC == 0 {
            let name = extract_string_from_utf8(&classfile.constant_pool[field.name_index as usize]);
            let descriptor_str = extract_string_from_utf8(&classfile.constant_pool[field.descriptor_index as usize]);
            let descriptor = parse_field_descriptor(loader_arc, descriptor_str.as_str()).unwrap();
            let type_ = descriptor.field_type;
            let val = default_value(type_);
            object_pointer.object.fields.borrow_mut().insert(name, val);
        }
    }
    current_frame.operand_stack.borrow_mut().push(new_obj);
}


//use ::std::mem::transmute;
//
//use crate::InterpreterState;
//
//pub const EXECUTION_ERROR: &str = "Fatal Error, when executing, this is a bug.";
//
//#[macro_export]
//macro_rules! null_pointer_check {
//($var_name:ident) => {
//    if $var_name == 0 {
//            unimplemented!("handle null pointers exceptions")
//        }
//};
//}
//#[macro_export]
//macro_rules! array_out_of_bounds_check {
//($index:expr,$array_length:ident) => {if ($index as u32) >= ($array_length as u32) {
//        unimplemented!("handle array out of bounds exceptions")
//    }};
//}
//#[macro_export]
//macro_rules! load {
//($type_:ident,$state:ident) => {
//    use ::interpreter::interpreter_util::{EXECUTION_ERROR, pop_long};
//    let index = $state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = pop_long($state);
//    use ::null_pointer_check;
//    null_pointer_check!(array_ref);
//    let array_elem:$type_ = unsafe {
//        let array_64: *mut u64 = ::std::mem::transmute(array_ref);
//        let array_length: u64 = *array_64.offset(-1);
//        let array_type:* mut $type_ = array_ref as * mut $type_;
//        use ::array_out_of_bounds_check;
//        array_out_of_bounds_check!(index as u64,array_length);
//        *(array_type.offset(index as isize)) as $type_
//    };
//    //todo this is more complicated in the u64 case
//    $state.operand_stack.push(array_elem as u32);
//};
//}
//
//#[macro_export]
//macro_rules! store {
//($type_:ident,$state:ident) => {
//    use ::interpreter::interpreter_util::{EXECUTION_ERROR, pop_long};
//    let value : $type_= $state.operand_stack.pop().expect(EXECUTION_ERROR) as $type_;
//    let index = $state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = pop_long($state);
//    use ::null_pointer_check;
//    null_pointer_check!(array_ref);
//    unsafe {
//        let array: *mut u64 = ::std::mem::transmute(array_ref);
//        let array_length: u64 = *array.offset(-1);
//        use ::array_out_of_bounds_check;
//        array_out_of_bounds_check!(index as u64,array_length);
//        let array_type : *mut $type_ = array_ref as *mut $type_;
//        *(array_type.offset(index as isize)) = value;
//    }
//};
//}
//
//pub fn store_i64(state: &mut InterpreterState){
//    let value  = pop_long(state);
//    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = pop_long(state);
//    null_pointer_check!(array_ref);
//    unsafe {
//        let array: *mut i64 = transmute(array_ref);
//        let array_length: i64 = *array.offset(-1);
//        array_out_of_bounds_check!(index as u64,array_length);
//        let array_type : *mut i64 = array_ref as *mut i64;
//        *(array_type.offset(index as isize)) = value;
//    }
//}
//
//pub fn load_u64(state: &mut InterpreterState){
//    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = pop_long(state);
//    null_pointer_check!(array_ref);
//    let array_elem: i64 = unsafe {
//        let array_64: *mut i64 = transmute(array_ref);
//        let array_length: i64 = *array_64.offset(-1);
//        let array_type:* mut i64 = array_ref as *mut i64;
//        array_out_of_bounds_check!(index as u64,array_length);
//        *(array_type.offset(index as isize))
//    };
//    push_long(array_elem,state);
//}
//
//pub fn pop_long(state: &mut InterpreterState) -> i64 {
//    let lower = state.operand_stack.pop().expect(EXECUTION_ERROR) as u64;
//    let upper = state.operand_stack.pop().expect(EXECUTION_ERROR) as u64;
//    return unsafe { transmute((upper << 32) | lower) }
//
//}
//
//pub fn push_long(to_push: i64, state: &mut InterpreterState) {
//    state.operand_stack.push( (to_push >> 32) as u32);
//    state.operand_stack.push( ((to_push << 32) >> 32) as u32);
//}
//
//pub fn push_byte(to_push: i8, state: &mut InterpreterState) {
//    state.operand_stack.push(to_push as u32)
//}
//
//pub fn pop_byte(state: &mut InterpreterState) -> i8 {
//    return state.operand_stack.pop().expect(EXECUTION_ERROR) as i8;
//}
//
//pub fn push_char(to_push: u16, state: &mut InterpreterState) {
//    state.operand_stack.push(to_push as u32)
//}
//
//pub fn pop_char(state: &mut InterpreterState) -> u16 {
//    return state.operand_stack.pop().expect(EXECUTION_ERROR) as u16;
//}
//
//pub fn push_short(to_push: i16, state: &mut InterpreterState) {
//    state.operand_stack.push(to_push as u32)
//}
//
//pub fn pop_short(state: &mut InterpreterState) -> i16 {
//    return state.operand_stack.pop().expect(EXECUTION_ERROR) as i16;
//}
//
//
//pub fn push_int(to_push: i32, state: &mut InterpreterState) {
//    state.operand_stack.push(unsafe { transmute(to_push) })
//}
//
//pub fn pop_int(state: &mut InterpreterState) -> i32 {
//    return unsafe { transmute(state.operand_stack.pop().expect(EXECUTION_ERROR)) };
//}
//
//
//pub fn push_float(to_push: f32, state: &mut InterpreterState) {
//    state.operand_stack.push(unsafe { ::std::mem::transmute(to_push) })
//}
//
//pub fn pop_float(state: &mut InterpreterState) -> f32 {
//    let value = state.operand_stack.pop().expect(EXECUTION_ERROR) as u32;
//    return unsafe { transmute(value) }
//}
//
//
//pub fn push_double(to_push: f64, state: &mut InterpreterState) {
//    push_long(unsafe { transmute(to_push) }, state)
//}
//
//pub fn pop_double(state: &mut InterpreterState) -> f64 {
//    let value = pop_long(state);
//    return unsafe {
//        ::std::mem::transmute(value)
//    }
//}
//
//pub fn store_n_32(state: &mut InterpreterState, n: u64) {
//    let reference = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    state.local_vars[n as usize] = reference as u32;
//}
//
//
//pub fn store_n_64(state: &mut InterpreterState, n: u64) {
//    let reference = pop_long(state);
//    state.local_vars[n as usize] = reference as u32;
//    state.local_vars[(n + 1) as usize] = (reference >> 32) as u32;//todo is this really the correct order
//}
//
//pub fn load_n_32(state: &mut InterpreterState, n: u64) {
//    let reference = state.local_vars[n as usize];
//    state.operand_stack.push(reference as u32)
//}
//
//pub fn load_n_64(state: &mut InterpreterState, n: u64) {
//    let least_significant = state.local_vars[n as usize];
//    let most_significant = state.local_vars[(n + 1) as usize];
//    state.operand_stack.push(most_significant );
//    state.operand_stack.push(least_significant );
//}
//
//
/*
//pub(crate) fn do_bipush(state: &mut InterpreterState) -> () {
//    let byte = pop_int(state) as i8;
//    push_int(byte as i32, state);
//}
//
//pub(crate) fn do_astore(code: &[u8], state: &mut InterpreterState) -> ! {
//    let index = code[1];
//    store_n_32(state, index as u64);
//    unimplemented!("Need to increase pc by 2");
//}
//
//pub(crate) fn do_anewarray(code: &[u8], state: &mut InterpreterState) -> ! {
//    let indexbyte1 = code[1] as u16;
//    let indexbyte2 = code[2] as u16;
//    let _index = (indexbyte1 << 8) | indexbyte2;
//    let _count = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    unimplemented!("Need to figure out how to get the constant pool in here.");
////    unimplemented!("Need to increase pc by 3");
//}
//
//pub(crate) fn do_aload(code: &[u8], state: &mut InterpreterState) -> ! {
//    let var_index = code[1];
//    load_n_64(state, var_index as u64);
//    unimplemented!("Need to increase pc by 2")
//}
//
//
//pub(crate) fn do_arraylength(state: &mut InterpreterState) -> () {
//    let array_ref = pop_long(state);
//    let length = unsafe {
//        let array: *mut i64 = transmute(array_ref);
//        *(array.offset(-1 as isize)) as i64
//    };
//    push_long(length,state)
//}
*/
//
//
//#[cfg(test)]
//pub mod tests{
//    use super::*;
//
//    #[test]
//    fn test_int_pop_push() {
//        let int_ = -654545864;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_int(int_,state);
//        assert_eq!(int_,pop_int(state));
//    }
//
//    #[test]
//    fn test_long_pop_push() {
//        let long_ = -654545864*435657687;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_long(long_,state);
//        assert_eq!(long_,pop_long(state));
//    }
//
//    #[test]
//    fn test_char_pop_push() {
//        let char_ = 'g';
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_char(char_ as u16, state);
//        assert_eq!(char_ as u16, pop_char(state));
//    }
//
//    #[test]
//    fn test_double_pop_push() {
//        let double_ = 0.4546545613512652;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_double(double_,state);
//        assert_eq!(double_,pop_double(state));
//    }
//
//
//    #[test]
//    fn test_float_pop_push() {
//        let float_ = -56.045f32;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_float(float_,state);
//        assert_eq!(float_,pop_float(state));
//    }
//
//    #[test]
//    fn test_byte_pop_push() {
//        let byte_  = -120i8;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_byte(byte_, state);//todo need to pop push i8
//        assert_eq!(byte_, pop_byte(state));
//    }
//
//}