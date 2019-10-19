
#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct PrologClassName<'l> {
    pub name: &'l str
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
    TopType
}