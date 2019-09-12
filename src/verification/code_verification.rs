//parseCodeAttribute(Class, Method, FrameSize, MaxStack, ParsedCode, Handlers, StackMap)

use std::io;
use std::io::Write;

use classfile::{Classfile, code_attribute, MethodInfo, stack_map_table_attribute};
use classfile::attribute_infos::{Code, ExceptionTableElem, ObjectVariableInfo, StackMapFrame, StackMapTable, VerificationTypeInfo, ArrayVariableInfo};
use verification::{class_name, class_prolog_name, extract_string_from_utf8, PrologGenContext, write_method_prolog_name};
use verification::types::{parse_method_descriptor, Type};

pub fn write_parse_code_attribute(context: &PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_load.iter() {
        for method_info in class_file.methods.iter() {
            write!(w, "parseCodeAttribute({},", class_prolog_name(&class_name(class_file)))?;
            write_method_prolog_name(class_file, method_info, w)?;

            let code = code_attribute(method_info);
            let max_stack = code.max_stack;
            let frame_size = code.max_locals;
            write!(w, ",{},{},", frame_size, max_stack)?;

            use verification::instruction_parser::output_instruction_info_for_code;
            output_instruction_info_for_code(code, w)?;


            write!(w, "[")?;
            for (i, exception_entry) in code.exception_table.iter().enumerate() {
                write_exception_handler(class_file, exception_entry, w)?;
                if i != code.exception_table.len() - 1 {
                    write!(w, ",")?;
                }
            }
            write!(w, "],")?;

            write_stack_map_frames(class_file, method_info, w)?;
            write!(w, ")")?;
        }
    }
    Ok(())
}

fn write_exception_handler(class_file: &Classfile, exception_handler: &ExceptionTableElem, w: &mut dyn Write) -> Result<(), io::Error> {
    let class_name = extract_string_from_utf8(&class_file.constant_pool[exception_handler.catch_type as usize]);
    write!(w, "handler({},{},{},{})", exception_handler.start_pc, exception_handler.end_pc, exception_handler.handler_pc, class_name)?;
    Ok(())
}

fn to_verification_type_helper(parameter_types: &Type) -> VerificationTypeInfo {
    match parameter_types {
        Type::ByteType(_) => { VerificationTypeInfo::Integer }
        Type::CharType(_) => { VerificationTypeInfo::Integer }
        Type::DoubleType(_) => { VerificationTypeInfo::Double }
        Type::FloatType(_) => { VerificationTypeInfo::Float }
        Type::IntType(_) => { VerificationTypeInfo::Integer }
        Type::LongType(_) => { VerificationTypeInfo::Long }
        Type::ReferenceType(r) => {
            VerificationTypeInfo::Object(ObjectVariableInfo {
                cpool_index: None,
                class_name: r.class_name.clone()
            })
        }
        Type::ShortType(_) => { VerificationTypeInfo::Integer }
        Type::BooleanType(_) => { VerificationTypeInfo::Integer }
        Type::ArrayReferenceType(_) => { unimplemented!() }
        Type::VoidType(_) => { panic!() }
    }
}

fn to_verification_type_array(parameter_types: &Vec<Type>) -> Vec<VerificationTypeInfo> {
    let mut res = Vec::new();
    for parameter_type in parameter_types {
        match parameter_type {
            Type::ByteType(_) => {
                res.push(VerificationTypeInfo::Integer);
            }
            Type::CharType(_) => {
                res.push(VerificationTypeInfo::Integer);
            }
            Type::DoubleType(_) => {
                res.push(VerificationTypeInfo::Top);
                res.push(VerificationTypeInfo::Double)
            }
            Type::FloatType(_) => { res.push(VerificationTypeInfo::Float) }
            Type::IntType(_) => { res.push(VerificationTypeInfo::Integer) }
            Type::LongType(_) => {
                res.push(VerificationTypeInfo::Top);
                res.push(VerificationTypeInfo::Long)
            }
            Type::ReferenceType(r) => {
                res.push(VerificationTypeInfo::Object(ObjectVariableInfo { cpool_index: None, class_name:r.class_name.clone() }))
            }
            Type::ShortType(_) => {
                res.push(VerificationTypeInfo::Integer);
            }
            Type::BooleanType(_) => {
                res.push(VerificationTypeInfo::Integer);
            }
            Type::ArrayReferenceType(art) => {
                    let sub_type = &art.sub_type;
                    res.push(VerificationTypeInfo::Array(ArrayVariableInfo { sub_type:Box::new(to_verification_type_helper(sub_type)) }))
            }
            Type::VoidType(_) => { panic!() }
        }
    }
    return res;
}

fn write_locals(locals: &Vec<VerificationTypeInfo>, w: &mut dyn Write) -> Result<(), io::Error> {
    write!(w, "[")?;
    for (i, local) in locals.iter().enumerate() {
        let verification_type_as_string = verification_type_as_string(local);
        write!(w, "{}", verification_type_as_string)?;
        if i != locals.len() - 1 {
            write!(w, ",")?;
        }
    }
    write!(w, "]")?;
    Ok(())
}

//todo this should really be a write function
fn verification_type_as_string(verification_type: &VerificationTypeInfo) -> String {
    match verification_type {
        VerificationTypeInfo::Top => { "top".to_string() }
        VerificationTypeInfo::Integer => { "integer".to_string() }
        VerificationTypeInfo::Float => { "float".to_string() }
        VerificationTypeInfo::Long => { "long".to_string() }
        VerificationTypeInfo::Double => { "double".to_string() }
        VerificationTypeInfo::Null => { "null".to_string() }
        VerificationTypeInfo::UninitializedThis => { unimplemented!() }
        VerificationTypeInfo::Object(_) => { unimplemented!() }
        VerificationTypeInfo::Uninitialized(_) => { unimplemented!() }
        VerificationTypeInfo::Array(a) => {
            let sub_str = verification_type_as_string(&a.sub_type);
            format!("arrayOf({})", sub_str)
        }
    }
}

fn write_operand_stack(operand_stack: &Vec<VerificationTypeInfo>, w: &mut dyn Write) -> Result<(), io::Error> {
    write_locals(operand_stack, w)
}

fn write_stack_map_frames(class_file: &Classfile, method_info: &MethodInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    let code: &Code = code_attribute(method_info);
    let stack_map: &StackMapTable = stack_map_table_attribute(method_info).expect("No stack msp table, this is a bug");

    let mut operand_stack = Vec::new();
    let mut locals = Vec::new();
    for _ in 0..code.max_locals {
        //top is the desired default type, as stated on 217 of the jvms12
        locals.push(VerificationTypeInfo::Top)
    }
    //todo dup
    let str_ = extract_string_from_utf8(&class_file.constant_pool[method_info.descriptor_index as usize]);
    let parsed_descriptor = parse_method_descriptor(str_.as_str()).expect("Error parsing method descriptor");
    dbg!(&parsed_descriptor.parameter_types);
    locals = to_verification_type_array(&parsed_descriptor.parameter_types);

    let mut current_offset = 0;
    write!(w,"[");
    for (i, entry) in stack_map.entries.iter().enumerate() {
        write!(w, "stackMap(")?;
        match entry {
            StackMapFrame::SameFrame(s) => {
                current_offset += s.offset_delta;
                write!(w, "{},frame(", current_offset)?;
                write_locals(&locals, w)?;
                write!(w, ",")?;
                write_operand_stack(&operand_stack, w)?;
                write!(w, ",[])")?;
                //todo check if flags needed and then write
            }
            StackMapFrame::AppendFrame(append_frame) => {
                unimplemented!();

                current_offset += append_frame.offset_delta;
                write!(w, "{},frame(", current_offset)?;
            }
            _ => {
                dbg!(entry);
                unimplemented!()
            }
        }
        write!(w, ")")?;

        if i != stack_map.entries.len() - 1 {
            write!(w, ",")?;
        }
    }
    write!(w,"]");
    Ok(())
}

//stackMap(Offset, TypeState)
//
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
//samePackageName(Class1, Class2)
// True iff the package names of Class1 and Class2 are the same.
//differentPack
// ageName(Class1, Class2
//)
//  True iff the package names of Class1 and Class2 are different.
