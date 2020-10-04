use descriptor_parser::MethodDescriptor;

use crate::InterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::member_name::MemberName;
use crate::java::lang::reflect::method::Method;
use crate::java_values::JavaValue;
use crate::JVMState;

// pub enum ResolutionError{
//
// }


// fn resolve_method(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class_name: ClassName, name: &String, desc: &str) -> Result<MethodId,ResolutionError> {
// 	let runtime_class = check_inited_class(jvm,int_state,&class_name.into(),jvm.bootstrap_loader.clone());//todo loader
// 	let index = runtime_class.view().lookup_method(name,&parse_method_descriptor(desc).unwrap()).unwrap().method_i();
// 	let id= jvm.method_table.write().unwrap().get_method_id(runtime_class, index as u16);
// 	Result::Ok(id)
// }

pub fn resolve_invoke_virtual(jvm: &JVMState, int_state: &mut InterpreterStateGuard, member_name: MemberName) -> Method {
    let method_type = member_name.get_type().cast_method_type();
    let rtype: JClass = method_type.get_rtype();
    let ptypes: JavaValue = method_type.get_ptypes();
    let parameter_types = ptypes.unwrap_array().unwrap_object_array().iter()
        .map(|x| JavaValue::Object(x.clone()).cast_class().as_type().to_ptype()).collect();
    let method_descriptor = MethodDescriptor { parameter_types, return_type: rtype.as_type().to_ptype() };
    let runtime_class = member_name.get_clazz().as_runtime_class();
    let res = runtime_class.view().lookup_method(&member_name.get_name().to_rust_string(), &method_descriptor);
    Method::method_object_from_method_view(jvm, int_state, &res.unwrap())
}
