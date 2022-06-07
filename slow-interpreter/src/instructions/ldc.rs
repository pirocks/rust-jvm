use another_jit_vm_ir::WasException;
use classfile_view::view::constant_info_view::{ConstantInfoView, StringView};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::{AllocatedHandle, InterpreterStateGuard, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::class_objects::get_or_create_class_object;
use crate::instructions::invoke::find_target_method;
use crate::interpreter::{run_function};
use crate::interpreter_util::new_object;
use crate::java::lang::string::JString;
use crate::java::NewAsObjectOrJavaValue;
use crate::java_values::{JavaValue};
use crate::new_java_values::NewJavaValueHandle;
use crate::rust_jni::interface::string::intern_safe;
use crate::stack_entry::{StackEntryPush};

fn load_class_constant<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, type_: CPDType) -> Result<NewJavaValueHandle<'gc>, WasException> {
    load_class_constant_by_type(jvm, int_state, type_)
}

pub fn load_class_constant_by_type<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, res_class_type: CPDType) -> Result<NewJavaValueHandle<'gc>, WasException> {
    let object = get_or_create_class_object(jvm, res_class_type.clone(), int_state)?;
    Ok(NewJavaValueHandle::Object(AllocatedHandle::NormalObject(object)))
}

fn load_string_constant<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, s: &StringView) -> NewJavaValueHandle<'gc>{
    let res_string = s.string();
    assert!(int_state.throw().is_none());
    let before_intern = JString::from_rust(jvm, int_state, res_string).expect("todo");
    let string = intern_safe(jvm, before_intern.full_object().into());
    string.new_java_value_handle()
}

pub fn create_string_on_stack<'gc, 'l>(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc,'l>, res_string: String) -> Result<(), WasException> {
    let java_lang_string = CClassName::string();
    let string_class = assert_inited_or_initing_class(jvm, java_lang_string.into());
    let str_as_vec = res_string.chars();
    let chars: Vec<JavaValue<'gc>> = str_as_vec.map(|x| JavaValue::Char(x as u16)).collect();
    let string_object = new_object(jvm, interpreter_state, &string_class).to_jv();
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Object(todo!()/*Some(jvm.allocate_object(todo!()/*Object::Array(ArrayObject::new_array(jvm, interpreter_state, chars, CPDType::CharType, jvm.thread_state.new_monitor("monitor for a string".to_string()))?)*/))*/));
    let char_array_type = CPDType::array(CPDType::CharType);
    let expected_descriptor = CMethodDescriptor { arg_types: vec![char_array_type], return_type: CPDType::VoidType };
    let (constructor_i, final_target_class) = find_target_method(jvm, interpreter_state, MethodName::constructor_init(), &expected_descriptor, string_class);
    let next_entry = StackEntryPush::new_java_frame(jvm, final_target_class, constructor_i as u16, todo!()/*args*/);
    let mut function_call_frame = interpreter_state.push_frame(next_entry);
    match run_function(jvm, interpreter_state) {
        Ok(_) => {}
        Err(_) => todo!(),
    }
    let was_exception = interpreter_state.throw().is_some();
    interpreter_state.pop_frame(jvm, function_call_frame, was_exception);
    if !jvm.config.compiled_mode_active {
    }
    if interpreter_state.throw().is_some() {
        unimplemented!()
    }
    interpreter_state.push_current_operand_stack(JavaValue::Object(string_object.unwrap_object()));
    Ok(())
}


pub fn from_constant_pool_entry<'gc, 'l>(c: &ConstantInfoView, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> NewJavaValueHandle<'gc> {
    match &c {
        ConstantInfoView::Integer(i) => NewJavaValueHandle::Int(i.int),
        ConstantInfoView::Float(f) => NewJavaValueHandle::Float(f.float),
        ConstantInfoView::Long(l) => NewJavaValueHandle::Long(l.long),
        ConstantInfoView::Double(d) => NewJavaValueHandle::Double(d.double),
        ConstantInfoView::String(s) => {
            let string_value = load_string_constant(jvm, int_state, s);
            intern_safe(jvm, string_value.unwrap_object_nonnull()).new_java_value_handle()
        }
        _ => panic!(),
    }
}