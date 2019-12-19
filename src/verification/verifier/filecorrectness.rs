use class_loading::{class_entry, Loader, class_entry_from_string};
use classfile::{ACC_FINAL, ACC_INTERFACE, ACC_PRIVATE, ACC_STATIC};
use verification::classnames::get_referred_name;
use verification::prolog_info_writer::{class_name, class_name_legacy, get_access_flags};
use verification::unified_type::UnifiedType;
use verification::verifier::{PrologClass, PrologClassMethod, TypeSafetyResult};
use verification::verifier::TypeSafetyResult::{NeedToLoad, NotSafe, Safe};
use class_loading::BOOTSTRAP_LOADER;
use std::sync::Arc;

#[allow(unused)]
fn same_runtime_package(class1: PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn different_runtime_package(class1: &PrologClass, class2: &PrologClass) -> bool {
    //sameRuntimePackage(Class1, Class2) :-
    //    classDefiningLoader(Class1, L),
    //    classDefiningLoader(Class2, L),
    //    samePackageName(Class1, Class2).
    //
    //differentRuntimePackage(Class1, Class2) :-
    //    classDefiningLoader(Class1, L1),
    //    classDefiningLoader(Class2, L2),
    //    L1 \= L2.
    //
    //differentRuntimePackage(Class1, Class2) :-
    //    differentPackageName(Class1, Class2).
    return (!std::sync::Arc::ptr_eq(&class1.loader, &class2.loader)) || different_package_name(class1, class2);
}

fn different_package_name(class1: &PrologClass, class2: &PrologClass) -> bool {
    let packages1 = class_entry_from_string(&get_referred_name(&class_name(&class1.class)), false).packages;
    let packages2 = class_entry_from_string(&get_referred_name(&class_name(&class2.class)), false).packages;
    return packages1 != packages2;

}

//todo have an actual loader type. instead of refering to loader name
pub fn loaded_class(class: &PrologClass, loader: Arc<Loader>) -> TypeSafetyResult {
    let class_entry = class_entry(&class.class);
    if loader.loading.read().unwrap().contains_key(&class_entry) || loader.loaded.read().unwrap().contains_key(&class_entry) {
        return Safe();
    } else {
        dbg!(class);
        dbg!(class_entry);
        dbg!(loader.loading.read().unwrap().keys());
        dbg!(loader.loaded.read().unwrap().keys());
        return NeedToLoad(vec![unimplemented!()]);
    }
}

pub fn is_bootstrap_loader(loader: &Arc<Loader>) -> bool {
    return std::sync::Arc::ptr_eq(loader, &BOOTSTRAP_LOADER);
}

pub fn get_class_methods(class: &PrologClass) -> Vec<PrologClassMethod> {
    let mut res = vec![];
    for method_index in 0..class.class.methods.len() {
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
    match from {
        UnifiedType::DoubleType => match to {
            UnifiedType::DoubleType => true,
            _ => is_assignable(&UnifiedType::TwoWord, to)
        },
        UnifiedType::LongType => match to {
            UnifiedType::LongType => true,
            _ => is_assignable(&UnifiedType::TwoWord, to)
        },
        UnifiedType::FloatType => match to {
            UnifiedType::FloatType => true,
            _ => is_assignable(&UnifiedType::OneWord, to)
        },
        UnifiedType::IntType => match to {
            UnifiedType::IntType => true,
            _ => is_assignable(&UnifiedType::OneWord, to)
        },
        UnifiedType::Reference => match to {
            UnifiedType::Reference => true,
            _ => is_assignable(&UnifiedType::OneWord, to)
        }
        UnifiedType::Class(c) => match to {
            UnifiedType::Class(c2) => {
                if c == c2 {
                    return true;
                }else {
                    unimplemented!()
                }
            },
            _ => is_assignable(&UnifiedType::Reference, to)
        },
        UnifiedType::ArrayReferenceType(a) => match to {
            UnifiedType::ArrayReferenceType(a2) => unimplemented!(),
            UnifiedType::Class(c) => unimplemented!(),
            _ => is_assignable(&UnifiedType::Reference, to)
        },
        UnifiedType::TopType => match to {
            UnifiedType::TopType => true,
            _ => panic!("This might be a bug. It's a weird edge case"),
        },
        UnifiedType::UninitializedEmpty => match to {
            UnifiedType::UninitializedEmpty => true,
            _ => is_assignable(&UnifiedType::Reference, to)
        },
        UnifiedType::Uninitialized(_) => match to {
            UnifiedType::Uninitialized(_) => unimplemented!(),
            _ => is_assignable(&UnifiedType::UninitializedEmpty, to)
        },
        UnifiedType::UninitializedThis => match to {
            UnifiedType::UninitializedThis => unimplemented!(),
            _ => is_assignable(&UnifiedType::UninitializedEmpty, to)
        },
        UnifiedType::NullType => match to {
            UnifiedType::NullType => true,
            UnifiedType::Class(c) => true,
            UnifiedType::ArrayReferenceType(a) => true,
            _ => is_assignable(unimplemented!(), to),
        },
        UnifiedType::OneWord => match to {
            UnifiedType::OneWord => true,
            UnifiedType::TopType => true,
            _ => false
        },
        UnifiedType::TwoWord => match to {
            UnifiedType::TwoWord => true,
            UnifiedType::TopType => true,
            _ => false
        },
        _ => panic!("This is a bug"),//todo , should have a better message function
    }
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
    get_referred_name(&class_name(&class.class)) == "java/lang/Cloneable" ||
        get_referred_name(&class_name(&class.class)) == "java/io/Serializable"
}

pub fn is_java_subclass_of(sub: &PrologClass, super_: &PrologClass) {
    unimplemented!()
}

pub fn class_super_class_name(class: &PrologClass) -> String {
    unimplemented!()
}

pub fn super_class_chain(chain_start: &PrologClass, loader: Arc<Loader>, res: &mut Vec<PrologClass>) -> TypeSafetyResult {
    if get_referred_name(&class_name(&chain_start.class)) == "java/lang/Object" {
        //todo magic constant
        if res.is_empty() && is_bootstrap_loader(&loader) {
            return Safe();
        } else {
            return NotSafe("java/lang/Object superclasschain failed. This is bad and likely unfixable.".to_string());
        }
    }
    let loaded = loaded_class(chain_start, loader);
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

