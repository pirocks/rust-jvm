use std::borrow::Borrow;
use std::io;
use std::io::Write;

use class_loading::JVMState;
use classfile::{ACC_ABSTRACT, ACC_ANNOTATION, ACC_BRIDGE, ACC_ENUM, ACC_FINAL, ACC_INTERFACE, ACC_MODULE, ACC_NATIVE, ACC_PRIVATE, ACC_PROTECTED, ACC_PUBLIC, ACC_STATIC, ACC_STRICT, ACC_SUPER, ACC_SYNTHETIC, ACC_TRANSIENT, ACC_VOLATILE, AttributeInfo, Classfile, code_attribute, FieldInfo, MethodInfo};
use classfile::attribute_infos::AttributeType;
use classfile::constant_infos::{ConstantInfo, ConstantKind};
use verification::code_writer::write_parse_code_attribute;
use verification::types::{parse_field_descriptor, parse_method_descriptor, write_type_prolog};
use verification::instruction_outputer::extract_class_from_constant_pool;
use verification::types::MethodDescriptor;
use verification::types::FieldDescriptor;
use verification::verifier::PrologClass;
use verification::classnames::{ClassName, NameReference};
use std::sync::Arc;

pub struct ExtraDescriptors {
    pub extra_method_descriptors: Vec<String>,
    pub extra_field_descriptors: Vec<String>,
}

pub struct PrologGenContext<'l> {
    pub state: &'l JVMState,
    pub to_verify: Vec<Arc<Classfile>>,
    pub extra: ExtraDescriptors,
}

pub(crate) trait WritableProlog {
    fn write(w: &mut dyn Write) -> Result<(), io::Error>;
}

pub fn gen_prolog(context: &mut PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    write_class_name(context, w)?;
    write_is_interface(context, w)?;
    write_class_is_not_final(context, w)?;
    write_class_super_class_name(context, w)?;
    write_class_interfaces(context, w)?;
    write_class_methods(context, w)?;
    write_method_name(context, w)?;
    write_class_attributes(context, w)?;
    write_class_defining_loader(context, w)?;
    write_is_bootstrap_loader(context, w)?;
    write_loaded_class(context, w)?;
    write_method_access_flags(context, w)?;
    write_is_init_and_is_not_init(context, w)?;
    write_is_and_is_not_attributes_method(context, w)?;
    write_is_and_is_not_protected(context, w)?;
    write_parse_field_descriptors(context, w)?;
    write_method_descriptor(context, w)?;
    write_parse_method_descriptor(context, w)?;
    write_parse_code_attribute(context, w)?;
    write_method_attributes(context, w)?;
//    write_extra_descriptors(context, w)?;
    write_assignable_direct_supertype(context, w)?;
    w.flush()?;
    Ok(())
}

pub const BOOTSTRAP_LOADER_NAME: &str = "bl";

#[allow(dead_code)]
pub struct ParsedFieldDescriptor {
    descriptor: String,
    parsed: FieldDescriptor,
}

#[allow(dead_code)]
pub struct ParsedMethodDescriptor {
    descriptor: String,
    parsed: MethodDescriptor,
}

//#[allow(dead_code)]
//fn write_field_descriptor(fd: ParsedFieldDescriptor, w: &mut dyn Write) -> Result<(), io::Error> {
//    write!(w, "parseFieldDescriptor('{}',", fd.descriptor)?;
//    write_type_prolog(&fd.parsed.field_type, w)?;
//    write!(w, ").\n")?;
//    Ok(())
//}

//fn write_method_descriptors(md: ParsedMethodDescriptor, w: &mut dyn Write) -> Result<(), io::Error> {
//    write!(w, "parseMethodDescriptor('{}'", md.descriptor)?;
//    write!(w, ",[")?;
//    for (i, parameter_type) in md.parsed.parameter_types.iter().enumerate() {
//        write_type_prolog(&parameter_type, w)?;
//        if i != md.parsed.parameter_types.len() - 1 {
//            write!(w, ",")?;
//        }
//    }
//    write!(w, "],")?;
//    write_type_prolog(&md.parsed.return_type, w)?;
//    write!(w, ").\n")?;
//    Ok(())
//}

#[allow(unused)]
fn get_extra_descriptors(context: & PrologGenContext) -> (Vec<ParsedFieldDescriptor>, Vec<ParsedMethodDescriptor>) {
    let extra_descriptors = &context.extra;
    let mut fd = vec![];
    let mut md = vec![];
    for field_descriptor in extra_descriptors.extra_field_descriptors.iter() {
        let parsed_type = parse_field_descriptor(field_descriptor.as_str()).expect("Error parsing field descriptor");
        fd.push(ParsedFieldDescriptor { descriptor: field_descriptor.clone(), parsed: parsed_type });
    }
    for method_descriptor in extra_descriptors.extra_method_descriptors.iter() {
        //todo dup
        let parsed = parse_method_descriptor(method_descriptor.as_str()).expect("Error parsing method descriptor");
        md.push(ParsedMethodDescriptor { descriptor: method_descriptor.clone(), parsed });//todo all this copying
    }
    (fd, md)
}

//todo this should realy be two functions.
pub fn write_loaded_class(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    if context.state.using_bootstrap_loader {
        for class_file in context.to_verify.iter() {
            write!(w, "loadedClass('{}',{}, {}).\n", class_name_legacy(class_file), BOOTSTRAP_LOADER_NAME, class_prolog_name(&class_name_legacy(class_file)))?;
        }
        write!(w, "loadedClass(ClassName,_,_) :- ")?;
        for class_file in context.to_verify.iter() {
            write!(w, "ClassName \\= '{}',", class_name_legacy(class_file))?;
        }
        //todo magic string
        for class_file in context.state.loaders[&"bl".to_string()].loaded.read().unwrap().values() {
            write!(w, "ClassName \\= '{}',", class_name_legacy(class_file))?;
        }
        write!(w, "write('Need to load:'),writeln(ClassName),fail.\n")?;
    } else {
        unimplemented!()
    }
    Ok(())
}

pub fn write_is_bootstrap_loader(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    if context.state.using_bootstrap_loader {
        write!(w, "isBootstrapLoader({}).\n", BOOTSTRAP_LOADER_NAME)?;
    } else {
        unimplemented!()
    }
    Ok(())
}

pub fn write_class_defining_loader(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        if context.state.using_bootstrap_loader {
            write!(w, "classDefiningLoader({},{}).\n", class_prolog_name(&class_name_legacy(class_file)), BOOTSTRAP_LOADER_NAME)?;
        } else {
            unimplemented!();
        }
    }
    Ok(())
}


pub fn class_name(class: &Arc<Classfile>) -> ClassName {
    let class_info_entry = match &(class.constant_pool[class.this_class as usize]).kind {
        ConstantKind::Class(c) => { c }
        _ => { panic!() }
    };

    return ClassName::Ref(NameReference {
        class_file:Arc::downgrade(&class),
        index: class_info_entry.name_index
    });
}

pub fn class_name_legacy(class: &Classfile) -> String {
    let class_info_entry = match &(class.constant_pool[class.this_class as usize]).kind {
        ConstantKind::Class(c) => { c }
        _ => { panic!() }
    };
    return extract_string_from_utf8(&class.constant_pool[class_info_entry.name_index as usize]);
}

pub(crate) fn class_prolog_name(class_: &String) -> String {
    //todo , if using bootstrap loader and the like
    return format!("class('{}', {})", class_, BOOTSTRAP_LOADER_NAME);
}

// Extracts the name, ClassName , of the class Class .
//classClassName(Class, ClassName)
//todo function for class object name
fn write_class_name(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        let class_name = class_name_legacy(class_file);
        write!(w, "classClassName({},'{}').\n", class_prolog_name(&class_name), class_name)?;
    }
    Ok(())
}

fn is_interface(class: &Classfile) -> bool {
    return (class.access_flags & ACC_INTERFACE) > 0;
}

fn is_final(class: &Classfile) -> bool {
    return (class.access_flags & ACC_FINAL) > 0;
}

//classIsInterface(Class)
// True iff the class, Class , is an interface.
fn write_is_interface(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        if is_interface(class_file.borrow()) {
            write!(w, "classIsInterface({}).\n", class_prolog_name(&class_name_legacy(&class_file)))?;
        }
    }
    Ok(())
}

//classIsNotFinal(Class)

fn write_class_is_not_final(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        if !is_final(&class_file) {
            write!(w, "classIsNotFinal({}).\n", class_prolog_name(&class_name_legacy(&class_file)))?;
        }
    }
    Ok(())
}

//todo this should go at top
pub fn extract_string_from_utf8(utf8: &ConstantInfo) -> String {
    match &(utf8).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        }
        other => {
            dbg!(other);
            panic!()
        }
    }
}

pub fn get_super_class_name(class: &Classfile) -> String {
    let class_info = match &(class.constant_pool[class.super_class as usize]).kind {
        ConstantKind::Class(c) => {
            c
        }
        _ => { panic!() }
    };
    match &(class.constant_pool[class_info.name_index as usize]).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        }
        _ => { panic!() }
    }
}

fn has_super_class(class: &Classfile) -> bool {
    return class.super_class != 0;
}

//classSuperClassName(Class, SuperClassName)

fn write_class_super_class_name(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    //todo check if has super class
    for class_file in context.to_verify.iter() {
        if has_super_class(&class_file) {
            let super_class_name = get_super_class_name(&class_file);
            let base_class = class_prolog_name(&class_name_legacy(&class_file));
            write!(w, "classSuperClassName({},'{}').\n", base_class, super_class_name)?;
        }
    }
    Ok(())
}

//classInterfaces(Class, Interfaces)

// Extracts a list, Interfaces , of the direct superinterfaces of the class Class .
// Extracts the name, SuperClassName , of the superclass of class Class .
// True iff the class, Class , is not a final class.
#[allow(dead_code)]
pub(crate) struct ClassInterfaces {
    names: Vec<String>//todo?
}

fn write_class_interfaces(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        write!(w, "classInterfaces({},[", class_prolog_name(&class_name_legacy(&class_file)))?;
        for (i, interface) in class_file.interfaces.iter().enumerate() {
            //todo getrid of this kind bs
            let interface_name_index = match &class_file.constant_pool[(*interface) as usize].kind {
                ConstantKind::Class(c) => { c.name_index }
                _ => { panic!() }
            };
            let interface_name = extract_string_from_utf8(&class_file.constant_pool[interface_name_index as usize]);
            let prolog_interface_name = class_prolog_name(&interface_name);
            if i == class_file.interfaces.len() - 1 {
                write!(w, "{}", prolog_interface_name)?;
            } else {
                write!(w, "{},", prolog_interface_name)?;
            }
        }
        write!(w, "]).\n")?;
    }
    Ok(())
}

#[allow(dead_code)]
pub(crate) struct Method {
    //todo class name
    //todo method name
}

pub(crate) fn write_method_prolog_name(class_file: &Classfile, method_info: &MethodInfo, w: &mut dyn Write, suppress_class: bool) -> Result<(), io::Error> {
    let class_functor = if !suppress_class {
        class_prolog_name(&class_name_legacy(class_file))
    } else { "_".to_string() };

    write!(w, "method({},'{}',", class_functor, method_name(class_file, method_info))?;
    prolog_method_descriptor(class_file, method_info, w)?;
    write!(w, ")")?;
    Ok(())
}

//classMethods(Class, Methods)
// Extracts a list, Methods , of the methods declared in the class Class .
fn write_class_methods(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        write!(w, "classMethods({},[", class_prolog_name(&class_name_legacy(&class_file)))?;
        for (i, method_info) in class_file.methods.iter().enumerate() {
            write_method_prolog_name(&class_file, method_info, w, false)?;
            if class_file.methods.len() - 1 != i {
                write!(w, ",")?;
            }
        }
        write!(w, "]).\n")?;
    }
    Ok(())
}

//classAttributes(Class, Attributes)

fn write_attribute(attribute_info: &AttributeInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    let name = get_attribute_name(attribute_info);
    write!(w, "attribute({},uhtodo)", name)?;
    Ok(())
}

//todo
fn get_attribute_name(attribute_info: &AttributeInfo) -> String {
    return match attribute_info.attribute_type {
        AttributeType::SourceFile(_) => { "SourceFile" }
        AttributeType::InnerClasses(_) => { "InnerClasses" }
        AttributeType::EnclosingMethod(_) => { "EnclosingMethod" }
        AttributeType::SourceDebugExtension(_) => { "SourceDebugExtension" }
        AttributeType::BootstrapMethods(_) => { "BootstrapMethods" }
        AttributeType::Module(_) => { "Module" }
        AttributeType::NestHost(_) => { "NestHost" }
        AttributeType::ConstantValue(_) => { "ConstantValue" }
        AttributeType::Code(_) => { "Code" }
        AttributeType::Exceptions(_) => { "Exceptions" }
        AttributeType::RuntimeVisibleParameterAnnotations(_) => { "RuntimeVisibleParameterAnnotations" }
        AttributeType::RuntimeInvisibleParameterAnnotations(_) => { "RuntimeInvisibleParameterAnnotations" }
        AttributeType::AnnotationDefault(_) => { "AnnotationDefault" }
        AttributeType::MethodParameters(_) => { "MethodParameters" }
        AttributeType::Synthetic(_) => { "Synthetic" }
        AttributeType::Deprecated(_) => { "Deprecated" }
        AttributeType::Signature(_) => { "Signature" }
        AttributeType::RuntimeVisibleAnnotations(_) => { "RuntimeVisibleAnnotations" }
        AttributeType::RuntimeInvisibleAnnotations(_) => { "RuntimeInvisibleAnnotations" }
        AttributeType::LineNumberTable(_) => { "LineNumberTable" }
        AttributeType::LocalVariableTable(_) => { "LocalVariableTable" }
        AttributeType::LocalVariableTypeTable(_) => { "LocalVariableTypeTable" }
        AttributeType::StackMapTable(_) => { "StackMapTable" }
        AttributeType::RuntimeVisibleTypeAnnotations(_) => { "RuntimeVisibleTypeAnnotations" }
        AttributeType::RuntimeInvisibleTypeAnnotations(_) => { "RuntimeInvisibleTypeAnnotations" }
        AttributeType::NestMembers(_) => { "NestMembers" }
    }.to_string();
}

fn write_class_attributes(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        write!(w, "classAttributes({}, [", class_prolog_name(&class_name_legacy(&class_file)))?;
        for (i, attribute) in class_file.attributes.iter().enumerate() {
            write_attribute(&attribute, w)?;
            if class_file.attributes.len() - 1 != i {
                write!(w, ",")?;
            }
        }
        write!(w, "]).\n")?;
    }
    Ok(())
}

//methodName(Method, Name)
// Extracts the name, Name , of the method Method .

fn write_method_name(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for method in class_file.methods.iter() {
            write!(w, "methodName(")?;
            write_method_prolog_name(class_file, &method, w, false)?;
            write!(w, ",'{}').\n", extract_string_from_utf8(&class_file.constant_pool[method.name_index as usize]))?;
        }
    }
    Ok(())
}


//methodAccessFlags(Method, AccessFlags
//)
// Extracts the access flags, AccessFlags , of the method Method .

#[allow(unused)]
pub fn get_access_flags(class: &PrologClass,method: &::verification::verifier::PrologClassMethod) -> u16 {
//    assert!(method.prolog_class == class);//todo why the duplicate parameters?
    class.class.methods[method.method_index as usize].access_flags
}

fn before_method_access_flags(class_file: &Classfile, method_info: &MethodInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    write!(w, "methodAccessFlags(")?;
    write_method_prolog_name(class_file, method_info, w, false)?;
    Ok(())
}

fn write_method_access_flags(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter() {
            before_method_access_flags(class_file, method_info, w)?;
            write!(w, ",[ignore_this")?;
            if (method_info.access_flags & ACC_PUBLIC) > 0 {
                write!(w, ", public")?;
            }
            if (method_info.access_flags & ACC_PRIVATE) > 0 {
                write!(w, ", private")?;
            }
            if (method_info.access_flags & ACC_PROTECTED) > 0 {
                write!(w, ", protected")?;
            }
            if (method_info.access_flags & ACC_STATIC) > 0 {
                write!(w, ", static")?;
            }
            if (method_info.access_flags & ACC_FINAL) > 0 {
                write!(w, ", final")?;
            }
            if (method_info.access_flags & ACC_SUPER) > 0 {
                write!(w, ", super")?;
            }
            if (method_info.access_flags & ACC_BRIDGE) > 0 {
                write!(w, ", bridge")?;
            }
            if (method_info.access_flags & ACC_VOLATILE) > 0 {
                write!(w, ", volatile")?;//todo wrong
            }
            if (method_info.access_flags & ACC_TRANSIENT) > 0 {
                write!(w, ", transient")?;
            }
            if (method_info.access_flags & ACC_NATIVE) > 0 {
                write!(w, ", native")?;
            }
            if (method_info.access_flags & ACC_INTERFACE) > 0 {
                write!(w, ", interface")?;
            }
            if (method_info.access_flags & ACC_ABSTRACT) > 0 {
                write!(w, ", abstract")?;
            }
            if (method_info.access_flags & ACC_STRICT) > 0 {
                write!(w, ", strict")?;
            }
            if (method_info.access_flags & ACC_SYNTHETIC) > 0 {
                write!(w, ", synthetic")?;
            }
            if (method_info.access_flags & ACC_ANNOTATION) > 0 {
                write!(w, ", annotation")?;
            }
            if (method_info.access_flags & ACC_ENUM) > 0 {
                write!(w, ", enum")?;
            }
            if (method_info.access_flags & ACC_MODULE) > 0 {
                write!(w, ", module")?;
            }
            write!(w, "]).\n")?;
        }
    }
    Ok(())
}

//methodDescriptor(Method, Descriptor
//)
// Extracts the descriptor, Descriptor , of the method Method .

pub fn prolog_method_descriptor(class_file: &Classfile, method_info: &MethodInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    let descriptor = extract_string_from_utf8(&class_file.constant_pool[method_info.descriptor_index as usize]);
    write!(w, "'{}'", descriptor)?;
    Ok(())
}

pub fn method_name(class_file: &Classfile, method_info: &MethodInfo) -> String {
    let method_name_utf8 = &class_file.constant_pool[method_info.name_index as usize];
    let method_name = extract_string_from_utf8(method_name_utf8);
    method_name
}

pub fn write_method_descriptor(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter() {
            write!(w, "methodDescriptor(")?;
            write_method_prolog_name(class_file, method_info, w, false)?;
            write!(w, ",")?;
            prolog_method_descriptor(class_file, method_info, w)?;
            write!(w, ").\n")?;
        }
    }
    Ok(())
}

//methodAttributes(Method, Attributes
//)
// Extracts a list, Attributes , of the attributes of the method Method .
fn write_method_attributes(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter() {
            write!(w, "methodAttributes(")?;
            write_method_prolog_name(class_file, method_info, w, false)?;
            match code_attribute(method_info) {
                None => {
                    write!(w, ", []).\n")?;
                }
                Some(_) => {
                    write!(w, ", [attribute('Code','')]).\n", )?;
                }
            }
        }
    }
    Ok(())
}

//isInit(Method)
// True iff Method (regardless of class) is <init> .
//isNotInit(Method)
// True iff Method (regardless of class) is not <init> .
fn write_is_init_and_is_not_init(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter() {
            let method_name_info = &class_file.constant_pool[method_info.name_index as usize];
            let method_name = extract_string_from_utf8(method_name_info);
            if method_name == "<init>".to_string() {
                write!(w, "isInit(")?;
            } else {
                write!(w, "isNotInit(")?;
            }
            write_method_prolog_name(&class_file, &method_info, w, true)?;
            write!(w, ").\n")?;
        }
    }
    Ok(())
}


//isNotFinal(Method, Class)

macro_rules! write_attribute {
($flag: ident, $isCaseString: expr,$isNotCaseString:expr,$method_info:ident,$class_file:ident,$w:ident) => {
    if ($method_info.access_flags & $flag) > 0 {
        write!($w, $isCaseString)?;
        write!($w,"(")?;
    } else {
        write!($w, $isNotCaseString)?;
        write!($w,"(")?;
    }
    write_method_prolog_name($class_file, $method_info, $w,true)?;
    write!($w, ",{}).\n", class_prolog_name(&class_name_legacy($class_file)))?;
};
}

fn write_is_and_is_not_attributes_method(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter() {
            write_attribute!(ACC_FINAL,"isFinal","isNotFinal",method_info,class_file,w);
            write_attribute!(ACC_STATIC,"isStatic","isNotStatic",method_info,class_file,w);
            write_attribute!(ACC_PRIVATE,"isPrivate","isNotPrivate",method_info,class_file,w);
        }
    }
    Ok(())
}

// True iff Method in class Class is not final .
//isStatic(Method, Class)
// True iff Method in class Class is static .
//isNotStatic(Method, Class)
// True iff Method in class Class is not static .
//isPrivate(Method, Class)
// True iff Method in class Class is private .
//isNotPrivate(Method, Class)
// True iff Method in class Class is not private .

//isProtected(MemberClass, MemberName, MemberDescriptor)
// True iff there is a member named MemberName with descriptor
// MemberDescriptor in the class MemberClass and it is protected .
//isNotProtected(MemberClass, MemberName, MemberDescriptor)

pub fn prolog_field_name(class_file: &Classfile, field_info: &FieldInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    let field_name = extract_string_from_utf8(&class_file.constant_pool[field_info.name_index as usize]);
    write!(w, "field({},'{}',", class_prolog_name(&class_name_legacy(class_file)), field_name)?;
    prolog_field_descriptor(class_file, field_info, w)?;
    write!(w, ")")?;
    Ok(())
}

pub fn prolog_field_descriptor(class_file: &Classfile, field_info: &FieldInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    let descriptor = extract_string_from_utf8(&class_file.constant_pool[field_info.descriptor_index as usize]);
    write!(w, "'{}'", descriptor)?;
    Ok(())
}


// True iff there is a member named MemberName with descriptor
// MemberDescriptor in the class MemberClass and it is not protected .

pub fn write_is_and_is_not_protected(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for field_info in class_file.fields.iter() {
            if (field_info.access_flags & ACC_PROTECTED) > 0 {
                write!(w, "isProtected('{}',", class_name_legacy(class_file))?;
            } else {
                write!(w, "isNotProtected('{}',", class_name_legacy(class_file))?;
            }
            prolog_field_name(class_file, field_info, w)?;
            write!(w, ",")?;
            prolog_field_descriptor(class_file, field_info, w)?;
            write!(w, ").\n")?;
        }
    }
    Ok(())
}

//parseFieldDescriptor(Descriptor, Type
//)
// Converts a field descriptor, Descriptor , into the corresponding verification
// type Type (§4.10.1.2).

pub fn write_parse_field_descriptors(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for field_info in class_file.fields.iter() {
            write!(w, "parseFieldDescriptor(")?;
            prolog_field_descriptor(class_file, field_info, w)?;
            let descriptor_string = extract_string_from_utf8(&class_file.constant_pool[field_info.descriptor_index as usize]);
            let parsed_type = parse_field_descriptor(descriptor_string.as_str()).expect("Error parsing field descriptor");
            write!(w, ",")?;
            write_type_prolog(&parsed_type.field_type, w)?;
            write!(w, ").\n")?;
        }
    }
    Ok(())
}

//descriptor name, list of types and return type
//parseMethodDescriptor(Descriptor, ArgTypeList, ReturnType)
// Converts a method descriptor, Descriptor , into a list of verification types,
// ArgTypeList , corresponding to the method argument types, and a verification
// type, ReturnType , corresponding to the return type.


pub fn write_parse_method_descriptor(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter() {
            write!(w, "parseMethodDescriptor(")?;
            prolog_method_descriptor(class_file, method_info, w)?;
            let method_descriptor_str = extract_string_from_utf8(&class_file.constant_pool[method_info.descriptor_index as usize]);
            let method_descriptor = parse_method_descriptor(method_descriptor_str.as_str()).expect("Error parsing method descriptor");
            write!(w, ",[")?;
            for (i, parameter_type) in method_descriptor.parameter_types.iter().enumerate() {
                write_type_prolog(&parameter_type, w)?;
                if i != method_descriptor.parameter_types.len() - 1 {
                    write!(w, ",")?;
                }
            }
            write!(w, "],")?;
            write_type_prolog(&method_descriptor.return_type, w)?;
            write!(w, ").\n")?;
        }
    }
    Ok(())
}


pub fn write_assignable_direct_supertype(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    write!(w, r#"

isAssignable(X,X).

isAssignable(oneWord, top).
isAssignable(twoWord, top).

isAssignable(int, X) :- isAssignable(oneWord, X).
isAssignable(float, X) :- isAssignable(oneWord, X).
isAssignable(long, X) :- isAssignable(twoWord, X).
isAssignable(double, X) :- isAssignable(twoWord, X).

isAssignable(reference, X) :- isAssignable(oneWord, X).
isAssignable(class(_, _), X) :- isAssignable(reference, X).
isAssignable(arrayOf(_), X) :- isAssignable(reference, X).

isAssignable(uninitialized, X) :- isAssignable(reference, X).
isAssignable(uninitializedThis, X) :- isAssignable(uninitialized, X).
isAssignable(uninitialized(_), X) :- isAssignable(uninitialized, X).

isAssignable(null, class(_, _)).
isAssignable(null, arrayOf(_)).
isAssignable(null, X) :-
    isAssignable(class('java/lang/Object', BL), X),
    isBootstrapLoader(BL).


isAssignable(class(X, Lx), class(Y, Ly)) :-
    isJavaAssignable(class(X, Lx), class(Y, Ly)).

isAssignable(arrayOf(X), class(Y, L)) :-
    isJavaAssignable(arrayOf(X), class(Y, L)).

isAssignable(arrayOf(X), arrayOf(Y)) :-
    isJavaAssignable(arrayOf(X), arrayOf(Y)).

"#)?;
    for class_file in context.to_verify.iter() {
        if class_file.super_class == 0 {
            continue;
        }
        let direct_super_type_class = extract_class_from_constant_pool(class_file.super_class, class_file);
        let direct_super_type_name = class_prolog_name(&extract_string_from_utf8(&class_file.constant_pool[direct_super_type_class.name_index as usize]));
        write!(w, "isAssignable({}, X) :- isAssignable({}, X).\n", class_prolog_name(&class_name_legacy(class_file)), direct_super_type_name)?;
    }
    Ok(())
}
