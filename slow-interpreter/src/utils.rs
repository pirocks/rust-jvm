use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;

use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::Object;
use crate::JVMState;
use crate::runtime_class::RuntimeClass;

pub fn lookup_method_parsed(state: &JVMState, int_state: &mut InterpreterStateGuard, class: Arc<RuntimeClass>, name: String, descriptor: &MethodDescriptor) -> Option<(usize, Arc<RuntimeClass>)> {
    lookup_method_parsed_impl(state, int_state, class, name, descriptor)
}

pub fn lookup_method_parsed_impl(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: Arc<RuntimeClass>, name: String, descriptor: &MethodDescriptor) -> Option<(usize, Arc<RuntimeClass>)> {
    let posible_methods = class.view().lookup_method_name(&name);
    let filtered = posible_methods.into_iter().filter(|m| {
        if m.is_signature_polymorphic() {
            true
        } else {
            &m.desc() == descriptor
        }
    }).collect::<Vec<_>>();
    assert!(filtered.len() <= 1);
    match filtered.iter().next() {
        None => {
            let class_name = class.view().super_name().unwrap();//todo is this unwrap safe?
            let lookup_type = PTypeView::Ref(ReferenceTypeView::Class(class_name));
            let super_class = assert_inited_or_initing_class(jvm, int_state, lookup_type); //todo this unwrap could fail, and this should really be using check_inited_class
            lookup_method_parsed_impl(jvm, int_state, super_class, name, descriptor)
        }
        Some(method_view) => {
            Some((method_view.method_i(), class.clone()))
        }
    }
}


//todo make this an impl method on Object
pub fn string_obj_to_string(str_obj: Option<Arc<Object>>) -> String {
    let temp = str_obj.unwrap().lookup_field("value");//todo pass the error up
    let chars = temp.unwrap_array();
    let borrowed_elems = chars.mut_array();
    let mut res = String::new();
    for char_ in borrowed_elems.deref() {
        res.push(char_.unwrap_char() as u8 as char);
    }
    res
}