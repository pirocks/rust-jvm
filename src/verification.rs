use std::borrow::Borrow;
use std::fmt;
use std::fmt::Pointer;

use bimap::BiMap;

use classfile::{ACC_FINAL, ACC_INTERFACE, AttributeInfo, Classfile, MethodInfo};
use classfile::constant_infos::{ConstantInfo, ConstantKind};
use interpreter::InstructionType::ret;
use classfile::attribute_infos::AttributeType;

pub struct PrologGenContext {
    pub class_files: Vec<Classfile>,
    pub name_to_classfile: BiMap<String, Classfile>,//todo need to init
}


const CLASS_IS_TYPE_SAFE: &str = "
classIsTypeSafe(Class) :-\
    classClassName(Class, Name) \
    classDefiningLoader(Class, L),\
    superclassChain(Name, L, Chain),\
    Chain \\= [],\
    classSuperClassName(Class, SuperclassName),\
    loadedClass(SuperclassName, L, Superclass),\
    classIsNotFinal(Superclass),\
    classMethods(Class, Methods),\
    checklist(methodIsTypeSafe(Class), Methods).\
\
classIsTypeSafe(Class) :-\
    classClassName(Class, 'java/lang/Object'),\
    classDefiningLoader(Class, L),\
    isBootstrapLoader(L),\
    classMethods(Class, Methods),\
    checklist(methodIsTypeSafe(Class), Methods).\
";

fn class_name(class: &Classfile) -> String {
    match &(class.constant_pool[class.this_class as usize]).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        },
        _ => { panic!() }
    }
}

fn class_prolog_name(class_: &String) -> String {
    let mut base = "prolog_name__".to_string();
    base.push_str(class_);
    return base
}

// Extracts the name, ClassName , of the class Class .
//classClassName(Class, ClassName)
//todo function for class object name
fn write_class_name(context: PrologGenContext, f: &mut fmt::Formatter) -> () {
    for class_file in context.class_files {
        let class_name = class_name(&class_file);
        write!(f, "classClassName({},{}).\n", class_prolog_name(&class_name), class_name);
    }
}

fn is_interface(class: &Classfile) -> bool {
    return (class.access_flags & ACC_INTERFACE) > 0
}


fn is_final(class: &Classfile) -> bool {
    return (class.access_flags & ACC_FINAL) > 0
}

//classIsInterface(Class)
// True iff the class, Class , is an interface.
fn write_is_interface(context: PrologGenContext, f: &mut fmt::Formatter) -> () {
    for class_file in context.class_files {
        if is_interface(class_file.borrow()) {
            write!(f, "classIsInterface({}).\n", class_prolog_name(&class_name(&class_file)));
        }
    }
}

//classIsNotFinal(Class)
// True iff the class, Class , is not a final class.

fn write_class_is_not_final(context: PrologGenContext, f: &mut fmt::Formatter) {
    for class_file in context.class_files {
        if !is_final(&class_file) {
            write!(f, "classIsNotFinal({}).\n", class_prolog_name(&class_name(&class_file)));
        }
    }
}

//todo this should go at top
fn extract_string_from_utf8(utf8: &ConstantInfo) -> String {
    match &(utf8).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        },
        _ => { panic!() }
    }
}

fn get_super_class_name(class: &Classfile) -> String {
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

fn write_class_super_class_name(context: PrologGenContext, f: &mut fmt::Formatter) {
    //todo check if has super class
    for class_file in context.class_files {
        if has_super_class(&class_file) {
            let super_class_name = get_super_class_name(&class_file);
            let base_class = class_prolog_name(&class_name(&class_file));
            write!(f, "classSuperClassName({},{}).\n", base_class, super_class_name);
        }
    }
}

//classInterfaces(Class, Interfaces)
// Extracts a list, Interfaces , of the direct superinterfaces of the class Class .

fn write_class_interfaces(context: PrologGenContext, f: &mut fmt::Formatter) {
    for class_file in context.class_files {
        write!(f, "classInterfaces({},[", class_prolog_name(&class_name(&class_file)));
        for (i, interface) in class_file.interfaces.iter().enumerate() {
            let interface_name = extract_string_from_utf8(&class_file.constant_pool[*interface as usize]);
            let prolog_interface_name = class_prolog_name(&interface_name);
            let class_name = class_name(&class_file);
            if i == class_file.interfaces.len() - 1 {
                write!(f, "{}", prolog_interface_name);
            } else {
                write!(f, "{},", prolog_interface_name);
            }
        }
        write!(f, "]).\n");
    }
}


fn write_method_prolog_name(class_file: &Classfile, method_info: &MethodInfo, f: &mut fmt::Formatter) {
    let method_name_utf8 = &class_file.constant_pool[method_info.name_index as usize];
    let method_name = extract_string_from_utf8(method_name_utf8);
    write!(f, "prolog_name__{}__Method_{}", class_name(class_file), method_name);
}

//classMethods(Class, Methods)
// Extracts a list, Methods , of the methods declared in the class Class .
fn write_class_methods(context: PrologGenContext, f: &mut fmt::Formatter) {
    for class_file in context.class_files {
        write!(f, "classMethods({},[", class_prolog_name(&class_name(&class_file)));
        for (i, method_info) in class_file.methods.iter().enumerate() {
            write_method_prolog_name(&class_file, method_info, f);
            if class_file.methods.len() - 1 != i {
                write!(f, ",");
            }
        }
        write!(f, "]).\n");
    }
}

//classAttributes(Class, Attributes)

fn write_attribute(attribute_info: &AttributeInfo, f: &mut fmt::Formatter) {
    let name = get_attribute_name(attribute_info);
    write!(f, "{}", name);
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

fn write_class_attributes(context: PrologGenContext, class_file: &Classfile, f: &mut fmt::Formatter) {
    for class_info in context.class_files {
        write!(f, "classAttributes({}, [", class_prolog_name(&class_name(&class_info)));
        for (i,attribute) in class_info.attributes.iter().enumerate() {
            write_attribute(&class_file.attributes[i], f);
            if class_info.attributes.len() - 1 != i {
                write!(f, ",");
            }
        }
        write!(f, "]).\n");
    }
}

// Extracts a list, Attributes , of the attributes of the class Class .
// Each attribute is represented as a functor application of the form
// attribute(AttributeName, AttributeContents) , where AttributeName
// is the name of the attribute. The format of the attribute's contents is unspecified.

//classDefiningLoader(Class, Loader)

//void writeClassDefiningLoader(struct ClassFile classFile, FILE *out) {
//fprintf(out, "classDefiningLoader( __");
////    write_class_name((struct PrologGenContext) classFile, out);//todo
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

//void write_method_name(struct ClassFile classFile, FILE *out) {
//    for (size_t i = 0; i < classFile.methods_count; ++i) {
//        struct cp_info method_name_info = classFile.constant_pool[classFile.methods[i].name_index];
//        assert(method_name_info.tag = CONSTANT_Utf8);
//        //todo print method name
//        uint16_t method_name_length = method_name_info.constantUtf8Info.length;
//        uint8_t *method_name = method_name_info.constantUtf8Info.bytes;
//        fprintf(out, "methodName(__%.*s__Method_%.*s,'%.*s'", get_name_length(classFile), get_name(classFile), method_name_length, method_name, method_name_length, method_name);
//    }
//}

//methodName(Method, Name)
// Extracts the name, Name , of the method Method .

//void writeMethodAccessFlags(struct ClassFile classFile,struct method_info methodInfo, FILE *out) {
//    if (methodInfo.access_flags & ACC_PUBLIC) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", public).\n");
//    }
//    if (methodInfo.access_flags & ACC_PRIVATE) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", private).\n");
//    }
//    if (methodInfo.access_flags & ACC_PROTECTED) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", protected).\n");
//    }
//    if (methodInfo.access_flags & ACC_STATIC) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", static).\n");
//    }
//    if (methodInfo.access_flags & ACC_FINAL) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", final).\n");
//    }
//    if (methodInfo.access_flags & ACC_SUPER) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", super).\n");
//    }
//    if (methodInfo.access_flags & ACC_BRIDGE) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", bridge).\n");
//    }
//    if (methodInfo.access_flags & ACC_VOLATILE) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", volatile).\n");
//    }
//    if (methodInfo.access_flags & ACC_TRANSIENT) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", transient).\n");
//    }
//    if (methodInfo.access_flags & ACC_NATIVE) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", native).\n");
//    }
//    if (methodInfo.access_flags & ACC_INTERFACE) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", interface).\n");
//    }
//    if (methodInfo.access_flags & ACC_ABSTRACT) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", abstract).\n");
//    }
//    if (methodInfo.access_flags & ACC_STRICT) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", strict).\n");
//    }
//    if (methodInfo.access_flags & ACC_SYNTHETIC) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", synthetic).\n");
//    }
//    if (methodInfo.access_flags & ACC_ANNOTATION) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", annotation).\n");
//    }
//    if (methodInfo.access_flags & ACC_ENUM) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", enum).\n");
//    }
//    if (methodInfo.access_flags & ACC_MODULE) {
//        beforeMethodAccessFlags(methodInfo,out);
//        fprintf(out,", module).\n");
//    }
//}
//
//methodAccessFlags(Method, AccessFlags
//)
// Extracts the access flags, AccessFlags , of the method Method .
//methodDescriptor(Method, Descriptor
//)
// Extracts the descriptor, Descriptor , of the method Method .
//methodAttributes(Method, Attributes
//)
// Extracts a list, Attributes , of the attributes of the method Method .

//void writeIsInitAndIsNotInit(struct ClassFile classFile, FILE *out) {
//for (size_t i = 0; i < classFile.methods_count; + + i) {
//struct cp_info method_name_info = classFile.constant_pool[classFile.methods[i].name_index];
//assert(method_name_info.tag == CONSTANT_Utf8);
//uint16_t method_name_length = method_name_info.constantUtf8Info.length;
//uint8_t * method_name = method_name_info.constantUtf8Info.bytes;
//if (0 == strncmp(method_name, "<init>", method_name_length)) {
//fprintf(out, "isInit(");
//} else {
//fprintf(out, "isNotInit(");
//}
//write_method_name(classFile, classFile.methods[i], out);
//fprintf(out, ").");
//}
//
//}

//isInit(Method)
// True iff Method (regardless of class) is <init> .
//isNotInit(Method)
// True iff Method (regardless of class) is not <init> .
//isNotFinal(Method, Class)

//void writeIsNotFinalMethod(struct ClassFile classFile, struct method_info methodInfo, FILE *out) {
//if (methodInfo.access_flags & ACC_FINAL) {
//fprintf(out, "isFinal(");
//} else {
//fprintf(out, "isNotFinal(");
//}
//write_method_name(classFile, methodInfo, out);
//fprintf(out, ",");
////    write_class_name(0, out);//todo
//fprintf(out, ").");
//}

//void writeIsNotFinal(struct ClassFile classFile, FILE *out) {
//for (size_t i = 0; i < classFile.methods_count; + + i) {
//struct method_info methodInfo = classFile.methods[i];
//writeIsNotFinalMethod(classFile, methodInfo, out);
//}
//}

// True iff Method in class Class is not final .
//isStatic(Method, Class)

//void writeIsStaticMethod(struct ClassFile classFile, struct method_info methodInfo, FILE *out) {
//if (methodInfo.access_flags & ACC_STATIC) {
//fprintf(out, "isStatic(");
//} else {
//fprintf(out, "isNotStatic(");
//}
//write_method_name(classFile, methodInfo, out);
//fprintf(out, ",");
////    write_class_name(0, out);//todo
//fprintf(out, ").");
//}

//void writeIsStatic(struct ClassFile classFile, FILE *out) {
//for (size_t i = 0; i < classFile.methods_count; + + i) {
//struct method_info methodInfo = classFile.methods[i];
//writeIsStaticMethod(classFile, methodInfo, out);
//}
//}


// True iff Method in class Class is static .
//isNotStatic(Method, Class)
// True iff Method in class Class is not static .
//isPrivate(Method, Class)
// True iff Method in class Class is private .
//isNotPrivate(Method, Class)
// True iff Method in class Class is not private .

//void writeIsPrivateMethod(struct ClassFile classFile, struct method_info methodInfo, FILE *out) {
//if (methodInfo.access_flags & ACC_PRIVATE) {
//fprintf(out, "isPrivate(");
//} else {
//fprintf(out, "isNotPrivate(");
//}
//write_method_name(classFile, methodInfo, out);
//fprintf(out, ",");
////    write_class_name(0, out);//todo
//fprintf(out, ").");
//}

//void writeIsPrivate(struct ClassFile classFile, FILE *out) {
//for (size_t i = 0; i < classFile.methods_count; + + i) {
//struct method_info methodInfo = classFile.methods[i];
//writeIsPrivateMethod(classFile, methodInfo, out);
//}
//}


//isProtected(MemberClass, MemberName, MemberDescriptor)
// True iff there is a member named MemberName with descriptor
// MemberDescriptor in the class MemberClass and it is protected .
//isNotProtected(MemberClass, MemberName, MemberDescriptor)


//void writeIsProtectedMethod(struct ClassFile classFile, struct method_info methodInfo, FILE *out) {
//if (methodInfo.access_flags & ACC_PROTECTED) {
//fprintf(out, "isProtected(");
//} else {
//fprintf(out, "isNotProtected(");
//}
//write_method_name(classFile, methodInfo, out);
//fprintf(out, ",");
////    write_class_name(0, out);//todo
//fprintf(out, ").");
//}

//void writeIsProtected(struct ClassFile classFile, FILE *out) {
//for (size_t i = 0; i < classFile.methods_count; + + i) {
//struct method_info methodInfo = classFile.methods[i];
//writeIsProtectedMethod(classFile, methodInfo, out);
//}
//}

//void print_type_byte(FILE * out){
//fprintf(out, "byte");
//}

//void print_type_char(FILE * out){
//fprintf(out, "char");
//}

//void print_type_short(FILE * out){
//fprintf(out, "short");
//}

//void print_type_int(FILE * out){
//fprintf(out, "int");
//}

//void print_type_long(FILE * out){
//fprintf(out, "long");
//}

//void print_type_float(FILE * out){
//fprintf(out, "float");
//}

//void print_type_double(FILE * out){
//fprintf(out, "double");
//}

//void print_type_boolean(FILE * out){
//fprintf(out, "boolean");
//}

//void print_type_void(FILE * out){
//fprintf(out, "boolean");
//}


//void writeType(char * * type_str, FILE* out ){
//char cur = * * type_str;
//switch(cur){
//case 'D':
//print_type_double(out);
//break;
//case 'F':
//print_type_float(out);
//break;
//case 'I':
//print_type_int(out);
//break;
//case 'J':
//print_type_long(out);
//break;
//case 'B':
//print_type_byte(out);
//break;
//case 'C':
//print_type_char(out);
//break;
//case 'S':
//print_type_short(out);
//break;
//case 'Z':
//print_type_boolean(out);
//break;
//case 'L':
//{
//( * type_str) + +;
////todo handle class name
//assert(false);
//}
//break;
//case '[':
//( * type_str) ++;
//fprintf(out, "arrayOf(");
//writeType(type_str, out);
//fprintf(out,")");
//break;
//case 'V':
//print_type_void(out);
//break;
//default:
//assert(false);
//break;
//}
//}


// True iff there is a member named MemberName with descriptor
// MemberDescriptor in the class MemberClass and it is not protected .
//parseFieldDescriptor(Descriptor, Type
//)

//void writeMethodDescriptor(struct ClassFile classFile, struct field_info fieldInfo){
//classFile.constant_pool[fieldInfo.descriptor_index].constantUtf8Info
//}

// Converts a field descriptor, Descriptor , into the corresponding verification
// type Type (§4.10.1.2).
// 1954.10
// Verification of class Files
// THE CLASS FILE FORMAT
//descriptor name, list of types and return type
//parseMethodDescriptor(Descriptor, ArgTypeList, ReturnType
//)

//void writeMethodDescriptor(struct ClassFile classFile, struct method_info methodInfo){
//classFile.constant_pool[methodInfo.descriptor_index].constantUtf8Info
//}

// Converts a method descriptor, Descriptor , into a list of verification types,
// ArgTypeList , corresponding to the method argument types, and a verification
// type, ReturnType , corresponding to the return type.
//parseCodeAttribute(Class, Method, FrameSize, MaxStack, ParsedCode, Handlers, StackMap
//)

//void writeFrame(){
////todo when stackmap becomes a thing
//}

//void writeParseCodeAttribute(struct ClassFile classFile, struct method_info methodInfo){
//assert(methodInfo.code_attribute != NULL);
//uint16_t max_stack = methodInfo.code_attribute -> max_stack;
//uint16_t frame_size = methodInfo.code_attribute -> max_locals;
////for now assume no stack map todo
////still need to handle parsed code and handlers
////handler(Start, End, Target, ClassName)
//for (size_t i = 0; i < methodInfo.attributes_count; + + i) {
//struct ExceptionTableElem handler = methodInfo.code_attribute -> exception_table[i];//...
//}
//    if(methodInfo.code_attribute->stack_map_table == NULL){
//        //then use empty stack map
//    }
//stackMap(Offset, TypeState)

//Offset is an integer indicating the bytecode offset at which the stack map frame
//applies (§4.7.4).
//The order of bytecode offsets in this list must be the same as in the class file.

//stackMap(Offset, frame(Locals, OperandStack, Flags))
//• Locals is a list of verification types, such that the i'th element of the list (with
//0-based indexing) represents the type of local variable i.
//Types of size 2 ( long and double ) are represented by two local variables
//(§2.6.1), with the first local variable being the type itself and the second local
//variable being top (§4.10.1.7).
//• OperandStack is a list of verification types, such that the first element of the list
//represents the type of the top of the operand stack, and the types of stack entries
//below the top follow in the list in the appropriate order.
//Types of size 2 ( long and double ) are represented by two stack entries, with the
//first entry being top and the second entry being the type itself.
//For example, a stack with a double value, an int value, and a long value is represented
//in a type state as a stack with five entries: top and double entries for the double
//value, an int entry for the int value, and top and long entries for the long value.
//Accordingly, OperandStack is the list [top, double, int, top, long] .
//• Flags is a list which may either be empty or have the single element
//flagThisUninit .
//If any local variable in Locals has the type uninitializedThis , then Flags has
//the single element flagThisUninit , otherwise Flags is an empty list.
//flagThisUninit is used in constructors to mark type states where initialization of this
//has not yet been completed. In such type states, it is illegal to return from the method.

//}


// Extracts the instruction stream, ParsedCode , of the method Method in Class ,
// as well as the maximum operand stack size, MaxStack , the maximal number
// of local variables, FrameSize , the exception handlers, Handlers , and the stack
// map StackMap .
// The representation of the instruction stream and stack map attribute must be as
// specified in §4.10.1.3 and §4.10.1.4.
//samePackageName(Class1, Class2
//)
// True iff the package names of Class1 and Class2 are the same.
//differentPack
// ageName(Class1, Class2
//)
//  True iff the package names of Class1 and Class2 are different.
