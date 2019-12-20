use crate::classnames::ClassName;
use crate::classfile::UninitializedVariableInfo;
use crate::loading::Loader;
use std::sync::Arc;
use crate::classnames::get_referred_name;
use crate::loading::class_entry_from_string;
use crate::classfile::Classfile;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayType {
    pub sub_type: Box<UnifiedType>
}

#[derive(Debug)]
pub struct ClassType {
    pub class_name: ClassName,
    pub loader: Arc<Loader>,
}

impl PartialEq for ClassType {
    fn eq(&self, other: &ClassType) -> bool {
        self.class_name == other.class_name &&
            Arc::ptr_eq(&self.loader, &other.loader)
    }
}

impl Eq for ClassType {}


//todo why are there two of these
pub fn class_type_to_class(class_type: &ClassType) -> Arc<Classfile> {
    let class_entry = class_entry_from_string(&get_referred_name(&class_type.class_name), false);
    class_type.loader.loading.read().map(|x| {
        let option = x.get(&class_entry).map(|x|{x.clone()});
        option.or_else(|| {
            let arc = class_type.loader.loaded.read().map(|x| x.get(&class_entry).unwrap().clone()).unwrap();
            Some(arc)
        })
    }).unwrap().unwrap().clone()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum UnifiedType {
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
    UninitializedEmpty,
}