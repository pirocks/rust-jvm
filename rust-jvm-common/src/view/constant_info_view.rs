use crate::classnames::ClassName;
use crate::classfile::{Classfile, CPIndex, ConstantKind, NameAndType, InterfaceMethodref, Fieldref, BootstrapMethod};
use std::sync::Arc;
use crate::view::ClassView;

#[derive(Debug)]
pub struct Utf8View{
    //todo
}

#[derive(Debug)]
pub struct IntegerView{
    //todo
}

#[derive(Debug)]
pub struct FloatView{
    //todo
}

#[derive(Debug)]
pub struct LongView{
    //todo
}

#[derive(Debug)]
pub struct DoubleView{
    //todo
}

#[derive(Debug)]
pub struct ClassPoolElemView{
    pub(crate) backing_class : Arc<Classfile>,
    pub(crate) name_index : usize
}

impl ClassPoolElemView{
    pub fn class_name(&self) -> ClassName{
        ClassName::Str(self.backing_class.constant_pool[self.name_index].extract_string_from_utf8())//todo should really find a way of getting this into a string pool
    }
}

#[derive(Debug)]
pub struct StringView{
    //todo
}

#[derive(Debug)]
pub struct FieldrefView{
    pub(crate) backing_class : Arc<Classfile>,
    pub(crate) i: usize
}
impl FieldrefView {
    fn field_ref(&self) -> &Fieldref{
        match &self.backing_class.constant_pool[self.i].kind{
            ConstantKind::Fieldref(fr) => fr,
            _ => panic!(),
        }
    }
    pub fn class(&self) -> String{
        self.backing_class.constant_pool[self.backing_class.extract_class_from_constant_pool(self.field_ref().class_index).name_index as usize].extract_string_from_utf8()
    }
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView { backing_class: self.backing_class.clone(), i: self.field_ref().name_and_type_index as usize }
    }
}


#[derive(Debug)]
pub struct MethodrefView{
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) class_index: CPIndex,//todo replace with one i:usize
    pub(crate) name_and_type_index: CPIndex,
}

impl MethodrefView{
    pub fn class(&self) -> ClassName{
        ClassName::Str(self.backing_class.extract_class_from_constant_pool_name(self.class_index))
    }
    pub fn name_and_type(&self) -> NameAndTypeView{
        NameAndTypeView {
            backing_class: self.backing_class.clone(),
            i: self.name_and_type_index as usize
        }
    }
}

#[derive(Debug)]
pub struct InterfaceMethodrefView{
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) i : usize,
}

impl InterfaceMethodrefView{
    fn interface_method_ref(&self) -> &InterfaceMethodref {
        match &self.backing_class.constant_pool[self.i].kind{
            ConstantKind::InterfaceMethodref(imr) => {
                imr
            },
            _ => panic!(),
        }
    }
    pub fn class(&self) -> ClassName{
        ClassName::Str(self.backing_class.extract_class_from_constant_pool_name(self.interface_method_ref().class_index))
    }
    pub fn name_and_type(&self) -> NameAndTypeView{
        NameAndTypeView { backing_class: self.backing_class.clone(), i: self.interface_method_ref().nt_index as usize }
    }
}

#[derive(Debug)]
pub struct NameAndTypeView{
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) i : usize,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct MethodHandleView{
    //todo
}

#[derive(Debug)]
pub struct MethodTypeView{
    //todo
}

#[derive(Debug)]
pub struct DynamicView{
    //todo
}

#[derive(Debug)]
pub struct InvokeDynamicView{
    pub(crate) backing_class: ClassView,
    pub(crate) bootstrap_method_attr_index: u16,
    pub(crate) name_and_type_index: CPIndex,
}

impl InvokeDynamicView{
    pub fn name_and_type(&self) -> NameAndTypeView{
        NameAndTypeView { backing_class: self.backing_class.clone(), i: self.name_and_type_index as usize }
    }
    pub fn bootstrap_method_attr(&self) -> BootstrapMethodView{
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct ModuleView{
    //todo
}

#[derive(Debug)]
pub struct PackageView{
    //todo
}

#[derive(Debug)]
pub struct InvalidConstantView{
    //todo
}

#[derive(Debug)]
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

impl ConstantInfoView{
    pub fn unwrap_class(&self) -> &ClassPoolElemView{
        match self{
            ConstantInfoView::Class(c) => {c},
            _ => panic!(),
        }
    }
}