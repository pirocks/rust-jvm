use crate::classnames::ClassName;
use crate::classfile::UninitializedVariableInfo;
use crate::loading::Loader;
use std::sync::Arc;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayType{
    pub sub_type: Box<UnifiedType>
}

#[derive(Debug)]
pub struct ClassType{
    pub class_name: ClassName,
    pub loader: Arc<Loader>
}

impl PartialEq for ClassType{
    fn eq(&self, other: &ClassType) -> bool {
        self.class_name == other.class_name &&
            Arc::ptr_eq(&self.loader,&other.loader)
    }
}

impl Eq for ClassType {}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum UnifiedType{
    ByteType,
    CharType,
    DoubleType,
    FloatType,
    IntType,
    LongType,
    Class(ClassType),
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