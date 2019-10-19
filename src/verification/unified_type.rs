use classfile::Classfile;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct NameReference<'l>{
    pub class_file: &'l Classfile<'l>,
    pub index : u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum PrologClassName<'l> {
    Ref(NameReference<'l>),
    Str(&'l str)
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
    ReferenceType(PrologClassName<'l>),
    ShortType,
    BooleanType,
    ArrayReferenceType(ArrayType<'l>),
    VoidType,
    TopType,
    NullType
}