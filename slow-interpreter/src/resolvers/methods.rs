use descriptor_parser::MethodDescriptor;

use crate::InterpreterStateGuard;
use crate::java::lang::member_name::MemberName;
use crate::java::lang::reflect::method::Method;
use crate::JVMState;
use classfile_view::view::HasAccessFlags;

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
    let return_type = method_type.get_rtype_as_type();
	let parameter_types = method_type.get_ptypes_as_types();
    let method_descriptor = MethodDescriptor { parameter_types, return_type };
    let runtime_class = member_name.get_clazz().as_runtime_class();
    let res = runtime_class.view().lookup_method(&member_name.get_name().to_rust_string(), &method_descriptor);
    Method::method_object_from_method_view(jvm, int_state, &res.unwrap())
}

pub fn resolve_invoke_static(jvm: &JVMState, int_state: &mut InterpreterStateGuard, member_name: MemberName) -> Method {
	let method_type = member_name.get_type().cast_method_type();
	let return_type = method_type.get_rtype_as_type();
	let parameter_types = method_type.get_ptypes_as_types();
	let runtime_class = member_name.get_clazz().as_runtime_class();
	dbg!(&return_type);
	dbg!(&parameter_types);
	dbg!(member_name.get_name().to_rust_string());
	let method_descriptor = MethodDescriptor { parameter_types, return_type };
	let res = runtime_class.view().lookup_method_name(&member_name.get_name().to_rust_string()).iter().filter(|m|{
		if m.is_signature_polymorphic(){
			//todo more comprehensive polymorphism sanity checks.
			true
		}else{
			m.desc() == method_descriptor
		}
	}).next().cloned();//todo assert only one match
	assert!(res.is_some());
	// assert!(res.as_ref().unwrap().is_synthetic());
	Method::method_object_from_method_view(jvm, int_state, &res.unwrap())
}
