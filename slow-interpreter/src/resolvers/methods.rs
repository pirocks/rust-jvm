use descriptor_parser::MethodDescriptor;

use crate::InterpreterStateGuard;
use crate::java::lang::member_name::MemberName;
use crate::java::lang::reflect::method::Method;
use crate::JVMState;
use crate::runtime_class::RuntimeClass;
use std::sync::Arc;

pub fn resolve_invoke_virtual<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, member_name: MemberName) -> (Method, usize,Arc<RuntimeClass>) {
    let method_type = member_name.get_type().cast_method_type();
    let return_type = method_type.get_rtype_as_type();
	let parameter_types = method_type.get_ptypes_as_types();
    let method_descriptor = MethodDescriptor { parameter_types, return_type };
    let runtime_class = member_name.get_clazz().as_runtime_class();
    let res = runtime_class.view().lookup_method(&member_name.get_name().to_rust_string(), &method_descriptor);
	let method_view = res.unwrap();
	(Method::method_object_from_method_view(jvm, int_state, &method_view),method_view.method_i(),runtime_class.clone())
}

pub fn resolve_invoke_static<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, member_name: MemberName, synthetic: &mut bool) -> (Method, usize,Arc<RuntimeClass>) {
	let method_type = member_name.get_type().cast_method_type();
	let return_type = method_type.get_rtype_as_type();
	let parameter_types = method_type.get_ptypes_as_types();
	let runtime_class = member_name.get_clazz().as_runtime_class();
	let method_descriptor = MethodDescriptor { parameter_types, return_type };
	let res = runtime_class.view().lookup_method_name(&member_name.get_name().to_rust_string()).iter().find(|m|{
		if m.is_signature_polymorphic(){
			//todo more comprehensive polymorphism sanity checks.
			true
		}else{
			m.desc() == method_descriptor
		}
	}).cloned();//todo assert only one match
	assert!(res.is_some());
	let method_view = &res.unwrap();
	if method_view.is_signature_polymorphic(){
		*synthetic = true
	}
	(Method::method_object_from_method_view(jvm, int_state, method_view),method_view.method_i(),runtime_class)
}
