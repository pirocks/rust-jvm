use classfile::attribute_infos::UninitializedVariableInfo;
use verification::classnames::ClassName;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayType{
    pub sub_type: Box<UnifiedType>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum UnifiedType{
    ByteType,
    CharType,
    DoubleType,
    FloatType,
    IntType,
    LongType,
    Class(ClassName),
    ShortType,
    BooleanType,
    ArrayReferenceType(ArrayType),
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis,
    //below here used internally in isAssignable

    TwoWord,
    OneWord,
    Reference,
    UninitializedEmpty
}