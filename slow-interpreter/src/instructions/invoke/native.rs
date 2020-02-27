use rust_jvm_common::classnames::class_name;
use runtime_common::java_values::JavaValue;
use std::mem::transmute;
use crate::rust_jni::{mangling, call_impl, call};
use rust_jvm_common::classfile::{ACC_NATIVE, ACC_STATIC};
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use std::sync::Arc;

use std::cell::Ref;
use std::borrow::Borrow;
use classfile_view::view::descriptor_parser::MethodDescriptor;

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
                    if mangled == "Java_sun_misc_Unsafe_objectFieldOffset".to_string() {
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
                        unsafe {
                            let ptr: *mut i64 = transmute(args[1].unwrap_long());
                            let val = args[2].unwrap_long();
                            ptr.write(val);
                        }
                        None
                    } else if mangled == "Java_sun_misc_Unsafe_getByte__J".to_string() {
                        unsafe {
                            let ptr: *mut i8 = transmute(args[1].unwrap_long());
                            JavaValue::Byte(ptr.read()).into()
                        }
                    } else if mangled == "Java_sun_misc_Unsafe_freeMemory".to_string() {
                        unsafe {
                            libc::free(transmute(args[1].unwrap_long()))
                        };
                        None
                        //todo all these unsafe function thingys are getting a tad excessive
                    } else if mangled == "Java_sun_misc_Unsafe_getObjectVolatile".to_string() {
                        let temp = args[1].unwrap_object().unwrap();
                        let array_idx = args[2].unwrap_long() as usize;
                        let res = &temp.unwrap_array().elems.borrow()[array_idx];
                        res.clone().into()
                    } else if mangled == "Java_sun_misc_Unsafe_compareAndSwapLong".to_string() {
                        let param1_obj = args[1].unwrap_object();
                        let unwrapped = param1_obj.unwrap();
                        let target_obj = unwrapped.unwrap_normal_object();
                        let var_offset = args[2].unwrap_long();
                        let old = args[3].unwrap_long();
                        let new = args[4].unwrap_long();
                        let classfile = &target_obj.class_pointer.classfile;
                        let field_name = classfile.constant_pool[classfile.fields[var_offset as usize].name_index as usize].extract_string_from_utf8();
                        let mut fields = target_obj.fields.borrow_mut();
                        let cur_val = fields.get(&field_name).unwrap().unwrap_long();
                        if cur_val != old {
                            JavaValue::Boolean(false)
                        } else {
                            fields.insert(field_name, JavaValue::Long(new));
                            JavaValue::Boolean(true)
                        }.into()
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_registerNatives"{
                        //todo
                        None
                    }
                    else {
                       frame.print_stack_trace();
                        dbg!(mangled);
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