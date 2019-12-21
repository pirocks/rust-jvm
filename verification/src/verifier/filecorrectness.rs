use std::sync::Arc;
use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::{PrologClass, PrologClassMethod};
use rust_jvm_common::loading::{Loader, class_entry_from_string};
use rust_jvm_common::classnames::{class_name, get_referred_name, class_name_legacy};
use rust_jvm_common::classfile::{ACC_STATIC, ACC_PRIVATE, ACC_INTERFACE, ACC_FINAL};
use crate::prolog::prolog_info_writer::get_access_flags;
use rust_jvm_common::loading::BOOTSTRAP_LOADER;
use rust_jvm_common::unified_types::ClassType;
use rust_jvm_common::unified_types::class_type_to_class;
use crate::verifier::TypeSafetyError;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::classfile::ConstantKind;

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
pub fn loaded_class(class: &ClassType, loader: Arc<Loader>) -> Result<PrologClass, TypeSafetyError> {
    let class_entry = class_entry_from_string(&get_referred_name(&class.class_name), false);
    match loader.loading.read().unwrap().get(&class_entry) {
        None => match loader.loaded.read().unwrap().get(&class_entry) {
            None => {
                dbg!(class);
                dbg!(class_entry);
                dbg!(loader.loading.read().unwrap().keys());
                dbg!(loader.loaded.read().unwrap().keys());
                Result::Err(TypeSafetyError::NeedToLoad(vec![class.class_name.clone()]))
            }
            Some(c) => {
                Result::Ok(PrologClass {
                    loader: loader.clone(),
                    class: c.clone(),
                })
            }
        },
        Some(c) => {
            Result::Ok(PrologClass {
                loader: loader.clone(),
                class: c.clone(),
            })
        }
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


pub fn loaded_class_(class_name: String, loader: Arc<Loader>) -> Result<PrologClass,TypeSafetyError> {
    loaded_class(&ClassType {class_name:ClassName::Str(class_name), loader:loader.clone() }, loader.clone())
}


pub fn class_is_interface(class: &PrologClass) -> bool {
    return class.class.access_flags & ACC_INTERFACE != 0;
}

pub fn is_java_sub_class_of(_from: &ClassType, _to: &ClassType) -> Result<(), TypeSafetyError> {
    unimplemented!()
}

pub fn is_assignable(from: &UnifiedType, to: &UnifiedType) -> Result<(), TypeSafetyError> {
    match from {
        UnifiedType::DoubleType => match to {
            UnifiedType::DoubleType => Result::Ok(()),
            _ => is_assignable(&UnifiedType::TwoWord, to)
        },
        UnifiedType::LongType => match to {
            UnifiedType::LongType => Result::Ok(()),
            _ => is_assignable(&UnifiedType::TwoWord, to)
        },
        UnifiedType::FloatType => match to {
            UnifiedType::FloatType => Result::Ok(()),
            _ => is_assignable(&UnifiedType::OneWord, to)
        },
        UnifiedType::IntType => match to {
            UnifiedType::IntType => Result::Ok(()),
            _ => is_assignable(&UnifiedType::OneWord, to)
        },
        UnifiedType::Reference => match to {
            UnifiedType::Reference => Result::Ok(()),
            _ => is_assignable(&UnifiedType::OneWord, to)
        }
        UnifiedType::Class(c) => match to {
            UnifiedType::Class(c2) => {
                if c == c2 {
                    return Result::Ok(());
                } else {
                    return is_java_assignable(c, c2);
                }
            }
            _ => is_assignable(&UnifiedType::Reference, to)
        },
        UnifiedType::ArrayReferenceType(a) => match to {
            UnifiedType::ArrayReferenceType(a2) => {
                if a == a2 {
                    return Result::Ok(());
                } else {
                    unimplemented!()
                }
            }
            UnifiedType::Class(_c) => unimplemented!(),
            _ => is_assignable(&UnifiedType::Reference, to)
        },
        UnifiedType::TopType => match to {
            UnifiedType::TopType => Result::Ok(()),
            _ => panic!("This might be a bug. It's a weird edge case"),
        },
        UnifiedType::UninitializedEmpty => match to {
            UnifiedType::UninitializedEmpty => Result::Ok(()),
            _ => is_assignable(&UnifiedType::Reference, to)
        },
        UnifiedType::Uninitialized(_) => match to {
            UnifiedType::Uninitialized(_) => unimplemented!(),
            _ => is_assignable(&UnifiedType::UninitializedEmpty, to)
        },
        UnifiedType::UninitializedThis => match to {
            UnifiedType::UninitializedThis => Result::Ok(()),
            _ => is_assignable(&UnifiedType::UninitializedEmpty, to)
        },
        UnifiedType::NullType => match to {
            UnifiedType::NullType => Result::Ok(()),
            UnifiedType::Class(_) => Result::Ok(()),
            UnifiedType::ArrayReferenceType(_) => Result::Ok(()),
            _ => is_assignable(unimplemented!(), to),
        },
        UnifiedType::OneWord => match to {
            UnifiedType::OneWord => Result::Ok(()),
            UnifiedType::TopType => Result::Ok(()),
            _ => Result::Err(TypeSafetyError::NotSafe("todo reason".to_string()))
        },
        UnifiedType::TwoWord => match to {
            UnifiedType::TwoWord => Result::Ok(()),
            UnifiedType::TopType => Result::Ok(()),
            _ => Result::Err(TypeSafetyError::NotSafe("todo reason".to_string()))
        },
        _ => panic!("This is a bug"),//todo , should have a better message function
    }
}

//todo how to handle arrays
pub fn is_java_assignable(from: &ClassType, to: &ClassType) -> Result<(), TypeSafetyError> {
    loaded_class(to, to.loader.clone())?;
    if class_is_interface(&PrologClass { class: class_type_to_class(to), loader: to.loader.clone() }) {
        return Result::Ok(());
    }
    return is_java_sub_class_of(from, to);
}

pub fn is_array_interface(class: PrologClass) -> bool {
    get_referred_name(&class_name(&class.class)) == "java/lang/Cloneable" ||
        get_referred_name(&class_name(&class.class)) == "java/io/Serializable"
}

pub fn is_java_subclass_of(_sub: &PrologClass, _super: &PrologClass) {
    unimplemented!()
}

pub fn class_super_class_name(class: &PrologClass) -> String {
    //todo dup, this must exist elsewhere
    let class_entry = &class.class.constant_pool[class.class.super_class as usize];
    let utf8 = match &class_entry.kind {
        ConstantKind::Class(c) => {
            &class.class.constant_pool[c.name_index as usize]
        },
        _ => panic!()
    };
    extract_string_from_utf8(utf8)
}

pub fn super_class_chain(chain_start: &PrologClass, loader: Arc<Loader>, res: &mut Vec<PrologClass>) -> Result<(), TypeSafetyError> {
    if get_referred_name(&class_name(&chain_start.class)) == "java/lang/Object" {
        //todo magic constant
        if res.is_empty() && is_bootstrap_loader(&loader) {
            return Result::Ok(());
        } else {
            return Result::Err(TypeSafetyError::NotSafe("java/lang/Object superclasschain failed. This is bad and likely unfixable.".to_string()));
        }
    }
    let class = loaded_class(&ClassType { class_name: class_name(&chain_start.class), loader: loader.clone() }, loader.clone())?;//todo loader duplication
    let super_class_name = class_super_class_name(&class);
    let super_class = loaded_class_(super_class_name, loader.clone())?;
    res.push(super_class);
    super_class_chain(&chain_start, loader.clone(), res)?;
    Result::Ok(())
}


pub fn is_static(method: &PrologClassMethod, class: &PrologClass) -> bool {
    //todo check if same
    (get_access_flags(class, method) & ACC_STATIC) > 0
}

pub fn is_private(method: &PrologClassMethod, class: &PrologClass) -> bool {
    //todo check if method class and class same
    (get_access_flags(class, method) & ACC_PRIVATE) > 0
}

pub fn does_not_override_final_method(class: &PrologClass, method: &PrologClassMethod) -> Result<(), TypeSafetyError> {
    dbg!(class_name_legacy(&class.class));
    if class_name_legacy(&class.class) == "java/lang/Object" {
        if is_bootstrap_loader(&class.loader) {
            Result::Ok(())
        } else {
            Result::Err(TypeSafetyError::NotSafe("Loading Object w/o bootstrap loader".to_string()))
        }
    } else if is_private(method, class) {
        Result::Ok(())
    } else if is_static(method, class) {
        Result::Ok(())
    } else if does_not_override_final_method_of_superclass(class, method) {
        Result::Ok(())
    } else {
        Result::Err(TypeSafetyError::NotSafe("Failed does_not_override_final_method".to_string()))
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
