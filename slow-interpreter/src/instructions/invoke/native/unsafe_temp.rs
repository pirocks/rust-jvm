#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//all of these functions should be implemented in libjvm
use std::mem::transmute;
use crate::utils::string_obj_to_string;

use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;
use crate::{JVMState, StackEntry};


pub fn compare_and_swap_long(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let param1_obj = args[1].unwrap_object();
    let unwrapped = param1_obj.unwrap();
    let target_obj = unwrapped.unwrap_normal_object();
    let var_offset = args[2].unwrap_long();
    let old = args[3].unwrap_long();
    let new = args[4].unwrap_long();
    let view = &target_obj.class_pointer.view();
    let field_name = view.field(var_offset as usize).field_name();
    let mut fields = target_obj.fields.borrow_mut();
    let cur_val = fields.get(&field_name).unwrap().unwrap_long();
    if cur_val != old {
        JavaValue::Boolean(false)
    } else {
        fields.insert(field_name, JavaValue::Long(new));
        JavaValue::Boolean(true)
    }.into()
}

pub fn get_object_volatile(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let temp = args[1].unwrap_object().unwrap();
    let array_idx = args[2].unwrap_long() as usize;
    let res = &temp.unwrap_array().elems.borrow()[array_idx];
    res.clone().into()
}

pub fn freeMemory(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        libc::free(transmute(args[1].unwrap_long()))
    };
    None
}

pub fn getByte__J(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        let ptr: *mut i8 = transmute(args[1].unwrap_long());
        JavaValue::Byte(ptr.read()).into()
    }
}

pub fn putLong__JJ(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        let ptr: *mut i64 = transmute(args[1].unwrap_long());
        let val = args[2].unwrap_long();
        ptr.write(val);
    }
    None
}

pub fn allocate_memory(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let res: i64 = unsafe {
        transmute(libc::malloc(transmute(args[1].unwrap_long())))
    };
    JavaValue::Long(res).into()
}

pub fn compare_and_swap_int(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let param1_obj = args[1].unwrap_object();
    let unwrapped = param1_obj.unwrap();
    let target_obj = unwrapped.unwrap_normal_object();
    let var_offset = args[2].unwrap_long();
    let old = args[3].unwrap_int();
    let new = args[4].unwrap_int();
    let view = &target_obj.class_pointer.view();
    let field_name = view.field(var_offset as usize).field_name();
    let mut fields = target_obj.fields.borrow_mut();
    let cur_val = fields.get(&field_name).unwrap().unwrap_int();
    if cur_val != old {
        JavaValue::Boolean(false)
    } else {
        fields.insert(field_name, JavaValue::Int(new));
        JavaValue::Boolean(true)
    }.into()
}

pub fn get_int_volatile(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let param1_obj = args[1].unwrap_object();
    let unwrapped = param1_obj.unwrap();
    let target_obj = unwrapped.unwrap_normal_object();
    let var_offset = args[2].unwrap_long();
    let view = &target_obj.class_pointer.view();
    let field_name = view.field(var_offset as usize).field_name();
    let fields = target_obj.fields.borrow();
    fields.get(&field_name).unwrap().clone().into()
}

pub fn object_field_offset(
    jvm: &JVMState,
    frame: &StackEntry,
    args: &mut Vec<JavaValue>,
) -> Option<JavaValue> {
    let param0_obj = args[0].unwrap_object();
    let _the_unsafe = param0_obj.as_ref().unwrap().unwrap_normal_object();
    let param1_obj = args[1].unwrap_object().unwrap();
    let field_name = string_obj_to_string(param1_obj.lookup_field("name").unwrap_object());
    let temp = param1_obj.lookup_field("clazz");
    let field_class = temp.cast_class();
    let field_class_name = field_class.as_type().unwrap_ref_type().unwrap_name();
    let inited_field_class = check_inited_class(jvm, &field_class_name, frame.class_pointer.loader(jvm).clone());
    let field_classfile = inited_field_class.view();
    let mut res = None;
    &field_classfile.fields().enumerate().for_each(|(i, f)| {
        if f.field_name() == field_name {
            res = Some(Some(JavaValue::Long(i as i64)));
        }
    });
    res.unwrap()
}


pub fn shouldBeInitialized(state: &JVMState, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let class_name_to_check = args[1].cast_class().as_type().unwrap_type_to_name().unwrap();//todo should be dome kind of should be inited
    JavaValue::Boolean(state.initialized_classes.read().unwrap().get(&class_name_to_check).is_some()).into()
}
