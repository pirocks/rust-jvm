use std::ops::Deref;

use classfile_view::view::HasAccessFlags;
use rust_jvm_common::compressed_classfile::{CCString, CPDType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::descriptor_parser::Descriptor;
use rust_jvm_common::loading::*;
use rust_jvm_common::loading::LoaderName::BootstrapLoader;
use rust_jvm_common::vtype::VType;

use crate::verifier::{ClassWithLoaderMethod, get_class};
use crate::verifier::TypeSafetyError;
use crate::VerifierContext;

pub fn different_runtime_package(vf: &VerifierContext, class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
    class1.loader != class2.loader ||
        different_package_name(vf, class1, class2)
}

fn different_package_name(vf: &VerifierContext, class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
    let name1 = vf.string_pool.lookup(class1.class_name.0);
    let name2 = vf.string_pool.lookup(class2.class_name.0);
    let split1: Vec<&str> = name1.split('/').collect();
    let split2: Vec<&str> = name2.split('/').collect();
    assert!(!split1.is_empty());
    assert!(!split2.is_empty());
    let package_slice1 = &split1[..split1.len() - 1];
    let package_slice2 = &split2[..split2.len() - 1];
    package_slice1.iter().zip(package_slice2.iter()).all(|(a, b)| {
        a == b
    })
}


pub fn is_bootstrap_loader(loader: &LoaderName) -> bool {
    loader == &BootstrapLoader
}

pub fn get_class_methods(vf: &VerifierContext, class: ClassWithLoader) -> Vec<ClassWithLoaderMethod> {
    let mut res = vec![];
    for method_index in 0..get_class(vf, &class).num_methods() {
        res.push(ClassWithLoaderMethod { class: class.clone(), method_index })
    }
    res
}

pub fn class_is_final(vf: &VerifierContext, class: &ClassWithLoader) -> bool {
    get_class(vf, class).is_final()
}


pub fn loaded_class(_vf: &VerifierContext, class_name: CClassName, loader: LoaderName) -> Result<ClassWithLoader, TypeSafetyError> {
    Result::Ok(ClassWithLoader { class_name, loader })
    // if vf.classes.class_loaded_by(&class_name, &loader) {
    //     Result::Ok(ClassWithLoader { class_name, loader })
    // } else {
    //     match vf.classes.pre_load(class_name.clone(), loader.clone()) {
    //         Ok(_) => Result::Ok(ClassWithLoader { class_name, loader }),
    //         Err(_) => unimplemented!(),
    //     }
    // }
}


pub fn class_is_interface(vf: &VerifierContext, class: &ClassWithLoader) -> bool {
    get_class(vf, class).is_interface()
}

pub fn is_java_sub_class_of(vf: &VerifierContext, from: &ClassWithLoader, to: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    if from.class_name == to.class_name {
        return Result::Ok(());
    }
    let mut chain = vec![];
    super_class_chain(vf, &loaded_class(vf, from.class_name.clone(), from.loader.clone())?, from.loader.clone(), &mut chain)?;
    match chain.iter().find(|x| {
        x.class_name == to.class_name
    }) {
        None => {
            // dbg!(&from.class_name);
            // dbg!(&to.class_name);
            // dbg!(&chain);
            // panic!();
            Result::Err(unknown_error_verifying!())
        }
        Some(c) => {
            loaded_class(vf, c.class_name.clone(), to.loader.clone())?;
            Result::Ok(())
        }
    }
}


//todo why is this in this file?
pub fn is_assignable(vf: &VerifierContext, from: &VType, to: &VType) -> Result<(), TypeSafetyError> {
    match from {
        VType::DoubleType => match to {
            VType::DoubleType => Result::Ok(()),
            _ => is_assignable(vf, &VType::TwoWord, to)
        },
        VType::LongType => match to {
            VType::LongType => Result::Ok(()),
            _ => is_assignable(vf, &VType::TwoWord, to)
        },
        VType::FloatType => match to {
            VType::FloatType => Result::Ok(()),
            _ => is_assignable(vf, &VType::OneWord, to)
        },
        VType::IntType => match to {
            VType::IntType => Result::Ok(()),
            _ => is_assignable(vf, &VType::OneWord, to)
        },
        VType::Reference => match to {
            VType::Reference => Result::Ok(()),
            _ => is_assignable(vf, &VType::OneWord, to)
        }
        VType::Class(c) => match to {
            VType::UninitializedThisOrClass(c2) => is_assignable(vf, &VType::Class(c.clone()), &c2.to_verification_type(BootstrapLoader)),//todo bootstrap loader
            VType::Class(c2) => {
                if c == c2 {
                    Result::Ok(())
                } else {
                    is_java_assignable_class(vf, c, c2)
                }
            }
            _ => is_assignable(vf, &VType::Reference, to)
        },
        VType::ArrayReferenceType(a) => match to {
            VType::ArrayReferenceType(a2) => {
                if a == a2 {
                    Result::Ok(())
                } else {
                    is_java_assignable(vf, from, to)
                }
            }
            //technically the next case should be partially part of is_java_assignable but is here
            VType::Class(c) => {
                if is_java_assignable(vf, from, to).is_ok() {
                    return Result::Ok(());
                }
                if is_assignable(vf, &VType::Reference, to).is_err() {
                    if c.class_name == CClassName::object() &&
                        c.loader == LoaderName::BootstrapLoader {
                        return Result::Ok(());
                    }
                }
                is_assignable(vf, &VType::Reference, to)
            }
            _ => is_assignable(vf, &VType::Reference, to)
        },
        VType::TopType => match to {
            VType::TopType => Result::Ok(()),
            _ => panic!("This might be a bug. It's a weird edge case"),
        },
        VType::UninitializedEmpty => match to {
            VType::UninitializedEmpty => Result::Ok(()),
            _ => is_assignable(vf, &VType::Reference, to)
        },
        VType::Uninitialized(u1) => match to {
            VType::Uninitialized(u2) => {
                if u1.offset == u2.offset {
                    return Result::Ok(());
                }
                is_assignable(vf, &VType::UninitializedEmpty, to)
            }
            _ => is_assignable(vf, &VType::UninitializedEmpty, to)
        },
        VType::UninitializedThis => match to {
            VType::UninitializedThis => Result::Ok(()),
            VType::UninitializedThisOrClass(_) => Result::Ok(()),
            _ => is_assignable(vf, &VType::UninitializedEmpty, to)
        },
        VType::NullType => match to {
            VType::NullType => Result::Ok(()),
            VType::Class(_) => Result::Ok(()),
            VType::ArrayReferenceType(_) => Result::Ok(()),
            _ => is_assignable(vf, &VType::Class(ClassWithLoader { class_name: CClassName::object(), loader: vf.current_loader.clone() }), to),
        },
        VType::OneWord => match to {
            VType::OneWord => Result::Ok(()),
            VType::TopType => Result::Ok(()),
            VType::Class(_) => {
                Result::Err(unknown_error_verifying!())
            }
            _ => {
                Result::Err(unknown_error_verifying!())
            }
        },
        VType::TwoWord => match to {
            VType::TwoWord => Result::Ok(()),
            VType::TopType => Result::Ok(()),
            _ => {
                dbg!(to);
                Result::Err(unknown_error_verifying!())
            }
        },
        VType::UninitializedThisOrClass(c) => {
            match to {
                VType::UninitializedThis => Result::Ok(()),
                _ => is_assignable(vf, &c.to_verification_type(BootstrapLoader), to)//todo bootstrap loader
            }
        }
        _ => {
            dbg!(from);
            panic!("This is a bug")
        }
    }
}

fn atom(t: &CPDType) -> bool {
    match t {
        CPDType::ByteType |
        CPDType::CharType |
        CPDType::DoubleType |
        CPDType::FloatType |
        CPDType::IntType |
        CPDType::LongType |
        CPDType::ShortType |
        CPDType::VoidType |
        CPDType::BooleanType => {
            true
        }
        CPDType::Ref(_) => {
            false
        }
    }
}

fn is_java_assignable(vf: &VerifierContext, left: &VType, right: &VType) -> Result<(), TypeSafetyError> {
    match left {
        VType::Class(c1) => {
            match right {
                VType::Class(c2) => {
                    is_java_assignable_class(vf, c1, c2)
                }
                VType::ArrayReferenceType(_a) => {
                    unimplemented!()
                }
                _ => unimplemented!()
            }
        }
        VType::ArrayReferenceType(a1) => {
            match right {
                VType::Class(c) => {
                    if c.class_name == CClassName::object() && vf.current_loader == c.loader {
                        return Result::Ok(());
                    }
                    unimplemented!()
                }
                VType::ArrayReferenceType(a2) => {
                    is_java_assignable_array_types(vf, a1.clone(), a2.clone())
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

fn is_java_assignable_array_types(vf: &VerifierContext, left: CPDType, right: CPDType) -> Result<(), TypeSafetyError> {
    if atom(&left) && atom(&right) && left == right {
        return Result::Ok(());
    }
    if !atom(&left) && !atom(&right) {
        //todo is this bootstrap loader thing ok?
        //todo in general there needs to be a better way of handling this
        return is_java_assignable(vf, &left.to_verification_type(vf.current_loader), &right.to_verification_type(vf.current_loader));//todo so is this correct or does the spec handle this in full generality?
    }
    Result::Err(unknown_error_verifying!())
}

fn is_java_assignable_class(vf: &VerifierContext, from: &ClassWithLoader, to: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    loaded_class(vf, to.class_name.clone(), to.loader.clone())?;
    if class_is_interface(vf, &ClassWithLoader { class_name: to.class_name.clone(), loader: to.loader.clone() }) {
        return Result::Ok(());
    }
    is_java_sub_class_of(vf, from, to)
}

pub fn is_array_interface(_vf: &VerifierContext, class: ClassWithLoader) -> bool {
    class.class_name == CClassName::cloneable() ||
        class.class_name == CClassName::serializable()
}

pub fn is_java_subclass_of(_vf: &VerifierContext, _sub: &ClassWithLoader, _super: &ClassWithLoader) {
    unimplemented!()
}

pub fn class_super_class_name(vf: &VerifierContext, class: &ClassWithLoader) -> CClassName {
    //todo dup, this must exist elsewhere
    let classfile = get_class(vf, class);
    classfile.super_name().unwrap()
}

pub fn super_class_chain(vf: &VerifierContext, chain_start: &ClassWithLoader, loader: LoaderName, res: &mut Vec<ClassWithLoader>) -> Result<(), TypeSafetyError> {
    if chain_start.class_name == CClassName::object() {
        //todo magic constant
        return Ok(());
        //todo need to still sorta do this check
        // if is_bootstrap_loader(&loader) {
        //     return Result::Ok(());
        // } else {
        //     return Result::Err(TypeSafetyError::NotSafe("java/lang/Object superclasschain failed. This is bad and likely unfixable.".to_string()));
        // }
    }
    let class = loaded_class(vf, chain_start.class_name.clone(), loader.clone())?;//todo loader duplication
    let super_class_name = class_super_class_name(vf, &class);
    let super_class = loaded_class(vf, super_class_name.clone(), loader.clone())?;
    res.push(super_class);
    super_class_chain(vf, &loaded_class(vf, super_class_name, loader.clone())?, loader, res)?;
    Result::Ok(())
}


pub fn is_final_method(vf: &VerifierContext, method: &ClassWithLoaderMethod, _class: &ClassWithLoader) -> bool {
    //todo check if same
    get_class(vf, &method.class).method_view_i(method.method_index as u16).is_final()
}


pub fn is_static(vf: &VerifierContext, method: &ClassWithLoaderMethod, _class: &ClassWithLoader) -> bool {
    //todo check if same
    get_class(vf, &method.class).method_view_i(method.method_index as u16).is_static()
}

pub fn is_private(vf: &VerifierContext, method: &ClassWithLoaderMethod, _class: &ClassWithLoader) -> bool {
    //todo check if method class and class same
//    assert!(class == &method.class);
    get_class(vf, &method.class).method_view_i(method.method_index as u16).is_private()
}

pub fn does_not_override_final_method(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    if class.class_name == CClassName::object() {
        return Ok(());
        // if is_bootstrap_loader(&class.loader) {
        //     Result::Ok(())
        // } else {
        //     Result::Err(TypeSafetyError::NotSafe("Loading Object w/o bootstrap loader".to_string()))
        // }
    } else if is_private(vf, method, class) || is_static(vf, method, class) {
        Result::Ok(())
    } else {
        does_not_override_final_method_of_superclass(vf, class, method)
    }
}

pub fn final_method_not_overridden(
    vf: &VerifierContext,
    method: &ClassWithLoaderMethod,
    super_class: &ClassWithLoader,
    super_method_list: &[ClassWithLoaderMethod],
) -> Result<(), TypeSafetyError> {
    let method_class = get_class(vf, &method.class);
    let method_info = &method_class.method_view_i(method.method_index as u16);
    let method_name_ = method_info.name();
    let descriptor_string = method_info.desc_str();
    //todo this stuff needs indexing. The below is guilty of 3% total init time.
    let matching_method = super_method_list.iter().find(|x| {
        let x_method_class = get_class(vf, &x.class);
        let x_method_info = &x_method_class.method_view_i(x.method_index as u16);
        let x_method_name = x_method_info.name();
        let x_descriptor_string = x_method_info.desc_str();
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
                return if is_private(vf, method, super_class) || is_static(vf, method, super_class) {
                    does_not_override_final_method(vf, super_class, method)
                } else {
                    Result::Ok(())
                };
            }
        }
    };
    Result::Err(unknown_error_verifying!())
}

pub fn does_not_override_final_method_of_superclass(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    let super_class_name = class_super_class_name(vf, class);
    let super_class = loaded_class(vf, super_class_name, vf.current_loader.clone())?;
    let super_methods_list = get_class_methods(vf, super_class.clone());
    final_method_not_overridden(vf, method, &super_class, &super_methods_list)
}

pub fn get_access_flags(vf: &VerifierContext, _class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> u16 {
    //todo why the duplicate parameters?
    get_class(vf, &method.class).method_view_i(method.method_index as u16).access_flags()
}

//todo ClassName v. Name
pub fn is_protected(vf: &VerifierContext, super_: &ClassWithLoader, member_name: CCString, member_descriptor: &Descriptor) -> bool {
    let class = get_class(vf, super_);
    for method in class.methods() {
        let method_name = method.name();
        if member_name == method_name.0 {
            let parsed_member_types = method.desc();
            let member_types = match member_descriptor {
                Descriptor::Method(m) => m,
                _ => { panic!(); }
            };
            /*if &parsed_member_types.arg_types == &member_types.arg_types && parsed_member_types.return_type == member_types.return_type {
                return method.is_protected();
            }*/
            todo!()
        }
    }
    for field in class.fields() {
        let field_name = field.field_name();
        if member_name == field_name.0 {
            let parsed_member_type = field.field_type();
            let field_type = match member_descriptor {
                Descriptor::Field(f) => f,
                _ => panic!()
            };
            if parsed_member_type == field_type.0 {
                return field.is_protected();
            }
        }
    }
    panic!()
}