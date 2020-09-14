use classfile_view::view::HasAccessFlags;
use rust_jvm_common::classfile::{ACC_STATIC, REF_invokeInterface, REF_invokeStatic, REF_invokeVirtual};
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::invoke::native::mhn_temp::{IS_METHOD, REFERENCE_KIND_SHIFT};
use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;

pub fn MHN_init<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    //two params, is a static function.
    // init(MemberName mname, Object target);
    let mname = args[0].unwrap_normal_object();
    let target = args[1].unwrap_normal_object();
    // let name = mname.fields.borrow().get("name").unwrap().unwrap_object().map(|x|JavaValue::Object(x.into()).cast_string().to_rust_string());
    let debug = true;//name == "checkSpreadArgument".to_string().into();
    if target.class_pointer.view().name() == ClassName::method() {
        let flags = mname.fields.borrow().get("flags").unwrap().unwrap_int();
        let method_fields = target.fields.borrow();
        let clazz = method_fields.get("clazz").unwrap().cast_class();
        mname.fields.borrow_mut().insert("clazz".to_string(), clazz.clone().java_value());
        //todo need to resolve and then indicate the type of call
        //static v. invoke_virtual v. interface
        //see MethodHandles::init_method_MemberName
        let invoke_type_flag = ((if (flags | ACC_STATIC as i32) > 0 {
            REF_invokeStatic
        } else {
            let class_ptye = clazz.as_type();
            let class_name = class_ptye.unwrap_ref_type().try_unwrap_name().unwrap_or_else(|| unimplemented!("Handle arrays?"));
            let inited_class = check_inited_class(jvm, int_state, &class_name.into(), int_state.current_loader(jvm));
            if inited_class.view().is_interface() {
                REF_invokeInterface
            } else {
                REF_invokeVirtual
            }
        } as u32) << REFERENCE_KIND_SHIFT) as i32;
        let extra_flags = IS_METHOD | invoke_type_flag;


        // let signature = method_fields.get("signature").unwrap();


        // dbg!(signature);
        // create_method_type(state,frame,&string_obj_to_string(signature.unwrap_object()));
        // mname.fields.borrow_mut().insert("type".to_string(),frame.pop());

        let modifiers = method_fields.get("modifiers").unwrap().unwrap_int();
        mname.fields.borrow_mut().insert("flags".to_string(), JavaValue::Int(flags | modifiers | extra_flags));//todo is this really correct? what if garbage in flags?
        // let name = method_fields.get("name").unwrap();
        // mname.fields.borrow_mut().insert("name".to_string(),name.clone());
        if debug {
            dbg!(mname);
        }
    } else {

        //todo handle constructors and fields
        unimplemented!()
    }
    None//this is a void method.
}
