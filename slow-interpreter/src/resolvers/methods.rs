use crate::method_table::MethodId;
use rust_jvm_common::classnames::ClassName;
use crate::JVMState;
use crate::InterpreterStateGuard;
use crate::interpreter_util::check_inited_class;
use descriptor_parser::parse_method_descriptor;

pub enum ResolutionError{

}


fn resolve_method(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class_name: ClassName, name: &String, desc: &str) -> Result<MethodId,ResolutionError> {
	let runtime_class = check_inited_class(jvm,int_state,&class_name.into(),jvm.bootstrap_loader.clone());//todo loader
	let index = runtime_class.view().lookup_method(name,&parse_method_descriptor(desc).unwrap()).unwrap().method_i();
	let id= jvm.method_table.write().unwrap().get_method_id(runtime_class, index as u16);
	Result::Ok(id)
}