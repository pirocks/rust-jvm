#![feature(repr128)]
#![feature(arbitrary_enum_discriminant)]


mod constant_infos;
mod attribute_infos;

pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub attribute_type: attribute_infos::AttributeType,
}

pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,//[attributes_count];
}

pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>// [attributes_count];
//    struct Code_attribute * code_attribute;
}


const EXPECTED_CLASSFILE_MAGIC: u32 = 0xCAFEBABE;

#[repr(u8)]
pub enum ConstantKind {
    Utf8(constant_infos::Utf8) = 1,
    Integer(constant_infos::Integer) = 3,
    Float(constant_infos::Float) = 4,
    Long(constant_infos::Long) = 5,
    Double(constant_infos::Double) = 6,
    Class(constant_infos::Class) = 7,
    String(constant_infos::String) = 8,
    Fieldref(constant_infos::Fieldref) = 9,
    Methodref(constant_infos::Methodref) = 10,
    InterfaceMethodref(constant_infos::InterfaceMethodref) = 11,
    NameAndType(constant_infos::NameAndType) = 12,
    MethodHandle(constant_infos::MethodHandle) = 15,
    MethodType(constant_infos::MethodType) = 16,
    Dynamic(constant_infos::Dynamic) = 17,
    InvokeDynamic(constant_infos::InvokeDynamic) = 18,
    Module(constant_infos::Module) = 19,
    Package(constant_infos::Package) = 20,
}


pub struct ConstantInfo {
    pub kind: ConstantKind,
}
//bitflag! {
//    pub struct ClassAccessFlags{
//        //TODO THIS NEEDS TO BE DIFFERENT FOR DIFFERNT TYPES
//        //todo probably should just use u16 + arithmeti
//    //maybe not but at very least is incomplete
//    ACC_PUBLIC = 0X0001,
//    ACC_PRIVATE = 0x0002,
//    ACC_PROTECTED = 0x0004,
//    ACC_STATIC = 0x0008,
//    ACC_FINAL = 0X0010,
//    ACC_SUPER = 0X0020,
//    ACC_BRIDGE = 0X0040,
//    ACC_VOLATILE = 0x0040,
//    ACC_TRANSIENT = 0x0080,
//    ACC_NATIVE = 0x0100,
//    ACC_INTERFACE = 0X0200,
//    ACC_ABSTRACT = 0X0400,
//    ACC_STRICT = 0x0800,
//    ACC_SYNTHETIC = 0X1000,
//    ACC_ANNOTATION = 0X2000,
//    ACC_ENUM = 0X4000,
//    ACC_MODULE = 0X8000
//    }
//}

pub struct Classfile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub const_pool_count: u16,
    pub const_pool: Vec<ConstantInfo>,
    //    pub access_flags: ClassAccessFlags,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces_count: u16,
    pub interfaces: Vec<u16>,
    //todo use vec with capacity
    pub fields_count: u16,
    pub fields: Vec<FieldInfo>,
    pub methods_count: u16,
    pub methods: Vec<MethodInfo>,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}