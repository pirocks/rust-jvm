use crate::InterpreterState;
use std::rc::Rc;
use verification::verifier::instructions::branches::get_method_descriptor;
use rust_jvm_common::classfile::{ACC_NATIVE, ACC_STATIC, InvokeInterface};
use crate::interpreter_util::run_function;
use std::sync::Arc;
use rust_jvm_common::loading::LoaderArc;
use rust_jvm_common::classfile::MethodInfo;
use rust_jvm_common::classfile::ACC_ABSTRACT;
use crate::interpreter_util::check_inited_class;
use runtime_common::java_values::{JavaValue, Object, ArrayObject};
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::StackEntry;
use std::cell::Ref;
use crate::rust_jni::{call_impl, call, mangling, get_all_methods};
use std::borrow::Borrow;
use utils::lookup_method_parsed;
use rust_jvm_common::classnames::class_name;
use std::intrinsics::transmute;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use rust_jvm_common::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::view::ClassView;


pub fn invoke_special(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) -> () {
    let loader_arc = current_frame.class_pointer.loader.clone();
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp as usize, &ClassView::from(current_frame.class_pointer.classfile.clone()));
    let method_class_name = method_class_type.unwrap_class_type();
//    trace!("Call:{} {}", method_class_name.get_referred_name(), method_name.clone());
    let target_class = check_inited_class(state, &method_class_name, current_frame.clone().into(), loader_arc.clone());
    let (target_m_i, final_target_class) = find_target_method(state, loader_arc.clone(), method_name.clone(), &parsed_descriptor, target_class);
    let target_m = &final_target_class.classfile.methods[target_m_i];
    invoke_special_impl(state, current_frame, &parsed_descriptor, target_m_i, final_target_class.clone(), target_m);
//    if method_name == "<init>"{
//        dbg!(&current_frame.operand_stack);
//    }
}

pub fn invoke_special_impl(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, parsed_descriptor: &MethodDescriptor, target_m_i: usize, final_target_class: Arc<RuntimeClass>, target_m: &MethodInfo) -> () {
    if target_m.access_flags & ACC_NATIVE > 0 {
        run_native_method(state, current_frame.clone(), final_target_class, target_m_i);
    } else {
        let mut args = vec![];
//        dbg!(method_class_name.get_referred_name());
//        dbg!(&method_name);
        let max_locals = target_m.code_attribute().unwrap().max_locals;
        setup_virtual_args(current_frame, &parsed_descriptor, &mut args, max_locals);
        let next_entry = StackEntry {
            last_call_stack: Some(current_frame.clone()),
            class_pointer: final_target_class.clone(),
            method_i: target_m_i as u16,
            local_vars: args.into(),
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into(),
        };
//        dbg!(target_m.code_attribute());
        run_function(state, Rc::new(next_entry));
        if state.throw.is_some() || state.terminate {
            unimplemented!()
        }
        if state.function_return {
            state.function_return = false;
//        trace!("Exit:{} {}", method_class_name.get_referred_name(), method_name.clone());
        }
    }
}

fn resolved_class(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) -> Option<(Arc<RuntimeClass>, String, MethodDescriptor)> {
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &ClassView::from(classfile.clone()));
    let class_name_ = match class_name_type {
        PTypeView::Ref(r) => {
            match r{
                ReferenceTypeView::Class(c) => c,
                ReferenceTypeView::Array(_a) => {
                    if expected_method_name == "clone".to_string() {
                        //todo replace with proper native impl
                        let temp = current_frame.pop().unwrap_object().unwrap();
                        let to_clone_array = temp.unwrap_array();
                        current_frame.push(JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: to_clone_array.elems.clone(), elem_type: to_clone_array.elem_type.clone() })))));
                        return None;
                    } else {
                        unimplemented!();
                    }
                }
            }
        }
        _ => panic!()
    };
    //todo should I be trusting these descriptors, or should i be using the runtime class on top of the operant stack
    let resolved_class = check_inited_class(state, &class_name_, current_frame.clone().into(), loader_arc.clone());
    (resolved_class, expected_method_name, expected_descriptor).into()
}

pub fn invoke_virtual(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) {
    let (_resolved_class, method_name, expected_descriptor) = match resolved_class(state, current_frame.clone(), cp) {
        None => return,
        Some(o) => { o }
    };
    //The resolved method must not be an instance initialization method,or the class or interface initialization method (§2.9)
    if method_name == "<init>".to_string() ||
        method_name == "<clinit>".to_string() {
        panic!()//should have been caught by verifier
    }

    //If the resolved method is not signature polymorphic ( §2.9), thenthe invokevirtual instruction proceeds as follows.
    //we assume that it isn't signature polymorphic for now todo

    //Let C be the class of objectref.
    let this_pointer = {
        let operand_stack = current_frame.operand_stack.borrow();
        &operand_stack[operand_stack.len() - expected_descriptor.parameter_types.len() - 1].clone()
    };
    let c = this_pointer.unwrap_object().unwrap().unwrap_normal_object().class_pointer.clone();
    let all_methods = get_all_methods(state, current_frame.clone(), c.clone());
//    current_frame.print_stack_trace();
//    dbg!(class_name(&c.classfile));
//    dbg!(&method_name);
//    dbg!(&expected_descriptor);
    let (final_target_class, new_i) = all_methods.iter().find(|(c, m)| {
        let cur_method_info = &c.classfile.methods[*m];
        let cur_name = cur_method_info.method_name(&c.classfile);
        let desc_str = cur_method_info.descriptor_str(&c.classfile);
        let cur_desc = parse_method_descriptor(desc_str.as_str()).unwrap();
        let expected_name = &method_name;
        &cur_name == expected_name &&
            expected_descriptor.parameter_types == cur_desc.parameter_types &&
            !cur_method_info.is_static() &&
            !cur_method_info.is_abstract() //&&
//            !cur_method_info.is_native()
    }).unwrap();
    let final_classfile = &final_target_class.classfile;
    let target_method = &final_classfile.methods[*new_i];
    let final_descriptor = parse_method_descriptor( target_method.descriptor_str(&final_classfile).as_str()).unwrap();
    invoke_virtual_method_i(state, current_frame.clone(), final_descriptor, final_target_class.clone(), *new_i, target_method)
}

pub fn invoke_virtual_method_i(state: &mut InterpreterState, current_frame: Rc<StackEntry>, expected_descriptor: MethodDescriptor, target_class: Arc<RuntimeClass>, target_method_i: usize, target_method: &MethodInfo) -> () {
    invoke_virtual_method_i_impl(state, current_frame.clone(), expected_descriptor, target_class, target_method_i, target_method)
}

pub fn invoke_virtual_method_i_impl(
    state: &mut InterpreterState,
    current_frame: Rc<StackEntry>,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method_i: usize,
    target_method: &MethodInfo,
) -> () {
    if target_method.access_flags & ACC_NATIVE > 0 {
        run_native_method(state, current_frame.clone(), target_class, target_method_i)
    } else if target_method.access_flags & ACC_ABSTRACT == 0 {
        //todo this is wrong?
        let mut args = vec![];
        let max_locals = target_method.code_attribute().unwrap().max_locals;

        setup_virtual_args(&current_frame, &expected_descriptor, &mut args, max_locals);
        let next_entry = StackEntry {
            last_call_stack: Some(current_frame),
            class_pointer: target_class.clone(),
            method_i: target_method_i as u16,
            local_vars: args.into(),
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into(),
        };
        run_function(state, Rc::new(next_entry));
        if state.throw.is_some() || state.terminate {
            unimplemented!();
        }
        if state.function_return {
            state.function_return = false;
            return;
        }
    } else {
        panic!()
    }
}

//todo we should be going to this first imo. b/c as is we have correctness issues with overloaded impls?
pub fn actually_virtual(state: &mut InterpreterState, current_frame: Rc<StackEntry>, expected_descriptor: MethodDescriptor, target_class: &Arc<RuntimeClass>, target_method: &MethodInfo) -> () {
//    dbg!("Called actually virtual");
//    dbg!(class_name(&target_class.classfile).get_referred_name());
//    current_frame.print_stack_trace();
    let this_pointer = {
        let operand_stack = current_frame.operand_stack.borrow();
        &operand_stack[operand_stack.len() - expected_descriptor.parameter_types.len() - 1].clone()
    };
    let new_target_class = this_pointer.unwrap_object().unwrap().unwrap_normal_object().class_pointer.clone();
    assert_eq!(new_target_class.classfile.access_flags & ACC_ABSTRACT, 0);
//    dbg!(class_name(&new_target_class.classfile).get_referred_name());
//todo so this is incorrect due to subclassing of return value.
    let all_methods = get_all_methods(state, current_frame.clone(), new_target_class.clone());
    let (final_target_class, new_i) = all_methods.iter().find(|(c, m)| {
        let cur_method_info = &c.classfile.methods[*m];
        let cur_name = cur_method_info.method_name(&c.classfile);
        let desc_str = cur_method_info.descriptor_str(&c.classfile);
        let cur_desc = parse_method_descriptor( desc_str.as_str()).unwrap();
        let expected_name = target_method.method_name(&target_class.classfile);
//        if expected_name == cur_name{
//            dbg!(&expected_name);
//            dbg!(&cur_name);
//            dbg!(&expected_descriptor);
//            dbg!(&cur_desc);
//        }

        cur_name == expected_name &&
            expected_descriptor.parameter_types == cur_desc.parameter_types &&
            !cur_method_info.is_static() &&
            !cur_method_info.is_abstract() &&
            !cur_method_info.is_native()
    }).unwrap();
    invoke_virtual_method_i(state, current_frame, expected_descriptor, final_target_class.clone(), *new_i, &final_target_class.classfile.methods[*new_i])
}

pub fn setup_virtual_args(current_frame: &Rc<StackEntry>, expected_descriptor: &MethodDescriptor, args: &mut Vec<JavaValue>, max_locals: u16) {
    for _ in 0..max_locals {
        args.push(JavaValue::Top);
    }
    let mut i = 1;
//    dbg!(&expected_descriptor.parameter_types);
//    if args.len() == 5 {
//        dbg!(&current_frame.operand_stack);
//    }
    for _ in &expected_descriptor.parameter_types {
        let value = current_frame.pop();
        match value.clone() {
            JavaValue::Long(_) | JavaValue::Double(_) => {
                args[i] = JavaValue::Top;
                args[i + 1] = value;
                i += 2
            }
            _ => {
                args[i] = value;
                i += 1
            }
        };
        //todo does ordering end up correct
    }
//    dbg!(&args[1..(expected_descriptor.parameter_types.len() + 1)]);
    if expected_descriptor.parameter_types.len() != 0 {
        args[1..i].reverse();
    }
    args[0] = current_frame.pop();


    /*
      for _ in 0..max_locals {
        args.push(JavaValue::Top);
    }
    for i in 1..(parsed_descriptor.parameter_types.len() + 1) {
        args[i] = current_frame.pop();
        //todo does ordering end up correct
    }
    args[1..(parsed_descriptor.parameter_types.len() + 1)].reverse();
    args[0] = current_frame.pop();
//    dbg!(&args);
*/
}

pub fn run_invoke_static(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) {
//todo handle monitor enter and exit
//handle init cases
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &ClassView::from(classfile.clone()));
    let class_name = class_name_type.unwrap_class_type();
    let target_class = check_inited_class(state, &class_name, current_frame.clone().into(), loader_arc.clone());
    let (target_method_i, final_target_method) = find_target_method(state, loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);

    invoke_static_impl(state, current_frame, expected_descriptor, final_target_method.clone(), target_method_i, &final_target_method.classfile.methods[target_method_i]);
}

pub fn invoke_static_impl(
    state: &mut InterpreterState,
    current_frame: Rc<StackEntry>,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method_i: usize,
    target_method: &MethodInfo,
) -> () {
    let mut args = vec![];
    if target_method.access_flags & ACC_NATIVE == 0 {
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        assert!(target_method.access_flags & ACC_STATIC > 0);
        assert_eq!(target_method.access_flags & ACC_ABSTRACT, 0);
        let max_locals = target_method.code_attribute().unwrap().max_locals;
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        for _ in 0..max_locals {
            args.push(JavaValue::Top);
        }
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        for i in 0..expected_descriptor.parameter_types.len() {
            args[i] = current_frame.pop();
            //todo does ordering end up correct
        }
        args[0..expected_descriptor.parameter_types.len()].reverse();
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        let next_entry = StackEntry {
            last_call_stack: Some(current_frame),
            class_pointer: target_class,
            method_i: target_method_i as u16,
            local_vars: args.clone().into(),
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into(),
        };
//        dbg!(&args);
        run_function(state, Rc::new(next_entry));
        if state.throw.is_some() || state.terminate {
            return;
        }
        if state.function_return {
            state.function_return = false;
            return;
        }
    } else {
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        //only works for static void
        run_native_method(state, current_frame.clone(), target_class.clone(), target_method_i);
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
    }
}

pub fn find_target_method(
    state: &mut InterpreterState,
    loader_arc: LoaderArc,
    expected_method_name: String,
    parsed_descriptor: &MethodDescriptor,
    target_class: Arc<RuntimeClass>,
) -> (usize, Arc<RuntimeClass>) {
    //todo bug need to handle super class, issue with that is need frame/state.
    lookup_method_parsed(state, target_class, expected_method_name, parsed_descriptor, &loader_arc).unwrap()
}


pub fn run_native_method(
    state: &mut InterpreterState,
    frame: Rc<StackEntry>,
    class: Arc<RuntimeClass>,
    method_i: usize,
) {
    //todo only works for static void methods atm
    let classfile = &class.classfile;
    let method = &classfile.methods[method_i];
    assert!(method.access_flags & ACC_NATIVE > 0);
    let parsed = MethodDescriptor::from_legacy(method, classfile);
    let mut args = vec![];
    //todo should have some setup args functions
    if method.access_flags & ACC_STATIC > 0 {
        for _ in &parsed.parameter_types {
            args.push(frame.pop());
        }
        args.reverse();
    } else {
        if method.access_flags & ACC_NATIVE > 0 {
            for _ in &parsed.parameter_types {
                args.push(frame.pop());
            }
            args.reverse();
            args.insert(0, frame.pop());
        } else {
            panic!();
//            setup_virtual_args(&frame, &parsed, &mut args, (parsed.parameter_types.len() + 1) as u16)
        }
    }
    println!("CALL BEGIN NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
    if method.method_name(classfile) == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        frame.push(JavaValue::Boolean(false))
    } else if method.method_name(classfile) == "arraycopy".to_string() {
        system_array_copy(&mut args)
    } else {
        let result = if state.jni.registered_natives.borrow().contains_key(&class) &&
            state.jni.registered_natives.borrow().get(&class).unwrap().borrow().contains_key(&(method_i as u16))
        {
            //todo dup
//            dbg!(class_name(&class.classfile).get_referred_name());
//            dbg!(&class.classfile.methods[method_i].method_name(&class.classfile));
            let res_fn = {
                let reg_natives = state.jni.registered_natives.borrow();
                let reg_natives_for_class = reg_natives.get(&class).unwrap().borrow();
                reg_natives_for_class.get(&(method_i as u16)).unwrap().clone()
            };
            call_impl(state, frame.clone(), class.clone(), args, parsed, &res_fn, !method.is_static())
        } else {
            let res = match call(state, frame.clone(), class.clone(), method_i, args.clone(), parsed) {
                Ok(r) => r,
                Err(_) => {
                    let mangled = mangling::mangle(class.clone(), method_i);
                    //todo actually impl these at some point
                    if mangled == "Java_sun_misc_Unsafe_compareAndSwapObject".to_string() {
                        //todo do nothing for now and see what happens
                        //
                        Some(JavaValue::Boolean(true))
                    } else if mangled == "Java_sun_misc_Unsafe_objectFieldOffset".to_string() {
//                        frame.print_stack_trace();
                        let param0_obj = args[0].unwrap_object();
                        let _the_unsafe = param0_obj.as_ref().unwrap().unwrap_normal_object();
                        let param1_obj = args[1].unwrap_object();
                        let field_obj = param1_obj.as_ref().unwrap().unwrap_normal_object();
                        let borrow_1 = field_obj.fields.borrow().get("name").unwrap().unwrap_object();
                        let name_str_obj = borrow_1.as_ref().unwrap().unwrap_normal_object();
                        let borrow_2 = name_str_obj.fields.borrow().get("value").unwrap().unwrap_object();
                        let name_char_array = borrow_2.as_ref().unwrap().unwrap_array().elems.borrow();
                        let mut field_name = String::with_capacity(name_char_array.len());
                        for char_ in &*name_char_array {
                            field_name.push(char_.unwrap_char());
                        }
                        let borrow_3 = field_obj.fields.borrow().get("clazz").unwrap().unwrap_object().unwrap();
                        let field_class = borrow_3.unwrap_normal_object();
                        let borrow_4 = field_class.object_class_object_pointer.borrow();
                        let field_class_classfile = borrow_4.as_ref().unwrap().classfile.clone();
                        let mut res = None;
                        &field_class_classfile.fields.iter().enumerate().for_each(|(i, f)| {
                            if f.name(&field_class_classfile) == field_name {
                                res = Some(Some(JavaValue::Long(i as i64)));
                            }
                        });
                        res.unwrap()
//                        unimplemented!()
                    } else if mangled == "Java_sun_misc_Unsafe_getIntVolatile".to_string() {
                        let param1_obj = args[1].unwrap_object();
                        let unwrapped = param1_obj.unwrap();
                        let target_obj = unwrapped.unwrap_normal_object();
                        let var_offset = args[2].unwrap_long();
                        let classfile = &target_obj.class_pointer.classfile;
                        let field_name = classfile.constant_pool[classfile.fields[var_offset as usize].name_index as usize].extract_string_from_utf8();
                        let fields = target_obj.fields.borrow();
                        fields.get(&field_name).unwrap().clone().into()
                    } else if mangled == "Java_sun_misc_Unsafe_compareAndSwapInt".to_string() {
                        let param1_obj = args[1].unwrap_object();
                        let unwrapped = param1_obj.unwrap();
                        let target_obj = unwrapped.unwrap_normal_object();
                        let var_offset = args[2].unwrap_long();
                        let old = args[3].unwrap_int();
                        let new = args[4].unwrap_int();
                        let classfile = &target_obj.class_pointer.classfile;
                        let field_name = classfile.constant_pool[classfile.fields[var_offset as usize].name_index as usize].extract_string_from_utf8();
                        let mut fields = target_obj.fields.borrow_mut();
                        let cur_val = fields.get(&field_name).unwrap().unwrap_int();
                        if cur_val != old {
                            JavaValue::Boolean(false)
                        } else {
                            fields.insert(field_name, JavaValue::Int(new));
                            JavaValue::Boolean(true)
                        }.into()
                    } else if mangled == "Java_sun_misc_Unsafe_allocateMemory".to_string() {
                        let res: i64 = unsafe {
                            transmute(libc::malloc(transmute(args[1].unwrap_long())))
                        };
                        JavaValue::Long(res).into()
                    } else if mangled == "Java_sun_misc_Unsafe_putLong__JJ".to_string() {
//                        let args_1_borrow = args[1].unwrap_object().unwrap();
//                        let target_obj = args_1_borrow.unwrap_normal_object();
//                        let classfile = &target_obj.class_pointer.classfile;
//                        let fieldinfo_arr = &classfile.fields;
//                        let field_info_idx = args[2].unwrap_long();
//                        let target_fields = &mut target_obj.fields.borrow_mut();
//                        target_fields.insert(classfile.constant_pool[fieldinfo_arr[field_info_idx as usize].name_index as usize].extract_string_from_utf8(), args[3].clone());
//                        None
                        frame.print_stack_trace();
                        unsafe {
                            let ptr: *mut i64 = transmute(args[1].unwrap_long());
                            let val = args[2].unwrap_long();
                            ptr.write(val);
                        }
                        None
                    } else if mangled == "Java_sun_misc_Unsafe_getByte__J".to_string(){
                        unsafe {
                            let ptr: *mut i8 = transmute(args[1].unwrap_long());
                            JavaValue::Byte(ptr.read()).into()
                        }
                    }else if mangled == "Java_sun_misc_Unsafe_freeMemory".to_string() {
                        unsafe {
                            libc::free(transmute(args[1].unwrap_long()))
                        };
                        None
                    } else {
//                        frame.print_stack_trace();
                        panic!()
                    }
                }
            };
            res
        };
        match result {
            None => {}
            Some(res) => frame.push(res),
        }
    }
    println!("CALL END NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
}

fn system_array_copy(args: &mut Vec<JavaValue>) -> () {
    let src_o = args[0].clone().unwrap_object();
    let src = src_o.as_ref().unwrap().unwrap_array();
    let src_pos = args[1].clone().unwrap_int() as usize;
    let src_o = args[2].clone().unwrap_object();
    let dest = src_o.as_ref().unwrap().unwrap_array();
    let dest_pos = args[3].clone().unwrap_int() as usize;
    let length = args[4].clone().unwrap_int() as usize;
    for i in 0..length {
        let borrowed: Ref<Vec<JavaValue>> = src.elems.borrow();
        let temp = (borrowed.borrow())[src_pos + i].borrow().clone();
        dest.elems.borrow_mut()[dest_pos + i] = temp;
    }
}


pub fn invoke_interface(state: &mut InterpreterState, current_frame: Rc<StackEntry>, invoke_interface: InvokeInterface) {
    invoke_interface.count;
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(invoke_interface.index as usize, &ClassView::from(classfile.clone()));
    let class_name_ =  class_name_type.unwrap_class_type();
    //todo should I be trusting these descriptors, or should i be using the runtime class on top of the operant stack
    let _target_class = check_inited_class(state, &class_name_, current_frame.clone().into(), loader_arc.clone());
    let mut args = vec![];
    let checkpoint = current_frame.operand_stack.borrow().clone();
    setup_virtual_args(&current_frame, &expected_descriptor, &mut args, expected_descriptor.parameter_types.len() as u16 + 1);
    let this_pointer_o = args[0].unwrap_object().unwrap();
    let this_pointer = this_pointer_o.unwrap_normal_object();
    current_frame.operand_stack.replace(checkpoint);
    let target_class = this_pointer.class_pointer.clone();
//    dbg!(invoke_interface.count);
//    dbg!(class_name(&target_class.classfile));
    let (target_method_i, final_target_class) = find_target_method(state, loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);

    invoke_virtual_method_i(state, current_frame, expected_descriptor, final_target_class.clone(), target_method_i, &final_target_class.classfile.methods[target_method_i]);
}