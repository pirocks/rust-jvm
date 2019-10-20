use classfile::Classfile;
use classfile::attribute_infos::UninitializedVariableInfo;
use verification::prolog_info_writer::extract_string_from_utf8;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct NameReference<'l>{
    pub class_file: &'l Classfile<'l>,
    pub index : u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum ClassNameReference<'l> {
    Ref(NameReference<'l>),
    Str(String)
}

pub fn get_referred_name<'l>(ref_ : &'l ClassNameReference<'l>) -> &'l str{
    match ref_{
        ClassNameReference::Ref(r) => {
            unimplemented!()
//            extract_string_from_utf8(&r.class_file.constant_pool[r.index as usize]).as_str()
        },
        ClassNameReference::Str(s) => {s},
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayType<'l> {
    pub sub_type: &'l UnifiedType<'l>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum UnifiedType<'l>{
    ByteType,
    CharType,
    DoubleType,
    FloatType,
    IntType,
    LongType,
    ReferenceType(ClassNameReference<'l>),
    ShortType,
    BooleanType,
    ArrayReferenceType(ArrayType<'l>),
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis
}