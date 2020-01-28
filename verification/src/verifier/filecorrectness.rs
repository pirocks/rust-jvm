use std::sync::Arc;
use rust_jvm_common::unified_types::ClassWithLoader;
use crate::verifier::{ClassWithLoaderMethod, get_class};
use rust_jvm_common::loading::Loader;
use rust_jvm_common::classfile::{ACC_STATIC, ACC_PRIVATE, ACC_INTERFACE, ACC_FINAL, ACC_PROTECTED};
use crate::verifier::TypeSafetyError;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::utils::{extract_string_from_utf8, method_name};
use rust_jvm_common::classfile::ConstantKind;
use rust_jvm_common::loading::LoaderName;
use crate::VerifierContext;
use classfile_parser::types::Descriptor;
use classfile_parser::types::parse_field_descriptor;
use classfile_parser::types::parse_method_descriptor;
use std::ops::Deref;
use rust_jvm_common::unified_types::VerificationType;
use rust_jvm_common::unified_types::ParsedType;

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
    let name1 = class1.class_name.get_referred_name();
    let name2 = class2.class_name.get_referred_name();
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
        res.push(ClassWithLoaderMethod { class: class, method_index })
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
    if from.class_name.get_referred_name() == to.class_name.get_referred_name() {
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
            panic!();
//            Result::Err(unknown_error_verifying!())
        }
        Some(c) => {
            loaded_class(vf, c.class_name.clone(), to.loader.clone())?;
            Result::Ok(())
        }
    }
}

pub fn is_assignable(vf: &VerifierContext, from: &VerificationType, to: &VerificationType) -> Result<(), TypeSafetyError> {
    match from {
        VerificationType::DoubleType => match to {
            VerificationType::DoubleType => Result::Ok(()),
            _ => is_assignable(vf, &VerificationType::TwoWord, to)
        },
        VerificationType::LongType => match to {
            VerificationType::LongType => Result::Ok(()),
            _ => is_assignable(vf, &VerificationType::TwoWord, to)
        },
        VerificationType::FloatType => match to {
            VerificationType::FloatType => Result::Ok(()),
            _ => is_assignable(vf, &VerificationType::OneWord, to)
        },
        VerificationType::IntType => match to {
            VerificationType::IntType => Result::Ok(()),
            _ => is_assignable(vf, &VerificationType::OneWord, to)
        },
        VerificationType::Reference => match to {
            VerificationType::Reference => Result::Ok(()),
            _ => is_assignable(vf, &VerificationType::OneWord, to)
        }
        VerificationType::Class(c) => match to {
            VerificationType::Class(c2) => {
                if c == c2 {
                    return Result::Ok(());
                } else {
                    return is_java_assignable_class(vf, c, c2);
                }
            }
            _ => is_assignable(vf, &VerificationType::Reference, to)
        },
        VerificationType::ArrayReferenceType(a) => match to {
            VerificationType::ArrayReferenceType(a2) => {
                if a == a2 {
                    return Result::Ok(());
                } else {
//                    dbg!(a);
//                    dbg!(a2);
                    is_java_assignable(vf, from, to)
                }
            }
            //technically the next case should be partially part of is_java_assignable but is here
            VerificationType::Class(c) => {
                if is_java_assignable(vf, from, to).is_ok() {
                    return Result::Ok(());
                }
                if !is_assignable(vf, &VerificationType::Reference, to).is_ok() {
                    //todo okay to use name like that?
                    if c.class_name == ClassName::object() &&
                        c.loader.name() == LoaderName::BootstrapLoader {
                        return Result::Ok(());
                    }
                }
                is_assignable(vf, &VerificationType::Reference, to)
            }
            _ => is_assignable(vf, &VerificationType::Reference, to)
        },
        VerificationType::TopType => match to {
            VerificationType::TopType => Result::Ok(()),
            _ => panic!("This might be a bug. It's a weird edge case"),
        },
        VerificationType::UninitializedEmpty => match to {
            VerificationType::UninitializedEmpty => Result::Ok(()),
            _ => is_assignable(vf, &VerificationType::Reference, to)
        },
        VerificationType::Uninitialized(u1) => match to {
            VerificationType::Uninitialized(u2) => {
                if u1.offset == u2.offset {
                    return Result::Ok(());
                }
                is_assignable(vf, &VerificationType::UninitializedEmpty, to)
            }
            _ => is_assignable(vf, &VerificationType::UninitializedEmpty, to)
        },
        VerificationType::UninitializedThis => match to {
            VerificationType::UninitializedThis => Result::Ok(()),
            _ => is_assignable(vf, &VerificationType::UninitializedEmpty, to)
        },
        VerificationType::NullType => match to {
            VerificationType::NullType => Result::Ok(()),
            VerificationType::Class(_) => Result::Ok(()),
            VerificationType::ArrayReferenceType(_) => Result::Ok(()),
            //todo really need to do something about these magic strings
            _ => is_assignable(vf, &VerificationType::Class(ClassWithLoader { class_name: ClassName::object(), loader: vf.bootstrap_loader.clone() }), to),
        },
        VerificationType::OneWord => match to {
            VerificationType::OneWord => Result::Ok(()),
            VerificationType::TopType => Result::Ok(()),
            VerificationType::Class(c) => {
                dbg!(c);
                panic!()
            }
            VerificationType::IntType => {
                panic!()
            }
            _ => {
//                dbg!(to);
                Result::Err(unknown_error_verifying!())
            }
        },
        VerificationType::TwoWord => match to {
            VerificationType::TwoWord => Result::Ok(()),
            VerificationType::TopType => Result::Ok(()),
            _ => {
                dbg!(to);
                Result::Err(unknown_error_verifying!())
            }
        },
        _ => {
            dbg!(from);
            panic!("This is a bug")
        }//todo , should have a better message function
    }
}

fn atom(t: &ParsedType) -> bool {
    match t {
        ParsedType::ByteType |
        ParsedType::CharType |
        ParsedType::DoubleType |
        ParsedType::FloatType |
        ParsedType::IntType |
        ParsedType::LongType |
        ParsedType::ShortType |
        ParsedType::VoidType |
        ParsedType::TopType |
        ParsedType::NullType |
        ParsedType::UninitializedThis |
//        ParsedType::TwoWord |
//        ParsedType::OneWord |
//        ParsedType::Reference |
//        ParsedType::UninitializedEmpty |
        ParsedType::BooleanType => {
            true
        }
        ParsedType::Class(_) |
        ParsedType::ArrayReferenceType(_) |
        ParsedType::Uninitialized(_) => {
            false
        }
    }
}

fn is_java_assignable(vf: &VerifierContext, left: &VerificationType, right: &VerificationType) -> Result<(), TypeSafetyError> {
    match left {
        VerificationType::Class(c1) => {
            match right {
                VerificationType::Class(c2) => {
                    is_java_assignable_class(vf, c1, c2)
                }
                VerificationType::ArrayReferenceType(_a) => {
                    unimplemented!()
                }
                _ => unimplemented!()
            }
        }
        VerificationType::ArrayReferenceType(a1) => {
            match right {
                VerificationType::Class(c) => {
                    if c.class_name == ClassName::object() && &vf.bootstrap_loader.name() == &c.loader.name() {
                        return Result::Ok(());
                    }
                    unimplemented!()
                }
                VerificationType::ArrayReferenceType(a2) => {
                    is_java_assignable_array_types(vf, a1.sub_type.deref(), a2.sub_type.deref())
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

fn is_java_assignable_array_types(vf: &VerifierContext, left: &ParsedType, right: &ParsedType) -> Result<(), TypeSafetyError> {
    if atom(&left) && atom(&right) {
        if left == right {
            return Result::Ok(());
        }
    }
    if !atom(&left) && !atom(&right) {
        return is_java_assignable(vf, &left.to_verification_type(), &right.to_verification_type());//todo so is this correct or does the spec handle this in full generality?
    }
    Result::Err(unknown_error_verifying!())
}

fn is_java_assignable_class(vf: &VerifierContext, from: &ClassWithLoader, to: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    loaded_class(vf, to.class_name.clone(), to.loader.clone())?;
    if class_is_interface(vf, &ClassWithLoader { class_name: to.class_name.clone(), loader: to.loader.clone() }) {
        return Result::Ok(());
    }
    return is_java_sub_class_of(vf, from, to);
}

pub fn is_array_interface(_vf: &VerifierContext, class: ClassWithLoader) -> bool {
    class.class_name.get_referred_name() == "java/lang/Cloneable" ||
        class.class_name.get_referred_name() == "java/io/Serializable"
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
    if chain_start.class_name == ClassName::object() {
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
//    assert!(class == method.class);
    (get_access_flags(vf, class, method) & ACC_STATIC) > 0
}

pub fn is_private(vf: &VerifierContext, method: &ClassWithLoaderMethod, class: &ClassWithLoader) -> bool {
    //todo check if method class and class same
//    assert!(class == method.class);
    (get_access_flags(vf, class, method) & ACC_PRIVATE) > 0
}

pub fn does_not_override_final_method(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    if class.class_name == ClassName::object() {
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
    let method_class = get_class(vf, method.class);
    let method_info = &method_class.methods[method.method_index];
    let method_name_ = method_name(&method_class, method_info);
    let descriptor_string = extract_string_from_utf8(&method_class.constant_pool[method_info.descriptor_index as usize]);
    let matching_method = super_method_list.iter().find(|x| {
        let x_method_class = get_class(vf, x.class);
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

pub fn get_access_flags(vf: &VerifierContext, _class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> u16 {
//    assert!(method.prolog_class == class);//todo why the duplicate parameters?
    get_class(vf, method.class).methods[method.method_index as usize].access_flags
}

//todo ClassName v. Name
pub fn is_protected(vf: &VerifierContext, super_: &ClassWithLoader, member_name: String, member_descriptor: &Descriptor) -> bool {
//    dbg!(super_);
//    dbg!(&member_name);
//    dbg!(&member_descriptor);
    let class = get_class(vf, super_);
    for method in &class.methods {
        let method_name = extract_string_from_utf8(&class.constant_pool[method.name_index as usize]);
        if member_name == method_name {
            let method_descriptor_string = extract_string_from_utf8(&class.constant_pool[method.descriptor_index as usize]);
            let parsed_member_types = match parse_method_descriptor(&super_.loader, method_descriptor_string.as_str()) {
                None => panic!(),
                Some(str_) => str_,
            };
            let member_types = match member_descriptor {
                Descriptor::Method(m) => m,
                _ => { panic!(); }
            };
            if parsed_member_types.parameter_types == member_types.parameter_types && parsed_member_types.return_type == member_types.return_type {
                if (method.access_flags & ACC_PROTECTED) > 0 {
                    return true;
                } else {
                    return false;
                }
            }
        }
    }
    for field in &class.fields {
        let field_name = extract_string_from_utf8(&class.constant_pool[field.name_index as usize]);
        if member_name == field_name {
            let field_descriptor_string = extract_string_from_utf8(&class.constant_pool[field.descriptor_index as usize]);
            let parsed_member_type = match parse_field_descriptor(&super_.loader, field_descriptor_string.as_str()) {
                None => panic!(),
                Some(str_) => str_,
            };
            let field_type = match member_descriptor {
                Descriptor::Field(f) => f,
                _ => panic!()
            };
            if parsed_member_type.field_type == field_type.field_type {
                if (field.access_flags & ACC_PROTECTED) > 0 {
                    return true;
                } else {
                    return false;
                }
            }
        }
    }
    panic!()
}