
use classfile::parsing_util::{ParsingContext, read16, read8, read32};

#[derive(Debug)]
#[derive(Eq)]
pub struct Utf8 {
    pub length : u16,
    pub string: String,
}

impl PartialEq for Utf8{
    fn eq(&self, other: &Self) -> bool {
        return self.length == other.length &&
            self.string == other.string;
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Integer{
    //unimplemented!()
    pub bytes: u32
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Float{
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Long{
    //unimplemented!()
    pub low_bytes: u32,
    pub high_bytes: u32
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Double{
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Class{
    //unimplemented!()
    pub name_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct String_{
    //unimplemented!()
    pub string_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Fieldref{
    //unimplemented!()
    pub class_index: u16,
    pub name_and_type_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Methodref{
    pub class_index: u16,
    pub name_and_type_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InterfaceMethodref{
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct NameAndType{
    //unimplemented!()
    pub name_index: u16,
    pub descriptor_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodHandle{
    //unimplemented!()
    pub reference_kind: u8,
    pub reference_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodType{
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Dynamic{
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InvokeDynamic{
    //unimplemented!()
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Module{
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Package{
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InvalidConstant {}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
//#[derive(Copy, Clone)]
pub enum ConstantKind {
    Utf8(Utf8),
    Integer(Integer),
    Float(Float),
    Long(Long),
    Double(Double),
    Class(Class),
    String(String_),
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
#[derive(Eq)]
//#[derive(Copy, Clone)]
pub struct ConstantInfo {
    pub kind: ConstantKind,
}

impl PartialEq for ConstantInfo {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
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
            let str_ = String::from_utf8(buffer).expect("Invalid utf8 in constant pool");
            ConstantKind::Utf8( Utf8 { length, string: str_ } )
        },
        INTEGER_CONST_NUM => {
            let bytes = read32(p);
            ConstantKind::Integer(Integer {bytes})
        },
        FLOAT_CONST_NUM => { unimplemented!() },
        LONG_CONST_NUM => {
            let high_bytes = read32(p);
            let low_bytes = read32(p);
            ConstantKind::Long(Long {high_bytes, low_bytes })
        },
        DOUBLE_CONST_NUM => { unimplemented!() },
        CLASS_CONST_NUM => {
            let name_index = read16(p);
            ConstantKind::Class( Class { name_index } )
        },
        STRING_CONST_NUM => {
            let string_index = read16(p);
            ConstantKind::String( String_ { string_index } )
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
        METHOD_HANDLE_CONST_NUM => {
            let reference_kind = read8(p);
            let reference_index = read16(p);
            ConstantKind::MethodHandle(MethodHandle {
                reference_kind, reference_index
            })
        },
        METHOD_TYPE_CONST_NUM => { unimplemented!() },
        DYNAMIC_CONST_NUM => { unimplemented!() },
        INVOKE_DYNAMIC_CONST_NUM => {
            let bootstrap_method_attr_index = read16(p);
            let name_and_type_index = read16(p);
            ConstantKind::InvokeDynamic(InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            })
        },
        MODULE_CONST_NUM => { unimplemented!() },
        PACKAGE_CONST_NUM => { unimplemented!() },
        INVALID_CONSTANT_CONST_NUM => {
            assert!(false);
            unimplemented!();
        },
        _ => {
            dbg!(kind);
            assert!(false);
            unimplemented!();
        }
    };
    return ConstantInfo { kind: result_kind };
}


pub fn parse_constant_infos(p: &mut ParsingContext, constant_pool_count: u16) -> Vec<ConstantInfo> {
    let mut constants = Vec::with_capacity(constant_pool_count as usize);
    let mut skip_next_iter = true;
    //skip first loop iteration b/c the first element of the constant pool isn't a thing
    for i in 0..constant_pool_count {
        if skip_next_iter {
            constants.push(ConstantInfo { kind: (ConstantKind::InvalidConstant(InvalidConstant {})) });
            skip_next_iter = false;
            continue
        }
        let constant_info = parse_constant_info(p);
//        dbg!(&constant_info);
//        dbg!(i);
        match constant_info.kind{
            ConstantKind::Long(_) | ConstantKind::Double(_)  => {
                skip_next_iter = true;
            },
            _ => {}
        }
        constants.push(constant_info);
    }
    return constants;
}
