use verification::verifier::{PrologClass, TypeSafetyResult, PrologClassMethod};
use verification::prolog_info_writer::{class_name_legacy, get_access_flags};
use verification::unified_type::UnifiedType;
use class_loading::{class_entry, Loader};
use verification::verifier::TypeSafetyResult::{Safe, NeedToLoad, NotSafe};
use classfile::{ACC_INTERFACE, ACC_PRIVATE, ACC_STATIC, ACC_FINAL};

#[allow(unused)]
fn same_runtime_package(class1: PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn different_runtime_package(class1: PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}


//todo have an actual loader type. instead of refering to loader name
pub fn loaded_class(class: &PrologClass, loader: Loader) -> TypeSafetyResult {
    let class_entry = class_entry(&class.class);
    if loader.loading.borrow().contains_key(&class_entry) || loader.loaded.borrow().contains_key(&class_entry) {
        return Safe();
    } else {
        return NeedToLoad(vec![unimplemented!()]);
    }
}



pub fn is_bootstrap_loader(loader: &String) -> bool {
    return loader == &"bl".to_string();//todo  what if someone defines a Loader class called bl
}

pub fn get_class_methods(class: &PrologClass) -> Vec<PrologClassMethod> {
    let mut res = vec![];
    for method_index in 0..class.class.methods.borrow_mut().len() {
        res.push(PrologClassMethod { prolog_class: class, method_index })
    }
    res
}

pub fn class_is_final(class: &PrologClass) -> bool {
    class.class.access_flags & ACC_FINAL != 0
}





pub fn loaded_class_(class_name: String, loader_name: String) -> Option<PrologClass> {
    unimplemented!()
}



pub fn class_is_interface(class: &PrologClass) -> bool {
    return class.class.access_flags & ACC_INTERFACE != 0;
}

#[allow(unused)]
pub fn is_java_sub_class_of(from: &PrologClass, to: &PrologClass) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn is_assignable(from: &UnifiedType, to: &UnifiedType) -> bool {
    unimplemented!()
}

//todo how to handle arrays
pub fn is_java_assignable(from: &PrologClass, to: &PrologClass) -> bool {
    match loaded_class(to, unimplemented!()) {
        TypeSafetyResult::Safe() => { return class_is_interface(to); }
        _ => unimplemented!()
    }
    unimplemented!();
    return is_java_sub_class_of(from, to);
}

pub fn is_array_interface(class: PrologClass) -> bool {
    class_name_legacy(&class.class) == "java/lang/Cloneable" ||
        class_name_legacy(&class.class) == "java/io/Serializable"
}

pub fn is_java_subclass_of(sub: &PrologClass, super_: &PrologClass) {
    unimplemented!()
}

pub fn class_super_class_name(class: &PrologClass) -> String {
    unimplemented!()
}

pub fn super_class_chain(chain_start: &PrologClass, loader: String) -> Vec<PrologClass> {
    let loaded = loaded_class(chain_start, unimplemented!());
    unimplemented!()
}



pub fn is_static(method: &PrologClassMethod, class: &PrologClass) -> bool {
    //todo check if same
    (get_access_flags(class, method) & ACC_STATIC) > 0
}

pub fn is_private(method: &PrologClassMethod, class: &PrologClass) -> bool {
    //todo check if method class and class same
    (get_access_flags(class, method) & ACC_PRIVATE) > 0
}

pub fn does_not_override_final_method(class: &PrologClass, method: &PrologClassMethod) -> TypeSafetyResult {
    dbg!(class_name_legacy(&class.class));
    if class_name_legacy(&class.class) == "java/lang/Object" {
        if is_bootstrap_loader(&class.loader) {
            Safe()
        } else {
            NotSafe("Loading Object w/o bootstrap loader".to_string())
        }
    } else if is_private(method, class) {
        Safe()
    } else if is_static(method, class) {
        Safe()
    } else if does_not_override_final_method_of_superclass(class, method) {
        Safe()
    } else {
        NotSafe("Failed does_not_override_final_method".to_string())
    }
}

#[allow(unused)]
pub fn final_method_not_overridden(method: &PrologClassMethod, super_class: &PrologClass, method_list: &Vec<PrologClassMethod>) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn does_not_override_final_method_of_superclass(class: &PrologClass, method: &PrologClassMethod) -> bool {
    unimplemented!()
}

