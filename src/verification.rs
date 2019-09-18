use std::borrow::Borrow;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::io::Lines;
use std::process::{ChildStdout, Stdio};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use log::{info, trace, warn};
use regex::Regex;
use tempfile::NamedTempFile;

use class_loading::JVMClassesState;
use classfile::{ACC_ABSTRACT, ACC_ANNOTATION, ACC_BRIDGE, ACC_ENUM, ACC_FINAL, ACC_INTERFACE, ACC_MODULE, ACC_NATIVE, ACC_PRIVATE, ACC_PROTECTED, ACC_PUBLIC, ACC_STATIC, ACC_STRICT, ACC_SUPER, ACC_SYNTHETIC, ACC_TRANSIENT, ACC_VOLATILE, AttributeInfo, Classfile, code_attribute, FieldInfo, MethodInfo, parse_class_file};
use classfile::attribute_infos::AttributeType;
use classfile::constant_infos::{ConstantInfo, ConstantKind};
use classfile::parsing_util::ParsingContext;
use verification::code_verification::write_parse_code_attribute;
use verification::PrologOutput::{NeedsAnotherClass, True};
use verification::types::{parse_field_descriptor, parse_method_descriptor, write_type_prolog};

use self::prolog_initial_defs::prolog_initial_defs;

#[derive(Debug)]
pub struct NeedsToLoadAnotherClass {
    pub another_class: Box<String>
}

pub fn verify(state: &JVMClassesState) -> Option<String> {
    let mut prolog = Command::new("/usr/bin/prolog")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn prolog");
    let mut prolog_output = BufReader::new(prolog.stdout.take().expect("error reading prolog output"));
    let mut prolog_input = prolog.stdin.take().expect("error getting prolog input stream");
    let mut output_lines = prolog_output.lines();

    let mut context = init_prolog_context(&state);
    let mut generated_prolog_defs_file = NamedTempFile::new().expect("Error creating tempfile");
    trace!("tempfile for prolog defs created at: {}", generated_prolog_defs_file.path().as_os_str().to_str().expect("Could not convert path to str"));

    prolog_initial_defs(&mut prolog_input).unwrap();
    let initial_defs_written = read_true_false_another_class(&mut output_lines);
    match initial_defs_written {
        True => {},
        PrologOutput::False => { panic!() },
        NeedsAnotherClass(_) => { panic!() },
    }

    gen_prolog(&mut context, &mut generated_prolog_defs_file.as_file()).unwrap();
    write!(&mut prolog_input, "['{}'].\n", generated_prolog_defs_file.path().as_os_str().to_os_string().to_str().expect("Could not convert path to string")).unwrap();

    prolog_input.flush().unwrap();
    let generated_defs_res = read_true_false_another_class(&mut output_lines);
    match generated_defs_res {
        True => {},
        PrologOutput::False => { panic!() },
        NeedsAnotherClass(_) => { panic!() },
    }

    write!(&mut prolog_input, "classIsTypeSafe(class('java/lang/Object', bl)).\n").unwrap();
    prolog_input.flush().unwrap();

    let loading_attempt_res = read_true_false_another_class(&mut output_lines);
    prolog.kill().expect("Unable to kill prolog");
    match loading_attempt_res {
        True => {
            return None;
        },
        PrologOutput::False => { panic!() },
        NeedsAnotherClass(s) => {
            return Some(s);
        },
    }
}

enum PrologOutput {
    True,
    False,
    NeedsAnotherClass(String),
}

fn read_true_false_another_class(lines: &mut Lines<BufReader<ChildStdout>>) -> PrologOutput {
    let need_to_load_regex = Regex::new("Need to load:('([A-Za-z/]+)'|([A-Za-z/]+))").expect("Error parsing regex.");
    loop {
        let cur = lines.next();
        dbg!(&cur);
        let r = cur.unwrap();
        let s = r.unwrap();
        if s.contains("true.") {
            assert!(!s.contains("false."));
            return PrologOutput::True;
        } else if s.contains("false.") {
            assert!(!s.contains("true."));
            dbg!("false");
            return PrologOutput::False;
        } else if need_to_load_regex.is_match(s.as_str()) {//todo pattern needs string const
            let captures = need_to_load_regex.captures(s.as_str()).unwrap();
            dbg!(&captures);
            let class_name = captures.get(3).unwrap().as_str().to_string();
            dbg!("got class name");
            return PrologOutput::NeedsAnotherClass(class_name);
        }
    }
}

fn init_prolog_context<'s>(state: &'s JVMClassesState) -> PrologGenContext<'s> {
    let mut to_verify = Vec::new();
    for class_entry in &state.loading_in_progress {
        let path = state.indexed_classpath.get(class_entry).unwrap();
        let mut p = ParsingContext { f: File::open(path).expect("This is a bug") };
        let class_file = parse_class_file(&mut p);
        to_verify.push(class_file)
    }
    let mut context: PrologGenContext<'s> = PrologGenContext { state, to_verify, extra: ExtraDescriptors { extra_method_descriptors: Vec::new(), extra_field_descriptors: Vec::new() } };
    (context)
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

pub struct ExtraDescriptors {
    pub extra_method_descriptors: Vec<String>,
    pub extra_field_descriptors : Vec<String>
}

pub struct PrologGenContext<'l> {
    pub state: &'l JVMClassesState,
    pub to_verify: Vec<Classfile>,
    pub extra: ExtraDescriptors
}

pub fn gen_prolog(context: &mut PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
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
    write_parse_field_descriptors(context, w)?;
    write_method_descriptor(context,w)?;
    write_parse_method_descriptor(context,w)?;
    write_parse_code_attribute(context,w)?;
    write_method_attributes(context,w)?;
    write_extra_descriptors(context,w)?;
    w.flush()?;
    Ok(())
}

const BOOTSTRAP_LOADER_NAME: &str = "bl";

pub fn write_extra_descriptors(context: &PrologGenContext,w: &mut dyn Write) ->  Result<(), io::Error>{
    let extra_descriptors = &context.extra;
    for field_descriptor in extra_descriptors.extra_field_descriptors.iter(){
        //todo dup
        write!(w,"parseFieldDescriptor('{}',",field_descriptor)?;
        let parsed_type = parse_field_descriptor(field_descriptor.as_str()).expect("Error parsing field descriptor");
        write_type_prolog(context, &parsed_type.field_type,w)?;
        write!(w,").\n")?;
    }
    for method_descriptor in extra_descriptors.extra_method_descriptors.iter(){
        //todo dup
        write!(w,"parseMethodDescriptor('{}'",method_descriptor)?;
        let method_descriptor = parse_method_descriptor(method_descriptor.as_str()).expect("Error parsing method descriptor");
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
    Ok(())
}

pub fn write_loaded_class(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    if context.state.using_bootstrap_loader {
        let to_load_classes = context.to_verify.iter();
        let already_loaded_classes = context.state.bootstrap_loaded_classes.values();
        for class_file in to_load_classes {
            write!(w,"loadedClass('{}',{}, {}).\n", class_name(class_file), BOOTSTRAP_LOADER_NAME , class_prolog_name(&class_name(class_file)))?;
        }
        for class_file in already_loaded_classes {
            write!(w,"loadedClass('{}',{}, ClassDefinition).\n", class_name(class_file), BOOTSTRAP_LOADER_NAME )?;//todo duplication
        }
        write!(w, "loadedClass(ClassName,_,_) :- write('Need to load:'),writeln(ClassName),fail.\n")?;
    } else {
        unimplemented!()
    }
    Ok(())
}

pub fn write_is_bootstrap_loader(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    if context.state.using_bootstrap_loader {
        write!(w, "isBootstrapLoader({}).\n", BOOTSTRAP_LOADER_NAME)?;
    }
    else{
        unimplemented!()
    }
    Ok(())
}


pub fn write_class_defining_loader(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        if context.state.using_bootstrap_loader {
            write!(w, "classDefiningLoader({},{}).\n", class_prolog_name(&class_name(class_file)), BOOTSTRAP_LOADER_NAME)?;
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
//    let mut base = "n__".to_string();
//    base.push_str(class_.replace("/","__").as_str());
    //todo , if using bootstrap loader and the like
    return format!("class('{}', {})",class_,BOOTSTRAP_LOADER_NAME)
}

// Extracts the name, ClassName , of the class Class .
//classClassName(Class, ClassName)
//todo function for class object name
fn write_class_name(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error> {
    for class_file in context.to_verify.iter() {
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
    for class_file in context.to_verify.iter() {
        if is_interface(class_file.borrow()) {
            write!(w, "classIsInterface({}).\n", class_prolog_name(&class_name(&class_file)))?;
        }
    }
    Ok(())
}

//classIsNotFinal(Class)
// True iff the class, Class , is not a final class.

fn write_class_is_not_final(context: &PrologGenContext, w: &mut dyn Write)-> Result<(),io::Error> {
    for class_file in context.to_verify.iter() {
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
    for class_file in context.to_verify.iter() {
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
    for class_file in context.to_verify.iter() {
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
    write!(w, "method({},'{}',", class_prolog_name(&class_name(class_file)), method_name(class_file, method_info))?;
    prolog_method_descriptor(class_file,method_info,w)?;
    write!(w,")")?;
    Ok(())
}

//classMethods(Class, Methods)
// Extracts a list, Methods , of the methods declared in the class Class .
fn write_class_methods(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error>{
    for class_file in context.to_verify.iter() {
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
    for class_file in context.to_verify.iter() {
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
    for class_file in context.to_verify.iter() {
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
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter() {
            before_method_access_flags(class_file, method_info, w)?;
            write!(w, ",[ignore_this")?;
            if (method_info.access_flags & ACC_PUBLIC) > 0 {
                write!(w, ", public")?;
            }
            if (method_info.access_flags & ACC_PRIVATE) > 0{
                write!(w, ", private")?;
            }
            if (method_info.access_flags & ACC_PROTECTED) > 0{
                write!(w, ", protected")?;
            }
            if (method_info.access_flags & ACC_STATIC) > 0{
                write!(w, ", static")?;
            }
            if (method_info.access_flags & ACC_FINAL) > 0{
                write!(w, ", final")?;
            }
            if (method_info.access_flags & ACC_SUPER) > 0{
                write!(w, ", super")?;
            }
            if (method_info.access_flags & ACC_BRIDGE) > 0{
                write!(w, ", bridge")?;
            }
            if (method_info.access_flags & ACC_VOLATILE) > 0{
                write!(w, ", volatile")?;//todo wrong
            }
            if (method_info.access_flags & ACC_TRANSIENT) > 0{
                write!(w, ", transient")?;
            }
            if (method_info.access_flags & ACC_NATIVE) > 0{
                write!(w, ", native")?;
            }
            if (method_info.access_flags & ACC_INTERFACE) > 0{
                write!(w, ", interface")?;
            }
            if (method_info.access_flags & ACC_ABSTRACT) > 0{
                write!(w, ", abstract")?;
            }
            if (method_info.access_flags & ACC_STRICT) > 0{
                write!(w, ", strict")?;
            }
            if (method_info.access_flags & ACC_SYNTHETIC) > 0{
                write!(w, ", synthetic")?;
            }
            if (method_info.access_flags & ACC_ANNOTATION) > 0{
                write!(w, ", annotation")?;
            }
            if (method_info.access_flags & ACC_ENUM) > 0{
                write!(w, ", enum")?;
            }
            if (method_info.access_flags & ACC_MODULE) > 0{
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

pub fn prolog_method_descriptor(class_file: &Classfile, method_info: & MethodInfo, w: &mut dyn Write) -> Result<(),io::Error>{
    let descriptor= extract_string_from_utf8(&class_file.constant_pool[method_info.descriptor_index as usize]);
    write!(w,"'{}'",descriptor)?;
    Ok(())
}

fn method_name(class_file: &Classfile, method_info: &MethodInfo) -> String{
    let method_name_utf8 = &class_file.constant_pool[method_info.name_index as usize];
    let method_name = extract_string_from_utf8(method_name_utf8);
    method_name
}

fn method_prolog_name(class_file: &Classfile, method_info: &MethodInfo) -> String {
    method_name(class_file,method_info).replace("<clinit>", "__clinit").replace("<init>","__init")
}

pub fn write_method_descriptor(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error>{
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter(){
            write!(w, "methodDescriptor(")?;
            write_method_prolog_name(class_file,method_info,w)?;
            write!(w,",")?;
            prolog_method_descriptor(class_file,method_info,w)?;
            write!(w,").\n")?;
        }
    }
    Ok(())
}

//methodAttributes(Method, Attributes
//)
// Extracts a list, Attributes , of the attributes of the method Method .
fn write_method_attributes(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error>{
    for class_file in context.to_verify.iter(){
        for method_info in class_file.methods.iter(){
            match code_attribute(method_info) {
                None => {},
                Some(c) => {
                    write!(w, "methodAttributes(")?;
                    write_method_prolog_name(class_file,method_info,w)?;
                    write!(w,", [attribute('Code','')]).\n",)?;
                },
            }

//            for attribute in method_info.attributes.iter(){
//                match &attribute.attribute_type{
//                    AttributeType::Code(c) => {
//                        write!(w,"methodAttributes(")?;
//                        write_method_prolog_name(class_file,method_info,w)?;
//                        write!(w,", '').\n",)?;
//                    },
//                    _ => {/* only attribute that matters is code*/}
//                }
        }
    }
    Ok(())
}

//isInit(Method)
// True iff Method (regardless of class) is <init> .
//isNotInit(Method)
// True iff Method (regardless of class) is not <init> .
fn write_is_init_and_is_not_init(context: &PrologGenContext, w: &mut dyn Write) -> Result<(),io::Error> {
    for class_file in context.to_verify.iter() {
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

pub mod types;

pub mod instruction_parser;

//isProtected(MemberClass, MemberName, MemberDescriptor)
// True iff there is a member named MemberName with descriptor
// MemberDescriptor in the class MemberClass and it is protected .
//isNotProtected(MemberClass, MemberName, MemberDescriptor)

pub fn prolog_field_name(class_file: &Classfile, field_info: &FieldInfo ,w: &mut dyn Write) -> Result<(), io::Error> {
    let field_name = extract_string_from_utf8(&class_file.constant_pool[field_info.name_index as usize]);
    write!(w, "field({},'{}',", class_prolog_name(&class_name(class_file)), field_name)?;
    prolog_field_descriptor(class_file, field_info, w)?;
    write!(w, ")")?;
    Ok(())
}

pub fn prolog_field_descriptor(class_file: &Classfile, field_info: &FieldInfo ,w: &mut dyn Write) -> Result<(), io::Error> {
    let descriptor = extract_string_from_utf8(&class_file.constant_pool[field_info.descriptor_index as usize]);
    write!(w, "'{}'", descriptor)?;
    Ok(())
}


// True iff there is a member named MemberName with descriptor
// MemberDescriptor in the class MemberClass and it is not protected .

pub fn write_is_and_is_not_protected(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
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

pub fn write_parse_field_descriptors(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
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
    for class_file in context.to_verify.iter() {
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

