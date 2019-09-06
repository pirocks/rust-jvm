use classfile::parsing_util::{ParsingContext, read8};
use std::any::Any;

pub struct Utf8 {
    pub utf8_string: String,
    pub bytes: Vec<u8>,
}

pub struct Integer{
    //todo
}

pub struct Float{
    //todo
}

pub struct Long{
    //todo
}

pub struct Double{
    //todo
}

pub struct Class{
    //todo
}

pub struct String{
    //todo
}

pub struct Fieldref{
    //todo
}

pub struct Methodref{
    //todo
}

pub struct InterfaceMethodref{
    //todo
}

pub struct NameAndType{
    //todo
}

pub struct MethodHandle{
    //todo
}

pub struct MethodType{
    //todo
}

pub struct Dynamic{
    //todo
}

pub struct InvokeDynamic{
    //todo
}

pub struct Module{
    //todo
}

pub struct Package{
    //todo
}

pub struct InvalidConstant {}


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
            //todo free
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
    match kind {
        UTF8_CONST_NUM => { todo!() },
        INTEGER_CONST_NUM => { todo!() },
        FLOAT_CONST_NUM => { todo!() },
        LONG_CONST_NUM => { todo!() },
        DOUBLE_CONST_NUM => { todo!() },
        CLASS_CONST_NUM => { todo!() },
        STRING_CONST_NUM => { todo!() },
        FIELDREF_CONST_NUM => { todo!() },
        METHODREF_CONST_NUM => { todo!() },
        INTERFACE_METHODREF_CONST_NUM => { todo!() },
        NAME_AND_TYPE_CONST_NUM => { todo!() },
        METHOD_HANDLE_CONST_NUM => { todo!() },
        METHOD_TYPE_CONST_NUM => { todo!() },
        DYNAMIC_CONST_NUM => { todo!() },
        INVOKE_DYNAMIC_CONST_NUM => { todo!() },
        MODULE_CONST_NUM => { todo!() },
        PACKAGE_CONST_NUM => { todo!() },
        INVALID_CONSTANT_CONST_NUM => {
            assert!(false);
        },
        _ => {
            assert!(false);
        }
    }
    todo!()
}


pub fn parse_constant_infos(p: &mut ParsingContext, constant_pool_count: u16) -> Vec<ConstantInfo> {
    let mut res = Vec::with_capacity(constant_pool_count as usize);
    let invalid_constant = ConstantInfo { kind: (ConstantKind::InvalidConstant(InvalidConstant {})) };
    let mut skip_next_iter = true;
    //skip first loop iteration b/c the first element of the constant pool isn't a thing
    for _ in 0..constant_pool_count {
        if skip_next_iter {
            res.push(ConstantInfo { kind: (ConstantKind::InvalidConstant(InvalidConstant {})) });
            skip_next_iter = false;
            continue
        }
        let constant_info = parse_constant_info(p);
        if (constant_info).kind.type_id() == ConstantKind::Double.type_id() || (constant_info).kind.type_id() == ConstantKind::Long.type_id() {
            skip_next_iter = true;
        }
        res.push(constant_info);
    }
    return res;
}
