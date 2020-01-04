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
use crate::VerifierContext;
use classfile_parser::types::Descriptor;
use classfile_parser::types::parse_field_descriptor;
use classfile_parser::types::parse_method_descriptor;

#[allow(unused)]
fn same_runtime_package(vf: &VerifierContext, class1: ClassWithLoader, class2: &ClassWithLoader) -> bool {
    unimplemented!()
}

pub fn different_runtime_package(vf: &VerifierContext, class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
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
    return (!std::sync::Arc::ptr_eq(&class1.loader, &class2.loader)) || different_package_name(vf, class1, class2);
}

fn different_package_name(_vf: &VerifierContext, class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
    let name1 = get_referred_name(&class1.class_name);
    let name2 = get_referred_name(&class2.class_name);
    let split1: Vec<&str> = name1.split("/").collect();
    let split2: Vec<&str> = name2.split("/").collect();
    assert!(split1.len() >= 1);
    assert!(split2.len() >= 1);
    let package_slice1 = &split1[..split1.len() - 1];
    let package_slice2 = &split2[..split2.len() - 1];
    package_slice1.iter().zip(package_slice2.iter()).all(|(a, b)| {
        a == b
    })
//    dbg!(&package_slice1);
//    dbg!(&package_slice2);
//    unimplemented!();
//    let packages1 = class_entry_from_string(&get_referred_name(&class1.class_name), false).packages;
//    let packages2 = class_entry_from_string(&get_referred_name(&class2.class_name), false).packages;
//    return packages1 != packages2;
}


pub fn is_bootstrap_loader(vf: &VerifierContext, loader: &Arc<dyn Loader + Send + Sync>) -> bool {
    return std::sync::Arc::ptr_eq(loader, &vf.bootstrap_loader);
}

pub fn get_class_methods<'l>(vf: &VerifierContext, class: &'l ClassWithLoader) -> Vec<ClassWithLoaderMethod<'l>> {
    let mut res = vec![];
    for method_index in 0..get_class(vf, class).methods.len() {
        res.push(ClassWithLoaderMethod { prolog_class: class, method_index })
    }
    res
}

pub fn class_is_final(vf: &VerifierContext, class: &ClassWithLoader) -> bool {
    get_class(vf, class).access_flags & ACC_FINAL != 0
}


pub fn loaded_class(_vf: &VerifierContext, class_name: ClassName, loader: Arc<dyn Loader + Send + Sync>) -> Result<ClassWithLoader, TypeSafetyError> {
    if loader.initiating_loader_of(&class_name) {
        Result::Ok(ClassWithLoader { class_name, loader })
    } else {
        match loader.pre_load(loader.clone(), &class_name) {
            Ok(_) => Result::Ok(ClassWithLoader { class_name, loader }),
            Err(_) => unimplemented!(),
        }
    }
}


pub fn class_is_interface(vf: &VerifierContext, class: &ClassWithLoader) -> bool {
    get_class(vf, class).access_flags & ACC_INTERFACE != 0
}

pub fn is_java_sub_class_of(vf: &VerifierContext, from: &ClassWithLoader, to: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    if get_referred_name(&from.class_name) == get_referred_name(&to.class_name) {
        return Result::Ok(());
    }
    let mut chain = vec![];
    super_class_chain(vf, &loaded_class(vf, from.class_name.clone(), from.loader.clone())?, from.loader.clone(), &mut chain)?;
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
            loaded_class(vf, c.class_name.clone(), to.loader.clone())?;
            Result::Ok(())
        }
    }
}

pub fn is_assignable(vf: &VerifierContext, from: &UnifiedType, to: &UnifiedType) -> Result<(), TypeSafetyError> {
    match from {
        UnifiedType::DoubleType => match to {
            UnifiedType::DoubleType => Result::Ok(()),
            _ => is_assignable(vf, &UnifiedType::TwoWord, to)
        },
        UnifiedType::LongType => match to {
            UnifiedType::LongType => Result::Ok(()),
            _ => is_assignable(vf, &UnifiedType::TwoWord, to)
        },
        UnifiedType::FloatType => match to {
            UnifiedType::FloatType => Result::Ok(()),
            _ => is_assignable(vf, &UnifiedType::OneWord, to)
        },
        UnifiedType::IntType => match to {
            UnifiedType::IntType => Result::Ok(()),
            _ => is_assignable(vf, &UnifiedType::OneWord, to)
        },
        UnifiedType::Reference => match to {
            UnifiedType::Reference => Result::Ok(()),
            _ => is_assignable(vf, &UnifiedType::OneWord, to)
        }
        UnifiedType::Class(c) => match to {
            UnifiedType::Class(c2) => {
                if c == c2 {
                    return Result::Ok(());
                } else {
                    return is_java_assignable(vf, c, c2);
                }
            }
            _ => is_assignable(vf, &UnifiedType::Reference, to)
        },
        UnifiedType::ArrayReferenceType(a) => match to {
            UnifiedType::ArrayReferenceType(a2) => {
                if a == a2 {
                    return Result::Ok(());
                } else {
                    dbg!(a);
                    dbg!(a2);
                    unimplemented!()
                }
            }
            //technically the next case should be partially part of is_java_assignable but is here
            UnifiedType::Class(c) => {
                if !is_assignable(vf, &UnifiedType::Reference, to).is_ok() {
                    //todo okay to use name like that?
                    if c.class_name == ClassName::Str("java/lang/Object".to_string()) &&
                        c.loader.name() == LoaderName::BootstrapLoader {
                        return Result::Ok(());
                    }
                }
                is_assignable(vf, &UnifiedType::Reference, to)
            }
            _ => is_assignable(vf, &UnifiedType::Reference, to)
        },
        UnifiedType::TopType => match to {
            UnifiedType::TopType => Result::Ok(()),
            _ => panic!("This might be a bug. It's a weird edge case"),
        },
        UnifiedType::UninitializedEmpty => match to {
            UnifiedType::UninitializedEmpty => Result::Ok(()),
            _ => is_assignable(vf, &UnifiedType::Reference, to)
        },
        UnifiedType::Uninitialized(_) => match to {
            UnifiedType::Uninitialized(_) => unimplemented!(),
            _ => is_assignable(vf, &UnifiedType::UninitializedEmpty, to)
        },
        UnifiedType::UninitializedThis => match to {
            UnifiedType::UninitializedThis => Result::Ok(()),
            _ => is_assignable(vf, &UnifiedType::UninitializedEmpty, to)
        },
        UnifiedType::NullType => match to {
            UnifiedType::NullType => Result::Ok(()),
            UnifiedType::Class(_) => Result::Ok(()),
            UnifiedType::ArrayReferenceType(_) => Result::Ok(()),
            //todo really need to do something about these magic strings
            _ => is_assignable(vf, &UnifiedType::Class(ClassWithLoader { class_name: ClassName::Str("java/lang/Object".to_string()), loader: vf.bootstrap_loader.clone() }), to),
        },
        UnifiedType::OneWord => match to {
            UnifiedType::OneWord => Result::Ok(()),
            UnifiedType::TopType => Result::Ok(()),
            _ => { /*dbg!(to);*/Result::Err(unknown_error_verifying!()) }
        },
        UnifiedType::TwoWord => match to {
            UnifiedType::TwoWord => Result::Ok(()),
            UnifiedType::TopType => Result::Ok(()),
            _ => {
                dbg!(to);
                Result::Err(unknown_error_verifying!())
            }
        },
        _ => panic!("This is a bug"),//todo , should have a better message function
    }
}

pub fn is_java_assignable(vf: &VerifierContext, from: &ClassWithLoader, to: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    loaded_class(vf, to.class_name.clone(), to.loader.clone())?;
    if class_is_interface(vf, &ClassWithLoader { class_name: to.class_name.clone(), loader: to.loader.clone() }) {
        return Result::Ok(());
    }
    return is_java_sub_class_of(vf, from, to);
}

pub fn is_array_interface(_vf: &VerifierContext, class: ClassWithLoader) -> bool {
    get_referred_name(&class.class_name) == "java/lang/Cloneable" ||
        get_referred_name(&class.class_name) == "java/io/Serializable"
}

pub fn is_java_subclass_of(_vf: &VerifierContext, _sub: &ClassWithLoader, _super: &ClassWithLoader) {
    unimplemented!()
}

pub fn class_super_class_name(vf: &VerifierContext, class: &ClassWithLoader) -> ClassName {
    //todo dup, this must exist elsewhere
    let classfile = get_class(vf, class);
    let class_entry = &classfile.constant_pool[classfile.super_class as usize];
    let utf8 = match &class_entry.kind {
        ConstantKind::Class(c) => {
            &classfile.constant_pool[c.name_index as usize]
        }
        _ => panic!()
    };
    ClassName::Str(extract_string_from_utf8(utf8))//todo use weak ref + index instead
}

pub fn super_class_chain(vf: &VerifierContext, chain_start: &ClassWithLoader, loader: Arc<dyn Loader + Send + Sync>, res: &mut Vec<ClassWithLoader>) -> Result<(), TypeSafetyError> {
    if get_referred_name(&chain_start.class_name) == "java/lang/Object" {
        //todo magic constant
        if /*res.is_empty() &&*/ is_bootstrap_loader(vf, &loader) {
            return Result::Ok(());
        } else {
            return Result::Err(TypeSafetyError::NotSafe("java/lang/Object superclasschain failed. This is bad and likely unfixable.".to_string()));
        }
    }
    let class = loaded_class(vf, chain_start.class_name.clone(), loader.clone())?;//todo loader duplication
    let super_class_name = class_super_class_name(vf, &class);
    let super_class = loaded_class(vf, super_class_name.clone(), loader.clone())?;
    res.push(super_class);
    super_class_chain(vf, &loaded_class(vf, super_class_name.clone(), loader.clone())?, loader.clone(), res)?;
    Result::Ok(())
}


pub fn is_final_method(vf: &VerifierContext, method: &ClassWithLoaderMethod, class: &ClassWithLoader) -> bool {
    //todo check if same
    (get_access_flags(vf, class, method) & ACC_FINAL) > 0
}


pub fn is_static(vf: &VerifierContext, method: &ClassWithLoaderMethod, class: &ClassWithLoader) -> bool {
    //todo check if same
    (get_access_flags(vf, class, method) & ACC_STATIC) > 0
}

pub fn is_private(vf: &VerifierContext, method: &ClassWithLoaderMethod, class: &ClassWithLoader) -> bool {
    //todo check if method class and class same
    (get_access_flags(vf, class, method) & ACC_PRIVATE) > 0
}

pub fn does_not_override_final_method(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    dbg!(class_name_legacy(&get_class(vf,class)));
    if get_referred_name(&class.class_name) == "java/lang/Object" {
        if is_bootstrap_loader(vf, &class.loader) {
            Result::Ok(())
        } else {
            Result::Err(TypeSafetyError::NotSafe("Loading Object w/o bootstrap loader".to_string()))
        }
    } else if is_private(vf, method, class) {
        Result::Ok(())
    } else if is_static(vf, method, class) {
        Result::Ok(())
    } else {
        does_not_override_final_method_of_superclass(vf, class, method)
    }
}

pub fn final_method_not_overridden(vf: &VerifierContext, method: &ClassWithLoaderMethod, super_class: &ClassWithLoader, super_method_list: &Vec<ClassWithLoaderMethod>) -> Result<(), TypeSafetyError> {
    let method_class = get_class(vf, method.prolog_class);
    let method_info = &method_class.methods[method.method_index];
    let method_name_ = method_name(&method_class, method_info);
    let descriptor_string = extract_string_from_utf8(&method_class.constant_pool[method_info.descriptor_index as usize]);
    let matching_method = super_method_list.iter().find(|x| {
        let x_method_class = get_class(vf, x.prolog_class);
        let x_method_info = &x_method_class.methods[x.method_index];
        let x_method_name = method_name(&x_method_class, x_method_info);
        let x_descriptor_string = extract_string_from_utf8(&x_method_class.constant_pool[x_method_info.descriptor_index as usize]);
        x_descriptor_string == descriptor_string && x_method_name == method_name_
    });
    match matching_method {
        None => {
            return does_not_override_final_method(vf, super_class, method);
        }
        Some(method) => {
            if is_final_method(vf, method, super_class) {
                if is_private(vf, method, super_class) || is_static(vf, method, super_class) {
                    return Result::Ok(());
                }
            } else {
                if is_private(vf, method, super_class) || is_static(vf, method, super_class) {
                    return does_not_override_final_method(vf, super_class, method);
                } else {
                    return Result::Ok(());
                }
            }
        }
    };
    Result::Err(unknown_error_verifying!())
}

pub fn does_not_override_final_method_of_superclass(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    let super_class_name = class_super_class_name(vf, class);
    let super_class = loaded_class(vf, super_class_name, vf.bootstrap_loader.clone())?;
    let super_methods_list = get_class_methods(vf, &super_class);
    final_method_not_overridden(vf, method, &super_class, &super_methods_list)
}

pub fn get_access_flags(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> u16 {
//    assert!(method.prolog_class == class);//todo why the duplicate parameters?
    get_class(vf, class).methods[method.method_index as usize].access_flags
}

//todo ClassName v. Name
pub fn is_protected(vf: &VerifierContext, super_: &ClassWithLoader, member_name: String, member_descriptor: &Descriptor) -> bool {
    let class = get_class(vf, super_);
    for method in &class.methods {
        let method_name = extract_string_from_utf8(&class.constant_pool[method.name_index as usize]);
        if member_name == method_name {
            let method_descriptor_string = extract_string_from_utf8(&class.constant_pool[method.descriptor_index as usize]);
            let parsed_member_types = match parse_method_descriptor(&super_.loader, method_descriptor_string.as_str()) {
                None => continue,
                Some(str_) => str_,
            };
            let member_types = match member_descriptor {
                Descriptor::Method(m) => m,
                _ => { continue; }
            };
            if parsed_member_types.parameter_types == member_types.parameter_types && parsed_member_types.return_type == member_types.return_type {
                return true;
            }
        }
    }
    for field in &class.fields {
        let field_name = extract_string_from_utf8(&class.constant_pool[field.name_index as usize]);
        if member_name == field_name {
            let field_descriptor_string = extract_string_from_utf8(&class.constant_pool[field.descriptor_index as usize]);
            let parsed_member_type = match parse_field_descriptor(&super_.loader, field_descriptor_string.as_str()) {
                None => continue,
                Some(str_) => str_,
            };
            let field_type = match member_descriptor {
                Descriptor::Field(f) => f,
                _ => continue
            };
            if parsed_member_type.field_type == field_type.field_type {
                return true;
            }
        }
    }
    return false;
}