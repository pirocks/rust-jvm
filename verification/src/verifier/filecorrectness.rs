use rust_jvm_common::unified_types::ClassWithLoader;
use crate::verifier::{ClassWithLoaderMethod, get_class};
use rust_jvm_common::loading::LoaderArc;
use rust_jvm_common::classfile::{ACC_STATIC, ACC_PRIVATE, ACC_INTERFACE, ACC_FINAL, ACC_PROTECTED};
use crate::verifier::TypeSafetyError;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::classfile::ConstantKind;
use rust_jvm_common::loading::LoaderName;
use crate::VerifierContext;
use std::ops::Deref;
use rust_jvm_common::unified_types::VType;
use rust_jvm_common::unified_types::PType;
use descriptor_parser::{MethodDescriptor, Descriptor, parse_field_descriptor};

pub fn different_runtime_package(vf: &VerifierContext, class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
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
}


pub fn is_bootstrap_loader(vf: &VerifierContext, loader: &LoaderArc) -> bool {
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


pub fn loaded_class(_vf: &VerifierContext, class_name: ClassName, loader: LoaderArc) -> Result<ClassWithLoader, TypeSafetyError> {
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
            VType::UninitializedThisOrClass(c2) => is_assignable(vf, &VType::Class(c.clone()), &c2.deref()),
            VType::Class(c2) => {
                if c == c2 {
                    return Result::Ok(());
                } else {
                    return is_java_assignable_class(vf, c, c2);
                }
            }
            _ => is_assignable(vf, &VType::Reference, to)
        },
        VType::ArrayReferenceType(a) => match to {
            VType::ArrayReferenceType(a2) => {
                if a == a2 {
                    return Result::Ok(());
                } else {
                    is_java_assignable(vf, from, to)
                }
            }
            //technically the next case should be partially part of is_java_assignable but is here
            VType::Class(c) => {
                if is_java_assignable(vf, from, to).is_ok() {
                    return Result::Ok(());
                }
                if !is_assignable(vf, &VType::Reference, to).is_ok() {
                    //todo okay to use name like that?
                    if c.class_name == ClassName::object() &&
                        c.loader.name() == LoaderName::BootstrapLoader {
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
            //todo really need to do something about these magic strings
            _ => is_assignable(vf, &VType::Class(ClassWithLoader { class_name: ClassName::object(), loader: vf.bootstrap_loader.clone() }), to),
        },
        VType::OneWord => match to {
            VType::OneWord => Result::Ok(()),
            VType::TopType => Result::Ok(()),
            VType::Class(c) => {
                dbg!(c);
                panic!()
//                Result::Err(unknown_error_verifying!())
            }
            VType::IntType => {
                panic!()
            }
            VType::ArrayReferenceType(_) => {
                panic!()
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
                _ => is_assignable(vf, c.deref(), to)
            }
        }
        _ => {
            dbg!(from);
            panic!("This is a bug")
        }//todo , should have a better message function
    }
}

fn atom(t: &PType) -> bool {
    match t {
        PType::ByteType |
        PType::CharType |
        PType::DoubleType |
        PType::FloatType |
        PType::IntType |
        PType::LongType |
        PType::ShortType |
        PType::VoidType |
        PType::TopType |
        PType::NullType |
        PType::UninitializedThis |
        PType::BooleanType => {
            true
        }
        PType::Ref(_) |
        PType::Uninitialized(_) |
        PType::UninitializedThisOrClass(_) => {
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
                    if c.class_name == ClassName::object() && &vf.bootstrap_loader.name() == &c.loader.name() {
                        return Result::Ok(());
                    }
                    unimplemented!()
                }
                VType::ArrayReferenceType(a2) => {
                    is_java_assignable_array_types(vf, a1, a2)
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

fn is_java_assignable_array_types(vf: &VerifierContext, left: &PType, right: &PType) -> Result<(), TypeSafetyError> {
    if atom(&left) && atom(&right) {
        if left == right {
            return Result::Ok(());
        }
    }
    if !atom(&left) && !atom(&right) {
        //todo is this bootstrap loader thing ok?
        //todo in general there needs to be a better way of handling this
        return is_java_assignable(vf, &left.to_verification_type(&vf.bootstrap_loader), &right.to_verification_type(&vf.bootstrap_loader));//todo so is this correct or does the spec handle this in full generality?
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
    ClassName::Str(utf8.extract_string_from_utf8())//todo use weak ref + index instead
}

pub fn super_class_chain(vf: &VerifierContext, chain_start: &ClassWithLoader, loader: LoaderArc, res: &mut Vec<ClassWithLoader>) -> Result<(), TypeSafetyError> {
    if chain_start.class_name == ClassName::object() {
        //todo magic constant
        if is_bootstrap_loader(vf, &loader) {
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
    let method_name_ = method_info.method_name(&method_class);
    let descriptor_string = method_info.descriptor_str(&method_class);
    let matching_method = super_method_list.iter().find(|x| {
        let x_method_class = get_class(vf, x.class);
        let x_method_info = &x_method_class.methods[x.method_index];
        let x_method_name = x_method_info.method_name(&x_method_class);
        let x_descriptor_string = x_method_info.descriptor_str(&x_method_class);
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
    //todo why the duplicate parameters?
    get_class(vf, method.class).methods[method.method_index as usize].access_flags
}

//todo ClassName v. Name
pub fn is_protected(vf: &VerifierContext, super_: &ClassWithLoader, member_name: String, member_descriptor: &Descriptor) -> bool {
    let class = get_class(vf, super_);
    for method in &class.methods {
        let method_name = method.method_name(&class);
        if member_name == method_name {
            let parsed_member_types = MethodDescriptor::from(method, &class);
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
        let field_name = class.constant_pool[field.name_index as usize].extract_string_from_utf8();
        if member_name == field_name {
            let field_descriptor_string = class.constant_pool[field.descriptor_index as usize].extract_string_from_utf8();
            let parsed_member_type = match parse_field_descriptor(field_descriptor_string.as_str()) {
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