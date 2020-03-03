#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use runtime_common::java_values::{JavaValue, NormalObject};
use std::sync::Arc;
use runtime_common::java_values::Object::Object;
use rust_jvm_common::classnames::ClassName;
use std::cell::RefCell;
use crate::interpreter_util::check_inited_class;

pub fn MHN_resolve(state: &mut InterpreterState, frame: &Rc<StackEntry>, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
//todo
//so as far as I can find this is undocumented.
//so as far as I can figure out we have a method name and a class
//we lookup for a matching method, throw various kinds of exceptions if it doesn't work
// and return a brand new object
//                        dbg!(&args[0]);
//     dbg!(&args[0].unwrap_object().unwrap().lookup_field("clazz"));
    dbg!(&args[1]);
    let member_name = args[0].unwrap_object().unwrap();
    dbg!(member_name.lookup_field("clazz"));
    dbg!(member_name.lookup_field("name"));
    let type_java_value = member_name.lookup_field("type");
    dbg!(&type_java_value.unwrap_normal_object().class_pointer.class_view.name()); // so this is a string before resolution?
    dbg!(member_name.lookup_field("flags"));
//                        let class = args[1].unwrap_object().unwrap();
//                        let name = string_obj_to_string(member_name.lookup_field("name").unwrap_object());
//todo maybe create a class for this resolution object
//todo actually do whatever I'm meant to do here.
//what openjdk has to say: methodHandles.cpp
// java_lang_invoke_MemberName::set_flags(   mname_oop, flags);
// java_lang_invoke_MemberName::set_vmtarget(mname_oop, m());
// java_lang_invoke_MemberName::set_vmindex( mname_oop, vmindex);   // vtable/itable index
// java_lang_invoke_MemberName::set_clazz(   mname_oop, m_klass->java_mirror());
// // Note:  name and type can be lazily computed by resolve_MemberName,
// // if Java code needs them as resolved String and MethodType objects.
// // The clazz must be eagerly stored, because it provides a GC
// // root to help keep alive the Method*.
// // If relevant, the vtable or itable value is stored as vmindex.
// // This is done eagerly, since it is readily available without
// // constructing any new objects.

    let type_ = type_java_value.unwrap_normal_object();
    let (flags,_) = if type_.class_pointer.class_view.name() == ClassName::method_type(){
        let r_type_class = type_java_value.unwrap_object_nonnull().lookup_field("rtype").unwrap_object_nonnull();
        let param_types_class = type_java_value.unwrap_object_nonnull().lookup_field("ptypes").unwrap_array().unwrap_object_array_nonnull();

        // private final Class<?> rtype;
        // private final Class<?>[] ptypes;

        (0,0)
    }else {
        unimplemented!()
    };

    let resolution_object = JavaValue::Object(Arc::new(Object(NormalObject {
        gc_reachable: false,
        fields: RefCell::new(Default::default()),
        class_pointer: check_inited_class(state, &ClassName::object(), frame.clone().into(), frame.class_pointer.loader.clone()),
        bootstrap_loader: true,
        // object_class_object_pointer: RefCell::new(None),
        // array_class_object_pointer: RefCell::new(None),
        class_object_ptype: RefCell::new(None)
    })).into());
    member_name.unwrap_normal_object().fields.borrow_mut().insert("resolution".to_string(), resolution_object);
// private Class<?> clazz;
// private String name;
// private Object type;
// private int flags;
// private Object resolution;
//     member_name.unwrap_normal_object().fields.borrow_mut().insert("flags".to_string(), unimplemented!());
//     JavaValue::Object(member_name.into()).into()
    unimplemented!()
}

pub fn MHN_getConstant() -> Option<JavaValue> {
//todo
    JavaValue::Int(0).into()
}
