use classfile::Classfile;
use classfile::attribute_infos::UninitializedVariableInfo;
use std::rc::Weak;

#[derive(Debug)]
pub struct NameReference{
    pub class_file: Weak<Classfile>,
    pub index : u16,
}

impl Eq for NameReference {}

//pub fn rc_ptr_eq<T: ?Sized>(this: &Rc<T>, other: &Rc<T>) -> bool {
//    unsafe  {
//        let this_ptr: *const T = &*this;
//        let other_ptr: *const T = &*other;
//        this_ptr == other_ptr
//    }
//}

impl PartialEq for NameReference{
    fn eq(&self, other: &NameReference) -> bool{
//        assert!(rc_ptr_eq(self.class_file,&other.class_file));
        &self.class_file.upgrade() == &other.class_file.upgrade() && self.index == other.index
    }
}

#[derive(Debug)]
#[derive(Eq)]
pub enum ClassNameReference {
    Ref(NameReference),
    Str(String)
}

impl PartialEq for ClassNameReference{
    fn eq(&self, other: &ClassNameReference) -> bool{
        match self{
            ClassNameReference::Ref(r1) => match other {
                ClassNameReference::Ref(r2) => {
                    //todo how is equality for classfiles defined here?
                    r1.class_file.ptr_eq(&r2.class_file) && r1.index == r2.index
                }
                ClassNameReference::Str(_) => {
                    unimplemented!()
                }
            },
            ClassNameReference::Str(s1) => match other {
                ClassNameReference::Str(s2) => {
                    s1 == s2
                }
                ClassNameReference::Ref(_) => {
                    unimplemented!()
                }
            },
        }
    }
}

impl std::clone::Clone for ClassNameReference{
    fn clone(&self) -> Self {
        match self{
            ClassNameReference::Ref(r) => {
                ClassNameReference::Ref(NameReference {
                    index:  r.index,
                    class_file: r.class_file.clone()
                })
            },
            ClassNameReference::Str(s) => {
                ClassNameReference::Str(s.clone())//todo fix
            },
        }
    }
}

pub fn get_referred_name(ref_ : &ClassNameReference) -> &String{
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
    ReferenceType(ClassNameReference),
    ShortType,
    BooleanType,
    ArrayReferenceType(ArrayType),
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis
}