use crate::classnames::ClassName;
use crate::classfile::{Classfile, CPIndex, ConstantKind, NameAndType};
use std::sync::Arc;

pub struct Utf8View{
    //todo
}

pub struct IntegerView{
    //todo
}

pub struct FloatView{
    //todo
}

pub struct LongView{
    //todo
}

pub struct DoubleView{
    //todo
}

pub struct ClassPoolElemView{
    backing_class : Arc<Classfile>,
    name_index : usize
}

impl ClassPoolElemView{
    pub fn class_name(&self) -> ClassName{
        ClassName::Str(self.backing_class.constant_pool[self.name_index].extract_string_from_utf8())//todo should really find a way of getting this into a string pool
    }
}

pub struct StringView{
    //todo
}

pub struct FieldrefView{
    //todo
}

pub struct MethodrefView{
    backing_class: Arc<Classfile>,
    class_index: CPIndex,//todo replace with one i:usize
    name_and_type_index: CPIndex,
}

impl MethodrefView{
    fn class(&self) -> ClassName{
        ClassName::Str(self.backing_class.constant_pool[self.class_index as usize].extract_string_from_utf8())
    }
    fn name_and_type(&self) -> NameAndTypeView{
        NameAndTypeView {
            backing_class: self.backing_class.clone(),
            i: self.name_and_type_index as usize
        }
    }
}

pub struct InterfaceMethodrefView{
    //todo
}

pub struct NameAndTypeView{
    backing_class: Arc<Classfile>,
    i : usize,
}

pub struct FieldDescriptorView{
    //todo
}

impl NameAndTypeView{
    fn name_and_type(&self) -> &NameAndType{
        match &self.backing_class.constant_pool[self.i as usize].kind{
            ConstantKind::NameAndType(nt) => {
                nt
            },
            _=>panic!()
        }
    }

    pub fn name(&self) -> String{
        self.backing_class.constant_pool[self.name_and_type().name_index as usize].extract_string_from_utf8()
    }
    pub fn desc(&self) -> String{
        let desc_str = self.backing_class.constant_pool[self.name_and_type().descriptor_index as usize].extract_string_from_utf8();
        desc_str//in future parse
    }
}

pub struct MethodHandleView{
    //todo
}

pub struct MethodTypeView{
    //todo
}

pub struct DynamicView{
    //todo
}

pub struct InvokeDynamicView{
    backing_class: Arc<Classfile>,
    bootstrap_method_attr_index: CPIndex,
    name_and_type_index: CPIndex,
}

pub struct ModuleView{
    //todo
}

pub struct PackageView{
    //todo
}

pub struct InvalidConstantView{
    //todo
}

pub enum ConstantInfoView {
    Utf8(Utf8View),
    Integer(IntegerView),
    Float(FloatView),
    Long(LongView),
    Double(DoubleView),
    Class(ClassPoolElemView),
    String(StringView),
    Fieldref(FieldrefView),
    Methodref(MethodrefView),
    InterfaceMethodref(InterfaceMethodrefView),
    NameAndType(NameAndTypeView),
    MethodHandle(MethodHandleView),
    MethodType(MethodTypeView),
    Dynamic(DynamicView),
    InvokeDynamic(InvokeDynamicView),
    Module(ModuleView),
    Package(PackageView),
    InvalidConstant(InvalidConstantView)
}
