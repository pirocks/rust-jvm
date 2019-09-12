use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::io;

use bimap::BiMap;

use classfile::{ACC_ABSTRACT, ACC_ANNOTATION, ACC_BRIDGE, ACC_ENUM, ACC_FINAL, ACC_INTERFACE, ACC_MODULE, ACC_NATIVE, ACC_PRIVATE, ACC_PROTECTED, ACC_PUBLIC, ACC_STATIC, ACC_STRICT, ACC_SUPER, ACC_SYNTHETIC, ACC_TRANSIENT, ACC_VOLATILE, AttributeInfo, Classfile, FieldInfo, MethodInfo};
use classfile::attribute_infos::{AttributeType};
use classfile::constant_infos::{ConstantInfo, ConstantKind};
use verification::types::{parse_field_descriptor, parse_method_descriptor, write_type_prolog};
use verification::code_verification::write_parse_code_attribute;

pub fn verify(_c : &Classfile){

}

/**
loadedClass(Name, InitiatingLoader, ClassDefinition)
True iff there exists a class named Name whose representation (in accordance
with this specification) when loaded by the class loader InitiatingLoader is
methodDescriptor(Method, Descriptor)
Extracts the descriptor, Descriptor , of the method Method .
methodAttributes(Method, Attributes)
Extracts a list, Attributes , of the attributes of the method Method .
isProtected(MemberClass, MemberName, MemberDescriptor)
True iff there is a member named MemberName with descriptor
MemberDescriptor in the class MemberClass and it is protected .
isNotProtected(MemberClass, MemberName, MemberDescriptor)
True iff there is a member named MemberName with descriptor
MemberDescriptor in the class MemberClass and it is not protected .
parseFieldDescriptor(Descriptor, Type)
Converts a field descriptor, Descriptor , into the corresponding verification
type Type (§4.10.1.2).
1954.10
Verification of class Files
THE CLASS FILE FORMAT
parseMethodDescriptor(Descriptor, ArgTypeList, ReturnType)
Converts a method descriptor, Descriptor , into a list of verification types,
ArgTypeList , corresponding to the method argument types, and a verification
type, ReturnType , corresponding to the return type.
parseCodeAttribute(Class, Method, FrameSize, MaxStack, ParsedCode,
Handlers, StackMap)
Extracts the instruction stream, ParsedCode , of the method Method in Class ,
as well as the maximum operand stack size, MaxStack , the maximal number
of local variables, FrameSize , the exception handlers, Handlers , and the stack
map StackMap .
The representation of the instruction stream and stack map attribute must be as
specified in §4.10.1.3 and §4.10.1.4.
samePackageName(Class1, Class2)
True iff the package names of Class1 and Class2 are the same.
differentPackageName(Class1, Class2)
True iff the package names of Class1 and Class2 are different.

*/

pub mod prolog_initial_defs;

pub struct PrologGenContext {
    pub using_bootsrap_loader: bool,
    pub to_load: Vec<Classfile>,
    pub name_to_classfile: BiMap<String, Classfile>,//todo need to init
    pub existing_classfiles: HashMap<Box<String>, Box<Classfile>>
}

pub fn gen_prolog<S: Write>(context: &PrologGenContext, w :&mut BufWriter<S>  )-> Result<(),io::Error>{
    write_class_name(context,w)?;
    write_is_interface(context,w)?;
    write_class_is_not_final(context,w)?;
    write_class_super_class_name(context,w)?;
    write_class_interfaces(context,w)?;
    write_class_methods(context,w)?;
    write_method_name(context,w)?;
    write_class_attributes(context,w)?;
    write_class_defining_loader(context, w)?;
    write_is_bootstrap_loader(context, w)?;
    write_loaded_class(context, w)?;
    write_method_access_flags(context,w)?;
    write_is_init_and_is_not_init(context,w)?;
    write_is_and_is_not_attributes_method(context, w)?;
    write_is_and_is_not_protected(context,w)?;
    write_parse_field_descriptor(context,w)?;
    write_parse_method_descriptor(context,w)?;
    write_parse_code_attribute(context,w)?;
    Ok(())
}

const BOOTSTRAP_LOADER_NAME: &str = "bootstraploader";

pub fn write_loaded_class(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    if context.using_bootsrap_loader {
        let to_load_classes = context.to_load.iter();
        let already_loaded_classes =context.existing_classfiles.iter();
        for class_file in to_load_classes {
            write!(w,"loadedClass({},{}, ClassDefinition).\n", class_name(class_file), BOOTSTRAP_LOADER_NAME )?;
        }
        for (_,class_file) in already_loaded_classes{
            write!(w,"loadedClass({},{}, ClassDefinition).\n", class_name(class_file), BOOTSTRAP_LOADER_NAME )?;//todo duplication
        }
    } else {
        unimplemented!()
    }
    Ok(())
}

pub fn write_is_bootstrap_loader(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    if context.using_bootsrap_loader {
        write!(w, "isBootstrapLoader({})", BOOTSTRAP_LOADER_NAME)?;
    }
    else{
        unimplemented!()
    }
    Ok(())
}


pub fn write_class_defining_loader(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_load.iter() {
        if context.using_bootsrap_loader {
            write!(w, "classDefiningLoader({},{}).\n", class_name(class_file), BOOTSTRAP_LOADER_NAME)?;
        } else {
            unimplemented!();
        }
    }
    Ok(())
}

pub fn class_name(class: &Classfile) -> String {
    let class_info_entry = match &(class.constant_pool[class.this_class as usize]).kind {
        ConstantKind::Class(c) => { c }
        _ => { panic!() }
    };
    return extract_string_from_utf8(&class.constant_pool[class_info_entry.name_index as usize]);

}

fn class_prolog_name(class_: &String) -> String {
    let mut base = "prolog_name__".to_string();
    base.push_str(class_.replace("/","__").as_str());
    return base
}

// Extracts the name, ClassName , of the class Class .
//classClassName(Class, ClassName)
//todo function for class object name
fn write_class_name(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error> {
    for class_file in context.to_load.iter() {
        let class_name = class_name(class_file);
        write!(w, "classClassName({},'{}').\n", class_prolog_name(&class_name), class_name)?;
    }
    Ok(())
}

fn is_interface(class: &Classfile) -> bool {
    return (class.access_flags & ACC_INTERFACE) > 0
}

fn is_final(class: &Classfile) -> bool {
    return (class.access_flags & ACC_FINAL) > 0
}

//classIsInterface(Class)
// True iff the class, Class , is an interface.
fn write_is_interface(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error> {
    for class_file in context.to_load.iter() {
        if is_interface(class_file.borrow()) {
            write!(w, "classIsInterface({}).\n", class_prolog_name(&class_name(&class_file)))?;
        }
    }
    Ok(())
}

//classIsNotFinal(Class)
// True iff the class, Class , is not a final class.

fn write_class_is_not_final(context: &PrologGenContext, w: &mut dyn Write)-> Result<(),io::Error> {
    for class_file in context.to_load.iter() {
        if !is_final(&class_file) {
            write!(w, "classIsNotFinal({}).\n", class_prolog_name(&class_name(&class_file)))?;
        }
    }
    Ok(())
}

//todo this should go at top
pub fn extract_string_from_utf8(utf8: &ConstantInfo) -> String {
    match &(utf8).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        },
        _ => { panic!() }
    }
}

pub fn get_super_class_name(class: &Classfile) -> String {
    let class_info = match &(class.constant_pool[class.super_class as usize]).kind {
        ConstantKind::Class(c) => {
            c
        },
        _ => { panic!() }
    };
    match &(class.constant_pool[class_info.name_index as usize]).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        },
        _ => { panic!() }
    }
}

fn has_super_class(class: &Classfile) -> bool {
    return class.super_class != 0;
}

//classSuperClassName(Class, SuperClassName)
// Extracts the name, SuperClassName , of the superclass of class Class .

fn write_class_super_class_name(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error> {
    //todo check if has super class
    for class_file in context.to_load.iter() {
        if has_super_class(&class_file) {
            let super_class_name = get_super_class_name(&class_file);
            let base_class = class_prolog_name(&class_name(&class_file));
            write!(w, "classSuperClassName({},'{}').\n", base_class, super_class_name)?;
        }
    }
    Ok(())
}

//classInterfaces(Class, Interfaces)
// Extracts a list, Interfaces , of the direct superinterfaces of the class Class .

fn write_class_interfaces(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error> {
    for class_file in context.to_load.iter() {
        write!(w, "classInterfaces({},[", class_prolog_name(&class_name(&class_file)))?;
        for (i, interface) in class_file.interfaces.iter().enumerate() {
            let interface_name = extract_string_from_utf8(&class_file.constant_pool[*interface as usize]);
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


fn write_method_prolog_name(class_file: &Classfile, method_info: &MethodInfo, w: &mut dyn Write)-> Result<(),io::Error> {
    let method_name_utf8 = &class_file.constant_pool[method_info.name_index as usize];
    let method_name = extract_string_from_utf8(method_name_utf8).replace("<init>","__init");
    write!(w, "prolog_name__{}__Method_{}", class_prolog_name(&class_name(class_file)), method_name)?;
    Ok(())
}

//classMethods(Class, Methods)
// Extracts a list, Methods , of the methods declared in the class Class .
fn write_class_methods(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error>{
    for class_file in context.to_load.iter() {
        write!(w, "classMethods({},[", class_prolog_name(&class_name(&class_file)))?;
        for (i, method_info) in class_file.methods.iter().enumerate() {
            write_method_prolog_name(&class_file, method_info, w)?;
            if class_file.methods.len() - 1 != i {
                write!(w, ",")?;
            }
        }
        write!(w, "]).\n")?;
    }
    Ok(())
}

//classAttributes(Class, Attributes)

fn write_attribute(attribute_info: &AttributeInfo, w: &mut dyn Write)-> Result<(),io::Error> {
    let name = get_attribute_name(attribute_info);
    write!(w, "attribute({},uhtodo)", name)?;
    Ok(())
}

//todo
fn get_attribute_name(attribute_info: &AttributeInfo) -> String {
    return match attribute_info.attribute_type{
        AttributeType::SourceFile(_) => {"SourceFile"},
        AttributeType::InnerClasses(_) => {"InnerClasses"},
        AttributeType::EnclosingMethod(_) => {"EnclosingMethod"},
        AttributeType::SourceDebugExtension(_) => {"SourceDebugExtension"},
        AttributeType::BootstrapMethods(_) => {"BootstrapMethods"},
        AttributeType::Module(_) => {"Module"},
        AttributeType::NestHost(_) => {"NestHost"},
        AttributeType::ConstantValue(_) => {"ConstantValue"},
        AttributeType::Code(_) => {"Code"},
        AttributeType::Exceptions(_) => {"Exceptions"},
        AttributeType::RuntimeVisibleParameterAnnotations(_) => {"RuntimeVisibleParameterAnnotations"},
        AttributeType::RuntimeInvisibleParameterAnnotations(_) => {"RuntimeInvisibleParameterAnnotations"},
        AttributeType::AnnotationDefault(_) => {"AnnotationDefault"},
        AttributeType::MethodParameters(_) => {"MethodParameters"},
        AttributeType::Synthetic(_) => {"Synthetic"},
        AttributeType::Deprecated(_) => {"Deprecated"},
        AttributeType::Signature(_) => {"Signature"},
        AttributeType::RuntimeVisibleAnnotations(_) => {"RuntimeVisibleAnnotations"},
        AttributeType::RuntimeInvisibleAnnotations(_) => {"RuntimeInvisibleAnnotations"},
        AttributeType::LineNumberTable(_) => {"LineNumberTable"},
        AttributeType::LocalVariableTable(_) => {"LocalVariableTable"},
        AttributeType::LocalVariableTypeTable(_) => {"LocalVariableTypeTable"},
        AttributeType::StackMapTable(_) => {"StackMapTable"},
        AttributeType::RuntimeVisibleTypeAnnotations(_) => {"RuntimeVisibleTypeAnnotations"},
        AttributeType::RuntimeInvisibleTypeAnnotations(_) => {"RuntimeInvisibleTypeAnnotations"},
    }.to_string()
}

fn write_class_attributes(context: &PrologGenContext, w: &mut dyn Write)-> Result<(),io::Error> {
    for class_file in context.to_load.iter() {
        write!(w, "classAttributes({}, [", class_prolog_name(&class_name(&class_file)))?;
        for (i,attribute) in class_file.attributes.iter().enumerate() {
            write_attribute(&attribute, w)?;
            if class_file.attributes.len() - 1 != i {
                write!(w, ",")?;
            }
        }
        write!(w, "]).\n")?;
    }
    Ok(())
}

// Extracts a list, Attributes , of the attributes of the class Class .
// Each attribute is represented as a functor application of the form
// attribute(AttributeName, AttributeContents) , where AttributeName
// is the name of the attribute. The format of the attribute's contents is unspecified.

//classDefiningLoader(Class, Loader)

//void writeClassDefiningLoader(struct ClassFile classFile, FILE *out) {
//fprintf(out, "classDefiningLoader( __");
//    write_class_name((struct PrologGenContext) classFile, out);//todo
//fprintf(out, ", bootStrapLoader).\n");
//}

// Extracts the defining class loader, Loader , of the class Class .

//void writeBootsrapLoader(struct ClassFile classFile, FILE *out) {
//fprintf(out, "isBootstrapLoader(bootStrapLoader).\n");
//}

//isBootstrapLoader(Loader)


// True iff the class loader Loader is the bootstrap class loader.
//loadedClass(Name, InitiatingLoader, ClassDefinition)

//void writeLoadedClass();


// True iff there exists a class named Name whose representation (in accordance
// with this specification) when loaded by the class loader InitiatingLoader is
// ClassDefinition .


//methodName(Method, Name)
// Extracts the name, Name , of the method Method .

fn write_method_name( context: &PrologGenContext, w: &mut dyn Write)-> Result<(),io::Error> {
    for class_file in context.to_load.iter() {
        for method in class_file.methods.iter(){
            write!(w,"methodName(")?;
            write_method_prolog_name(class_file,&method,w)?;
            write!(w, ",'{}').\n",extract_string_from_utf8( &class_file.constant_pool[method.name_index as usize]))?;
        }
    }
    Ok(())
}


//methodAccessFlags(Method, AccessFlags
//)
// Extracts the access flags, AccessFlags , of the method Method .

fn before_method_access_flags(class_file: &Classfile,method_info: &MethodInfo, w: &mut dyn Write)-> Result<(),io::Error>{
    write!(w,"methodAccessFlags(")?;
    write_method_prolog_name(class_file, method_info,w)?;
    Ok(())
}

fn write_method_access_flags(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error>{
    for class_file in context.to_load.iter() {
        for method_info in class_file.methods.iter() {
            if (method_info.access_flags & ACC_PUBLIC) > 0 {
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", public).\n")?;
            }
            if (method_info.access_flags & ACC_PRIVATE) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", private).\n")?;
            }
            if (method_info.access_flags & ACC_PROTECTED) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", protected).\n")?;
            }
            if (method_info.access_flags & ACC_STATIC) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", static).\n")?;
            }
            if (method_info.access_flags & ACC_FINAL) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", final).\n")?;
            }
            if (method_info.access_flags & ACC_SUPER) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", super).\n")?;
            }
            if (method_info.access_flags & ACC_BRIDGE) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", bridge).\n")?;
            }
            if (method_info.access_flags & ACC_VOLATILE) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", volatile).\n")?;
            }
            if (method_info.access_flags & ACC_TRANSIENT) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", transient).\n")?;
            }
            if (method_info.access_flags & ACC_NATIVE) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", native).\n")?;
            }
            if (method_info.access_flags & ACC_INTERFACE) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", interface).\n")?;
            }
            if (method_info.access_flags & ACC_ABSTRACT) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", abstract).\n")?;
            }
            if (method_info.access_flags & ACC_STRICT) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", strict).\n")?;
            }
            if (method_info.access_flags & ACC_SYNTHETIC) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", synthetic).\n")?;
            }
            if (method_info.access_flags & ACC_ANNOTATION) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", annotation).\n")?;
            }
            if (method_info.access_flags & ACC_ENUM) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", enum).\n")?;
            }
            if (method_info.access_flags & ACC_MODULE) > 0{
                before_method_access_flags(class_file,method_info, w)?;
                write!(w, ", module).\n")?;
            }
        }
    }
    Ok(())
}

//methodDescriptor(Method, Descriptor
//)
// Extracts the descriptor, Descriptor , of the method Method .

pub fn prolog_method_descriptor(class_file: &Classfile, method_info: & MethodInfo, w: &mut dyn Write) -> Result<(),io::Error>{
    write!(w,"descriptor__")?;
    write_method_prolog_name(class_file,method_info,w)?;
    Ok(())
}

pub fn write_method_descriptor(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error>{
    for class_file in context.to_load.iter(){
        for method_info in class_file.methods.iter(){
            let method_name = class_prolog_name(&class_name(class_file));
            write!(w, "methodDescriptor({},", method_name)?;
            prolog_method_descriptor(class_file,method_info,w)?;
            write!(w,").\n")?;
        }
    }
    Ok(())
}

//methodAttributes(Method, Attributes
//)
// Extracts a list, Attributes , of the attributes of the method Method .
//todo method attributes

//isInit(Method)
// True iff Method (regardless of class) is <init> .
//isNotInit(Method)
// True iff Method (regardless of class) is not <init> .
fn write_is_init_and_is_not_init(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error> {
    for class_file in context.to_load.iter() {
        for method_info in class_file.methods.iter(){
            let method_name_info = &class_file.constant_pool[method_info.name_index as usize];
            let method_name = extract_string_from_utf8(method_name_info);
            if method_name == "<init>".to_string() {
                write!(w, "isInit(")?;
            } else {
                write!(w, "isNotInit(")?;
            }
            write_method_prolog_name(&class_file, &method_info, w)?;
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
        write_method_prolog_name($class_file, $method_info, $w)?;
        write!($w, ",{}).\n", class_prolog_name(&class_name($class_file)))?;
    };
}

fn write_is_and_is_not_attributes_method(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_load.iter() {
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

pub mod types;

pub mod instruction_parser;

//isProtected(MemberClass, MemberName, MemberDescriptor)
// True iff there is a member named MemberName with descriptor
// MemberDescriptor in the class MemberClass and it is protected .
//isNotProtected(MemberClass, MemberName, MemberDescriptor)

pub fn prolog_field_name(class_file: &Classfile, field_info: &FieldInfo ,w: &mut dyn Write) -> Result<(), io::Error> {
    let field_name = extract_string_from_utf8(&class_file.constant_pool[field_info.name_index as usize]);
    write!(w, "prolog_name__{}__Field_{}", class_prolog_name(&class_name(class_file)), field_name)?;
    Ok(())
}

pub fn prolog_field_descriptor(class_file: &Classfile, field_info: &FieldInfo ,w: &mut dyn Write) -> Result<(), io::Error> {
    let field_name = extract_string_from_utf8(&class_file.constant_pool[field_info.name_index as usize]);
    write!(w, "prolog_descriptor__{}__Field_{}", class_prolog_name(&class_name(class_file)), field_name)?;
    Ok(())
}


// True iff there is a member named MemberName with descriptor
// MemberDescriptor in the class MemberClass and it is not protected .

pub fn write_is_and_is_not_protected(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_load.iter(){
        for field_info in class_file.fields.iter(){
            if(field_info.access_flags & ACC_PROTECTED) > 0 {
                write!(w,"isProtected({},",class_name(class_file))?;
            }else {
                write!(w,"isNotProtected({},",class_name(class_file))?;
            }
            prolog_field_name(class_file,field_info,w)?;
            write!(w,",")?;
            prolog_field_descriptor(class_file,field_info,w)?;
            write!(w,").\n")?;
        }
    }
    Ok(())
}

//parseFieldDescriptor(Descriptor, Type
//)
// Converts a field descriptor, Descriptor , into the corresponding verification
// type Type (§4.10.1.2).

pub fn write_parse_field_descriptor(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_load.iter(){
        for field_info in class_file.fields.iter(){
            write!(w,"parseFieldDescriptor(")?;
            prolog_field_descriptor(class_file,field_info,w)?;
            let descriptor_string = extract_string_from_utf8(&class_file.constant_pool[field_info.descriptor_index as usize]);
            let parsed_type = parse_field_descriptor(descriptor_string.as_str()).expect("Error parsing field descriptor");
            write!(w,",")?;
            write_type_prolog(context, &parsed_type.field_type,w)?;
            write!(w,").\n")?;
        }
    }
    Ok(())
}

//descriptor name, list of types and return type
//parseMethodDescriptor(Descriptor, ArgTypeList, ReturnType)
// Converts a method descriptor, Descriptor , into a list of verification types,
// ArgTypeList , corresponding to the method argument types, and a verification
// type, ReturnType , corresponding to the return type.


pub fn write_parse_method_descriptor(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error>{
    for class_file in context.to_load.iter(){
        for method_info in class_file.methods.iter() {
            write!(w,"parseMethodDescriptor(")?;
            prolog_method_descriptor(class_file, method_info, w)?;
            let method_descriptor_str = extract_string_from_utf8(&class_file.constant_pool[method_info.descriptor_index as usize]);
            let method_descriptor = parse_method_descriptor(method_descriptor_str.as_str()).expect("Error parsing method descriptor");
            write!(w,",[")?;
            for (i, parameter_type) in method_descriptor.parameter_types.iter().enumerate() {
                write_type_prolog(context, &parameter_type, w)?;
                if i != method_descriptor.parameter_types.len() - 1 {
                    write!(w, ",")?;
                }
            }
            write!(w,"],")?;
            write_type_prolog(context,&method_descriptor.return_type,w)?;
            write!(w, ").\n")?;
        }
    }
    Ok(())
}

pub mod code_verification;

