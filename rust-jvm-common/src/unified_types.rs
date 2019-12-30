use crate::classnames::ClassName;
use crate::classfile::UninitializedVariableInfo;
use crate::loading::Loader;
use std::sync::Arc;
use crate::classnames::get_referred_name;
use crate::loading::class_entry_from_string;
use crate::classfile::Classfile;
use crate::classnames::NameReference;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayType {
    pub sub_type: Box<UnifiedType>
}

//#[derive(Debug)]
pub struct ClassWithLoader {
    pub class_name: ClassName,
    pub loader: Arc<Loader>,
}

impl PartialEq for ClassWithLoader {
    fn eq(&self, other: &ClassWithLoader) -> bool {
        self.class_name == other.class_name &&
            Arc::ptr_eq(&self.loader, &other.loader)
    }
}

impl Eq for ClassWithLoader {}


impl Debug for ClassWithLoader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f,"<{},{}>",get_referred_name(&self.class_name),self.loader.name)
    }
}

//todo why are there two of these
pub fn class_type_to_class(class_type: &ClassWithLoader) -> Arc<Classfile> {
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
    Class(ClassWithLoader),
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


impl Clone for UnifiedType{
    fn clone(&self) -> Self {
        copy_recurse(self)
    }
}


fn copy_recurse(to_copy: &UnifiedType) -> UnifiedType {

    match to_copy {
        UnifiedType::Class(o) => {
            let class_name = match &o.class_name {
                ClassName::Ref(r) => { ClassName::Ref(NameReference { class_file: r.class_file.clone(), index: r.index }) }
                ClassName::Str(s) => { ClassName::Str(s.clone()) }
            };
            UnifiedType::Class(ClassWithLoader {class_name, loader: o.loader.clone() })
        }
        UnifiedType::Uninitialized(u) => UnifiedType::Uninitialized(UninitializedVariableInfo { offset: u.offset }),
        UnifiedType::ArrayReferenceType(a) => UnifiedType::ArrayReferenceType(ArrayType { sub_type: Box::from(copy_recurse(&a.sub_type)) }),

        UnifiedType::TopType => UnifiedType::TopType,
        UnifiedType::IntType => UnifiedType::IntType,
        UnifiedType::FloatType => UnifiedType::FloatType,
        UnifiedType::LongType => UnifiedType::LongType,
        UnifiedType::DoubleType => UnifiedType::DoubleType,
        UnifiedType::NullType => UnifiedType::NullType,
        UnifiedType::UninitializedThis => UnifiedType::UninitializedThis,
        _ => { panic!("Case wasn't covered with non-unified types") }
    }
}