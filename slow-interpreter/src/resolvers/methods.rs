use std::sync::Arc;

use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::MethodName;

use crate::interpreter::WasException;
use crate::InterpreterStateGuard;
use crate::java::lang::member_name::MemberName;
use crate::java::lang::reflect::method::Method;
use crate::JVMState;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::misc::get_all_methods;

pub fn resolve_invoke_virtual<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, member_name: MemberName<'gc_life>) -> Result<Result<(Method<'gc_life>, u16, Arc<RuntimeClass<'gc_life>>), ResolutionError>, WasException> {
    resolve_virtual_impl(jvm, int_state, member_name, false)
}

pub fn resolve_invoke_interface<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, member_name: MemberName<'gc_life>) -> Result<Result<(Method<'gc_life>, u16, Arc<RuntimeClass<'gc_life>>), ResolutionError>, WasException> {
    resolve_virtual_impl(jvm, int_state, member_name, true)
}

fn resolve_virtual_impl(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, member_name: MemberName<'gc_life>, include_interfaces: bool) -> Result<Result<(Method<'gc_life>, u16, Arc<RuntimeClass<'gc_life>>), ResolutionError>, WasException> {
    let method_type = member_name.get_type(jvm).cast_method_type();
    let return_type = method_type.get_rtype_as_type(jvm);
    let arg_types = method_type.get_ptypes_as_types(jvm);
    let method_descriptor = CMethodDescriptor { arg_types, return_type };
    let runtime_class = member_name.get_clazz(jvm).as_runtime_class(jvm);
    let temp = get_all_methods(jvm, int_state, runtime_class.clone(), include_interfaces)?;
    let res = temp.iter().find(|(candidate_rc, candidate_i)| {
        let view = candidate_rc.view();
        let candidate_view = view.method_view_i(*candidate_i);
        if candidate_view.is_signature_polymorphic() {
            true
        } else {
            candidate_view.desc() == &method_descriptor
        }
    });
    let (res_rc, res_i) = match res {
        Some(x) => x,
        None => return Ok(Err(ResolutionError::Linkage)),
    };
    let res_view = res_rc.view();
    let res_method_view = res_view.method_view_i(*res_i);
    Ok(Ok((Method::method_object_from_method_view(jvm, int_state, &res_method_view)?, res_method_view.method_i(), runtime_class.clone())))
}

pub fn resolve_invoke_special<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, member_name: MemberName<'gc_life>) -> Result<Result<(Method<'gc_life>, u16, Arc<RuntimeClass<'gc_life>>), ResolutionError>, WasException> {
    let method_type = member_name.get_type(jvm).cast_method_type();
    let return_type = method_type.get_rtype_as_type(jvm);
    let arg_types = method_type.get_ptypes_as_types(jvm);
    let method_descriptor = CMethodDescriptor { arg_types, return_type };
    let runtime_class = member_name.get_clazz(jvm).as_runtime_class(jvm);
    let runtime_class_view = runtime_class.view();
    let method_name_string = member_name.get_name(jvm).to_rust_string(jvm);
    let method_name = MethodName(jvm.string_pool.add_name(method_name_string, false));
    let temp = runtime_class_view.lookup_method_name(method_name);
    let res = temp.iter().find(|candidate| if candidate.is_signature_polymorphic() { true } else { candidate.desc() == &method_descriptor });
    let method_view = match res {
        Some(x) => x,
        None => {
            return Ok(Err(ResolutionError::Linkage));
        }
    };
    Ok(Ok((Method::method_object_from_method_view(jvm, int_state, &method_view)?, method_view.method_i(), runtime_class.clone())))
}

pub fn resolve_invoke_static<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, member_name: MemberName<'gc_life>, synthetic: &mut bool) -> Result<Result<(Method<'gc_life>, u16, Arc<RuntimeClass<'gc_life>>), ResolutionError>, WasException> {
    let method_type = member_name.get_type(jvm).cast_method_type();
    let return_type = method_type.get_rtype_as_type(jvm);
    let arg_types = method_type.get_ptypes_as_types(jvm);
    let runtime_class = member_name.get_clazz(jvm).as_runtime_class(jvm);
    let method_descriptor = CMethodDescriptor { arg_types, return_type };
    let runtime_class_view = runtime_class.view();
    let res = runtime_class_view
        .lookup_method_name(MethodName(jvm.string_pool.add_name(member_name.get_name(jvm).to_rust_string(jvm), false)))
        .iter()
        .find(|m| {
            if m.is_signature_polymorphic() {
                //todo more comprehensive polymorphism sanity checks.
                true
            } else {
                m.desc() == &method_descriptor
            }
        })
        .cloned(); //todo assert only one match
    match res {
        None => return Ok(Err(ResolutionError::Linkage)),
        Some(method_view) => {
            if method_view.is_signature_polymorphic() {
                *synthetic = true
            }
            Ok(Ok((Method::method_object_from_method_view(jvm, int_state, &method_view).expect("todo"), method_view.method_i(), runtime_class)))
        }
    }
}

pub enum ResolutionError {
    Linkage,
}