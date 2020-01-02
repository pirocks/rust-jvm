use std::sync::Arc;
use rust_jvm_common::unified_types::{UnifiedType, ClassWithLoader};
use crate::verifier::{ClassWithLoaderMethod, get_class};
use rust_jvm_common::loading::Loader;
use rust_jvm_common::classnames::{get_referred_name, class_name_legacy};
use rust_jvm_common::classfile::{ACC_STATIC, ACC_PRIVATE, ACC_INTERFACE, ACC_FINAL};
use crate::verifier::TypeSafetyError;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::utils::{extract_string_from_utf8, method_name};
use rust_jvm_common::classfile::ConstantKind;
use rust_jvm_common::loading::LoaderName;

#[allow(unused)]
fn same_runtime_package(class1: ClassWithLoader, class2: &ClassWithLoader) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn different_runtime_package(class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
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

fn different_package_name(class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
    unimplemented!()
//    let packages1 = class_entry_from_string(&get_referred_name(&class1.class_name), false).packages;
//    let packages2 = class_entry_from_string(&get_referred_name(&class2.class_name), false).packages;
//    return packages1 != packages2;
}

//todo retries on this should be supressed.
pub fn loaded_class(class: &ClassWithLoader, _loader: Arc<dyn Loader + Send + Sync>) -> Result<ClassWithLoader, TypeSafetyError> {
    let _class_entry = class_entry_from_string(&get_referred_name(&class.class_name), false);
//    match loader.loading.read().unwrap().get(&class_entry) {
//        None => match loader.loaded.read().unwrap().get(&class_entry) {
//            None => {
//                dbg!(class);
//                dbg!(class_entry);
//                dbg!(loader.loading.read().unwrap().keys());
//                dbg!(loader.loaded.read().unwrap().keys());
//                Result::Err(TypeSafetyError::NeedToLoad(vec![class.class_name.clone()]))
//            }
//            Some(c) => {
//                Result::Ok(ClassWithLoader {
//                    loader: loader.clone(),
//                    class_name: class_name(c),
//                })
//            }
//        },
//        Some(c) => {
//            Result::Ok(ClassWithLoader {
//                loader: loader.clone(),
//                class_name: class_name(c),
//            })
//        }
//    }
    unimplemented!()
}

pub fn is_bootstrap_loader(loader: &Arc<dyn Loader + Send + Sync>) -> bool {
    return std::sync::Arc::ptr_eq(loader, &BOOTSTRAP_LOADER.clone());
}

pub fn get_class_methods(class: &ClassWithLoader) -> Vec<ClassWithLoaderMethod> {
    let mut res = vec![];
    for method_index in 0..get_class(class).methods.len() {
        res.push(ClassWithLoaderMethod { prolog_class: class, method_index })
    }
    res
}

pub fn class_is_final(class: &ClassWithLoader) -> bool {
    get_class(class).access_flags & ACC_FINAL != 0
}


pub fn loaded_class_(class_name: String, loader: Arc<dyn Loader + Send + Sync>) -> Result<ClassWithLoader, TypeSafetyError> {
    loaded_class(&ClassWithLoader { class_name: ClassName::Str(class_name), loader: loader.clone() }, loader.clone())
}


pub fn class_is_interface(class: &ClassWithLoader) -> bool {
    get_class(class).access_flags & ACC_INTERFACE != 0
}

pub fn is_java_sub_class_of(from: &ClassWithLoader, to: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    if get_referred_name(&from.class_name) == get_referred_name(&to.class_name) {
        return Result::Ok(());
    }
    let mut chain = vec![];
    super_class_chain(&loaded_class(from, from.loader.clone())?, from.loader.clone(), &mut chain)?;
    match chain.iter().find(|x| {
        x.class_name == to.class_name
    }) {
        None => {
//            dbg!(chain);
            dbg!(&from.class_name);
            dbg!(&to.class_name);
            dbg!(&chain);
            Result::Err(unknown_error_verifying!())
        }
        Some(c) => {
            loaded_class(&ClassWithLoader { class_name: c.class_name.clone(), loader: c.loader.clone() }, to.loader.clone())?;
            Result::Ok(())
        }
    }
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
            //technically the next case should be partially part of is_java_assignable but is here
            UnifiedType::Class(c) => {
                if !is_assignable(&UnifiedType::Reference, to).is_ok(){
                    //todo okay to use name like that?
                    if c.class_name == ClassName::Str("java/lang/Object".to_string()) &&
                        c.loader.name() == LoaderName::BootstrapLoader{
                        return Result::Ok(())
                    }
                }
                is_assignable(&UnifiedType::Reference, to)
            },
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
            //todo really need to do something about these magic strings
            _ => is_assignable(&UnifiedType::Class(ClassWithLoader { class_name: ClassName::Str("java/lang/Object".to_string()), loader: BOOTSTRAP_LOADER.clone() }), to),
        },
        UnifiedType::OneWord => match to {
            UnifiedType::OneWord => Result::Ok(()),
            UnifiedType::TopType => Result::Ok(()),
            _ => {/*dbg!(to);*/Result::Err(unknown_error_verifying!())}
        },
        UnifiedType::TwoWord => match to {
            UnifiedType::TwoWord => Result::Ok(()),
            UnifiedType::TopType => Result::Ok(()),
            _ => {dbg!(to);Result::Err(unknown_error_verifying!())}
        },
        _ => panic!("This is a bug"),//todo , should have a better message function
    }
}

pub fn is_java_assignable(from: &ClassWithLoader, to: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    loaded_class(to, to.loader.clone())?;
    if class_is_interface(&ClassWithLoader { class_name: to.class_name.clone(), loader: to.loader.clone() }) {
        return Result::Ok(());
    }
    return is_java_sub_class_of(from, to);
}

pub fn is_array_interface(class: ClassWithLoader) -> bool {
    get_referred_name(&class.class_name) == "java/lang/Cloneable" ||
        get_referred_name(&class.class_name) == "java/io/Serializable"
}

pub fn is_java_subclass_of(_sub: &ClassWithLoader, _super: &ClassWithLoader) {
    unimplemented!()
}

pub fn class_super_class_name(class: &ClassWithLoader) -> String {
    //todo dup, this must exist elsewhere
    let classfile = get_class(class);
    let class_entry = &classfile.constant_pool[classfile.super_class as usize];
    let utf8 = match &class_entry.kind {
        ConstantKind::Class(c) => {
            &classfile.constant_pool[c.name_index as usize]
        }
        _ => panic!()
    };
    extract_string_from_utf8(utf8)
}

pub fn super_class_chain(chain_start: &ClassWithLoader, loader: Arc<dyn Loader + Send + Sync>, res: &mut Vec<ClassWithLoader>) -> Result<(), TypeSafetyError> {
    if get_referred_name(&chain_start.class_name) == "java/lang/Object" {
        //todo magic constant
        if /*res.is_empty() &&*/ is_bootstrap_loader(&loader) {
            return Result::Ok(());
        } else {
            return Result::Err(TypeSafetyError::NotSafe("java/lang/Object superclasschain failed. This is bad and likely unfixable.".to_string()));
        }
    }
    let class = loaded_class(&ClassWithLoader { class_name: chain_start.class_name.clone(), loader: loader.clone() }, loader.clone())?;//todo loader duplication
    let super_class_name = class_super_class_name(&class);
    let super_class = loaded_class_(super_class_name.clone(), loader.clone())?;
    res.push(super_class);
    super_class_chain(&loaded_class_(super_class_name, loader.clone())?, loader.clone(), res)?;
    Result::Ok(())
}


pub fn is_final_method(method: &ClassWithLoaderMethod, class: &ClassWithLoader) -> bool {
    //todo check if same
    (get_access_flags(class, method) & ACC_FINAL) > 0
}


pub fn is_static(method: &ClassWithLoaderMethod, class: &ClassWithLoader) -> bool {
    //todo check if same
    (get_access_flags(class, method) & ACC_STATIC) > 0
}

pub fn is_private(method: &ClassWithLoaderMethod, class: &ClassWithLoader) -> bool {
    //todo check if method class and class same
    (get_access_flags(class, method) & ACC_PRIVATE) > 0
}

pub fn does_not_override_final_method(class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    dbg!(class_name_legacy(&get_class(class)));
    if class_name_legacy(&get_class(class)) == "java/lang/Object" {
        if is_bootstrap_loader(&class.loader) {
            Result::Ok(())
        } else {
            Result::Err(TypeSafetyError::NotSafe("Loading Object w/o bootstrap loader".to_string()))
        }
    } else if is_private(method, class) {
        Result::Ok(())
    } else if is_static(method, class) {
        Result::Ok(())
    } else {
        does_not_override_final_method_of_superclass(class, method)
    }
}

pub fn final_method_not_overridden(method: &ClassWithLoaderMethod, super_class: &ClassWithLoader, super_method_list: &Vec<ClassWithLoaderMethod>) -> Result<(),TypeSafetyError> {
    let method_class = get_class(method.prolog_class);
    let method_info = &method_class.methods[method.method_index];
    let method_name_ = method_name(&method_class, method_info);
    let descriptor_string = extract_string_from_utf8(&method_class.constant_pool[method_info.descriptor_index as usize]);
    let matching_method = super_method_list.iter().find(|x|{
        let x_method_class = get_class(x.prolog_class);
        let x_method_info = &x_method_class.methods[x.method_index];
        let x_method_name = method_name(&x_method_class, x_method_info);
        let x_descriptor_string = extract_string_from_utf8(&x_method_class.constant_pool[x_method_info.descriptor_index as usize]);
        x_descriptor_string == descriptor_string  && x_method_name == method_name_
    });
    match matching_method {
        None => {
            return does_not_override_final_method(super_class,method)
        },
        Some(method) => {
            if is_final_method(method,super_class){
                if is_private(method,super_class) || is_static(method,super_class){
                    return Result::Ok(())
                }
            }else{
                if is_private(method,super_class) || is_static(method,super_class){
                    return does_not_override_final_method(super_class,method)
                }else {
                    return Result::Ok(())
                }
            }
        },
    };
    Result::Err(unknown_error_verifying!())
}

pub fn does_not_override_final_method_of_superclass(class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(),TypeSafetyError> {
    let super_class_name = class_super_class_name(class);
    let super_class = loaded_class_(super_class_name,BOOTSTRAP_LOADER.clone())?;
    let super_methods_list= get_class_methods(&super_class);
    final_method_not_overridden(method,&super_class,&super_methods_list)
}

pub fn get_access_flags(class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> u16 {
//    assert!(method.prolog_class == class);//todo why the duplicate parameters?
    get_class(class).methods[method.method_index as usize].access_flags
}