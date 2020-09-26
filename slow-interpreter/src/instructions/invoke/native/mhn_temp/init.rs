use classfile_view::view::HasAccessFlags;
use rust_jvm_common::classfile::{ACC_STATIC, REF_invokeInterface, REF_invokeStatic, REF_invokeVirtual};
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::invoke::native::mhn_temp::{IS_METHOD, REFERENCE_KIND_SHIFT};
use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;
use crate::java::lang::member_name::MemberName;
use crate::java::lang::reflect::method::Method;

pub fn MHN_init<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    //two params, is a static function.
    let mname = args[0].cast_member_name();
    let target = args[1].clone();
    init(jvm,int_state,mname,target)
}


pub fn init(jvm: &JVMState, int_state: &mut InterpreterStateGuard, mname : MemberName, target: JavaValue) -> Option<JavaValue>{
    if target.unwrap_normal_object().class_pointer.view().name() == ClassName::method() {//todo replace with a try cast
        dbg!(target.cast_method().get_clazz());
        dbg!(target.cast_method().get_modifiers());
        dbg!(target.cast_method().get_name().to_rust_string());
        let target = target.cast_method();
        method_init(jvm, int_state, mname.clone(), target);
    } else {

        //todo handle constructors and fields
        unimplemented!()
    }

    dbg!(mname.get_name_or_null().map(|x|x.to_rust_string()));
    dbg!(mname.get_clazz_or_null());
    dbg!(mname.get_flags());
    // dbg!(mname.get_type().cast_method_type().to_string(jvm, int_state).to_rust_string());
    // dbg!(mname.get_resolution().cast_object().to_string(jvm, int_state).to_rust_string());

    None//this is a void method.
}

fn method_init(jvm: &JVMState, int_state: &mut InterpreterStateGuard, mname: MemberName, method: Method) {
    let flags = mname.get_flags();
    let clazz = method.get_clazz();
    mname.set_clazz(clazz.clone());
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
    let modifiers = method.get_modifiers();
    //todo is this really correct? what if garbage in flags?
    mname.set_flags(flags | modifiers | extra_flags);
}