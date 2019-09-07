use classfile::parsing_util::{ParsingContext, read8, read16};
use std::any::Any;
use std::io::Read;
use classfile::attribute_infos::AttributeType::ConstantValue;

#[derive(Debug)]
pub struct Utf8 {
    pub length : u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct Integer{
    //unimplemented!()
}

#[derive(Debug)]
pub struct Float{
    //unimplemented!()
}

#[derive(Debug)]
pub struct Long{
    //unimplemented!()
}

#[derive(Debug)]
pub struct Double{
    //unimplemented!()
}

#[derive(Debug)]
pub struct Class{
    //unimplemented!()
    pub name_index: u16
}

#[derive(Debug)]
pub struct String{
    //unimplemented!()
    pub string_index: u16
}

#[derive(Debug)]
pub struct Fieldref{
    //unimplemented!()
    pub class_index: u16,
    pub name_and_type_index: u16
}

#[derive(Debug)]
pub struct Methodref{
    pub class_index: u16,
    pub name_and_type_index: u16
}

#[derive(Debug)]
pub struct InterfaceMethodref{
    //unimplemented!()
}

#[derive(Debug)]
pub struct NameAndType{
    //unimplemented!()
    pub name_index: u16,
    pub descriptor_index: u16
}

#[derive(Debug)]
pub struct MethodHandle{
    //unimplemented!()
}

#[derive(Debug)]
pub struct MethodType{
    //unimplemented!()
}

#[derive(Debug)]
pub struct Dynamic{
    //unimplemented!()
}

#[derive(Debug)]
pub struct InvokeDynamic{
    //unimplemented!()
}

#[derive(Debug)]
pub struct Module{
    //unimplemented!()
}

#[derive(Debug)]
pub struct Package{
    //unimplemented!()
}

#[derive(Debug)]
pub struct InvalidConstant {}

#[derive(Debug)]
pub enum ConstantKind {
    Utf8(Utf8),
    Integer(Integer),
    Float(Float),
    Long(Long),
    Double(Double),
    Class(Class),
    String(String),
    Fieldref(Fieldref),
    Methodref(Methodref),
    InterfaceMethodref(InterfaceMethodref),
    NameAndType(NameAndType),
    MethodHandle(MethodHandle),
    MethodType(MethodType),
    Dynamic(Dynamic),
    InvokeDynamic(InvokeDynamic),
    Module(Module),
    Package(Package),
    InvalidConstant(InvalidConstant),
}

pub fn is_utf8(utf8 : &ConstantKind) -> Option<&Utf8>{
    return match utf8 {
        ConstantKind::Utf8(s) => { Some(s) },
        _ => { None }
    }
}

#[derive(Debug)]
pub struct ConstantInfo {
    pub kind: ConstantKind,
}



/*
struct cp_info readCPInfo(FILE *file) {
    struct cp_info result;
    enum ConstantKind tag = read8(file);
    result.tag = tag;
    switch (tag) {
        case CONSTANT_Class:
            result.constantClassInfo.name_index = read16(file);
            break;
        case CONSTANT_Fieldref:
            result.constantFieldrefInfo.class_index = read16(file);
            result.constantFieldrefInfo.name_and_type_index = read16(file);
            break;
        case CONSTANT_Methodref:
            result.constantMethodrefInfo.class_index = read16(file);
            result.constantMethodrefInfo.name_and_type_index = read16(file);
            break;
        case CONSTANT_InterfaceMethodref:
            result.constantInterfaceMethodrefInfo.class_index = read16(file);
            result.constantInterfaceMethodrefInfo.name_and_type_index = read16(file);
            break;
        case CONSTANT_String:
            result.constantStringInfo.string_index = read16(file);
            break;
        case CONSTANT_Integer:
            result.constantIntegerInfo.bytes = read32(file);
            break;
        case CONSTANT_Float:
            result.constantFloatInfo.bytes = read32(file);
            break;
        case CONSTANT_Long: {
            uint64_t high = read32(file);
            uint64_t low = read32(file);
            result.constantLongInfo.bytes = (high << 32u) + low;
            //these take up two bytes on constant pool table.
        }
            break;
        case CONSTANT_Double: {
            uint64_t high = read32(file);
            uint64_t low = read32(file);
            result.constantDoubleInfo.bytes = (high << 32u) + low;
            //these take up two bytes on constant pool table.
        }
            break;
        case CONSTANT_NameAndType:
            result.constantNameAndTypeInfo.name_index = read16(file);
            result.constantNameAndTypeInfo.descriptor_index = read16(file);
            break;
        case CONSTANT_Utf8:
            result.constantUtf8Info.length = read16(file);
            result.constantUtf8Info.bytes = malloc(result.constantUtf8Info.length * sizeof(uint8_t)
                                                   + sizeof(char));//+1 byte for null termiator
            memset(result.constantUtf8Info.bytes,
                   0,
                   result.constantUtf8Info.length * sizeof(uint8_t) + sizeof(char));
            fread(result.constantUtf8Info.bytes, sizeof(uint8_t), result.constantUtf8Info.length, file);
            break;
        case CONSTANT_MethodHandle:
            result.constantMethodHandleInfo.reference_kind = read8(file);
            result.constantMethodHandleInfo.reference_index = read16(file);
            break;
        case CONSTANT_MethodType:
            result.constantMethodTypeInfo.descriptor_index = read16(file);
            break;
        case CONSTANT_Dynamic:
            result.constantDynamicInfo.bootstrap_method_attr_index = read16(file);
            result.constantDynamicInfo.name_and_type_index = read16(file);
            break;
        case CONSTANT_InvokeDynamic:
            result.constantInvokeDynamicInfo.bootstrap_method_attr_index = read16(file);
            result.constantInvokeDynamicInfo.name_and_type_index = read16(file);
            break;
        case CONSTANT_Module:
            result.constantModuleInfo.name_index = read16(file);
            break;
        case CONSTANT_Package:
            result.constantPackageInfo.name_index = read16(file);
            break;
        default:
            assert(false);
    }
    return result;
}
*/


const UTF8_CONST_NUM: u8 = 1;
const INTEGER_CONST_NUM: u8 = 3;
const FLOAT_CONST_NUM: u8 = 4;
const LONG_CONST_NUM: u8 = 5;
const DOUBLE_CONST_NUM: u8 = 6;
const CLASS_CONST_NUM: u8 = 7;
const STRING_CONST_NUM: u8 = 8;
const FIELDREF_CONST_NUM: u8 = 9;
const METHODREF_CONST_NUM: u8 = 10;
const INTERFACE_METHODREF_CONST_NUM: u8 = 11;
const NAME_AND_TYPE_CONST_NUM: u8 = 12;
const METHOD_HANDLE_CONST_NUM: u8 = 15;
const METHOD_TYPE_CONST_NUM: u8 = 16;
const DYNAMIC_CONST_NUM: u8 = 17;
const INVOKE_DYNAMIC_CONST_NUM: u8 = 18;
const MODULE_CONST_NUM: u8 = 19;
const PACKAGE_CONST_NUM: u8 = 20;
const INVALID_CONSTANT_CONST_NUM: u8 = 21;

pub fn parse_constant_info(p: &mut ParsingContext) -> ConstantInfo{
    let kind = read8(p);
    let result_kind = match kind {
        UTF8_CONST_NUM => {
            let length = read16(p);
            let mut buffer = Vec::new();
            for _ in 0..length{
                buffer.push(read8(p))
            }
            buffer.push('\0' as u8);
            ConstantKind::Utf8( Utf8 { length : length, bytes:buffer } )
        },
        INTEGER_CONST_NUM => { unimplemented!() },
        FLOAT_CONST_NUM => { unimplemented!() },
        LONG_CONST_NUM => { unimplemented!() },
        DOUBLE_CONST_NUM => { unimplemented!() },
        CLASS_CONST_NUM => {
            let name_index = read16(p);
            ConstantKind::Class( Class { name_index } )
        },
        STRING_CONST_NUM => {
            let string_index = read16(p);
            ConstantKind::String( String { string_index } )
        },
        FIELDREF_CONST_NUM => {
            let class_index = read16(p);
            let name_and_type_index = read16(p);
            ConstantKind::Fieldref( Fieldref {class_index,name_and_type_index})
        },
        METHODREF_CONST_NUM => {
            let class_index = read16(p);
            let name_and_type_index = read16(p);
            ConstantKind::Methodref( Methodref {class_index,name_and_type_index})
        },
        INTERFACE_METHODREF_CONST_NUM => { unimplemented!() },
        NAME_AND_TYPE_CONST_NUM => {
            let name_index = read16(p);
            let descriptor_index = read16(p);
            ConstantKind::NameAndType( NameAndType { name_index,descriptor_index } )
        },
        METHOD_HANDLE_CONST_NUM => { unimplemented!() },
        METHOD_TYPE_CONST_NUM => { unimplemented!() },
        DYNAMIC_CONST_NUM => { unimplemented!() },
        INVOKE_DYNAMIC_CONST_NUM => { unimplemented!() },
        MODULE_CONST_NUM => { unimplemented!() },
        PACKAGE_CONST_NUM => { unimplemented!() },
        INVALID_CONSTANT_CONST_NUM => {
            assert!(false);
            unimplemented!();
        },
        _ => {
            assert!(false);
            unimplemented!();
        }
    };
    return ConstantInfo {kind: result_kind };
}


pub fn parse_constant_infos(p: &mut ParsingContext, constant_pool_count: u16) {
    p.constants = Vec::with_capacity(constant_pool_count as usize);
    let invalid_constant = ConstantInfo { kind: (ConstantKind::InvalidConstant(InvalidConstant {})) };
    let mut skip_next_iter = true;
    //skip first loop iteration b/c the first element of the constant pool isn't a thing
    for _ in 0..constant_pool_count {
        if skip_next_iter {
            p.constants.push(ConstantInfo { kind: (ConstantKind::InvalidConstant(InvalidConstant {})) });
            skip_next_iter = false;
            continue
        }
        let constant_info = parse_constant_info(p);
        if (constant_info).kind.type_id() == ConstantKind::Double.type_id() || (constant_info).kind.type_id() == ConstantKind::Long.type_id() {
            skip_next_iter = true;
        }
        p.constants.push(constant_info);
    }
}
