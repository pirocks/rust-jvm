use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::classfile::{Classfile, CPIndex, ConstantKind, NameAndType, InterfaceMethodref, Fieldref, MethodHandle, ReferenceKind, Methodref};
use std::sync::Arc;
use crate::view::ClassView;
use crate::view::attribute_view::BootstrapMethodView;
use crate::view::ptype_view::{ReferenceTypeView, PTypeView};
use descriptor_parser::parse_field_descriptor;

#[derive(Debug)]
pub struct Utf8View {
    //todo
}

#[derive(Debug)]
pub struct IntegerView {
    //todo
}

#[derive(Debug)]
pub struct FloatView {
    //todo
}

#[derive(Debug)]
pub struct LongView {
    //todo
}

#[derive(Debug)]
pub struct DoubleView {
    //todo
}

#[derive(Debug)]
pub struct ClassPoolElemView {
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) name_index: usize,
}

impl ClassPoolElemView {
    pub fn class_name(&self) -> ReferenceTypeView {
        //todo should really find a way of getting this into a string pool
        let name_str = self.backing_class.constant_pool[self.name_index].extract_string_from_utf8();

        let type_ = PTypeView::from_ptype(&parse_field_descriptor(&name_str).unwrap().field_type);
        type_.unwrap_ref_type().clone()
        // ClassName::Str(name_str)
    }
}

#[derive(Debug)]
pub struct StringView {
    //todo
}

#[derive(Debug)]
pub struct FieldrefView {
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) i: usize,
}

impl FieldrefView {
    fn field_ref(&self) -> &Fieldref {
        match &self.backing_class.constant_pool[self.i].kind {
            ConstantKind::Fieldref(fr) => fr,
            _ => panic!(),
        }
    }
    pub fn class(&self) -> String {
        self.backing_class.constant_pool[self.backing_class.extract_class_from_constant_pool(self.field_ref().class_index).name_index as usize].extract_string_from_utf8()
    }
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView { backing_class: self.backing_class.clone(), i: self.field_ref().name_and_type_index as usize }
    }
}


#[derive(Debug)]
pub struct MethodrefView {
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) i: usize,
}

impl MethodrefView {
    fn get_raw(&self) -> &Methodref {
        match &self.backing_class.constant_pool[self.i].kind {
            ConstantKind::Methodref(mf) => mf,
            c => {
                dbg!(c);
                panic!()
            }
        }
    }

    pub fn class(&self) -> ClassName {
        let class_index = self.get_raw().class_index;
        ClassName::Str(self.backing_class.extract_class_from_constant_pool_name(class_index))
    }
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView {
            backing_class: self.backing_class.clone(),
            i: self.get_raw().name_and_type_index as usize,
        }
    }
}

#[derive(Debug)]
pub struct InterfaceMethodrefView {
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) i: usize,
}

impl InterfaceMethodrefView {
    fn interface_method_ref(&self) -> &InterfaceMethodref {
        match &self.backing_class.constant_pool[self.i].kind {
            ConstantKind::InterfaceMethodref(imr) => {
                imr
            }
            _ => panic!(),
        }
    }
    pub fn class(&self) -> ClassName {
        ClassName::Str(self.backing_class.extract_class_from_constant_pool_name(self.interface_method_ref().class_index))
    }
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView { backing_class: self.backing_class.clone(), i: self.interface_method_ref().nt_index as usize }
    }
}

#[derive(Debug)]
pub struct NameAndTypeView {
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) i: usize,
}

#[derive(Debug)]
pub struct FieldDescriptorView {
    //todo
}

impl NameAndTypeView {
    fn name_and_type(&self) -> &NameAndType {
        match &self.backing_class.constant_pool[self.i as usize].kind {
            ConstantKind::NameAndType(nt) => {
                nt
            }
            _ => panic!()
        }
    }

    pub fn name(&self) -> String {
        self.backing_class.constant_pool[self.name_and_type().name_index as usize].extract_string_from_utf8()
    }
    pub fn desc(&self) -> String {
        let desc_str = self.backing_class.constant_pool[self.name_and_type().descriptor_index as usize].extract_string_from_utf8();
        desc_str//in future parse
    }
}

#[derive(Debug)]
pub struct MethodHandleView {
    pub(crate) backing_class: Arc<Classfile>,
    pub i: usize,
}

impl MethodHandleView {
    fn get_raw(&self) -> &MethodHandle {
        match &self.backing_class.constant_pool[self.i].kind {
            ConstantKind::MethodHandle(mh) => {
                mh
            }
            _ => panic!(),
        }
    }

    pub fn get_reference_kind(&self) -> ReferenceKind {
        self.get_raw().reference_kind.clone()
    }

    pub fn get_reference_data(&self) -> ReferenceData {
        match self.get_raw().reference_kind {
            ReferenceKind::GetField => unimplemented!(),
            ReferenceKind::GetStatic => unimplemented!(),
            ReferenceKind::PutField => unimplemented!(),
            ReferenceKind::PutStatic => unimplemented!(),
            ReferenceKind::InvokeVirtual => unimplemented!(),
            ReferenceKind::InvokeStatic => {
                assert!(self.backing_class.major_version >= 52);
                //if the class file
                // version number is 52.0 or above, the constant_pool entry at that
                // index must be either a CONSTANT_Methodref_info structure or a
                // CONSTANT_InterfaceMethodref_info structure (§4.4.2) representing a
                // class's or interface's method for which a method handle is to be created.
                let reference_idx = self.get_raw().reference_index as usize;
                let invoke_static = match &self.backing_class.constant_pool[reference_idx].kind {
                    ConstantKind::Methodref(_) => InvokeStatic::Method(MethodrefView { backing_class: self.backing_class.clone(), i: reference_idx }),
                    ConstantKind::InterfaceMethodref(_) => InvokeStatic::Interface(InterfaceMethodrefView { backing_class: self.backing_class.clone(), i: reference_idx }),
                    ck => {
                        dbg!(ck);
                        panic!()
                    }
                };
                ReferenceData::InvokeStatic(invoke_static)
            }
            ReferenceKind::InvokeSpecial => unimplemented!(),
            ReferenceKind::NewInvokeSpecial => unimplemented!(),
            ReferenceKind::InvokeInterface => unimplemented!(),
        }
    }
}

//todo need a better name
pub enum ReferenceData {
    InvokeStatic(InvokeStatic),
}

pub enum InvokeStatic {
    Interface(InterfaceMethodrefView),
    Method(MethodrefView),
}

#[derive(Debug)]
pub struct MethodTypeView {
    //todo
}

#[derive(Debug)]
pub struct DynamicView {
    //todo
}

#[derive(Debug)]
pub struct InvokeDynamicView {
    pub(crate) backing_class: ClassView,
    pub(crate) bootstrap_method_attr_index: u16,
    pub(crate) name_and_type_index: CPIndex,
}

impl InvokeDynamicView {
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView {
            backing_class: self.backing_class.backing_class.clone(),
            i: self.name_and_type_index as usize,
        }
    }
    pub fn bootstrap_method(&self) -> BootstrapMethodView {
        BootstrapMethodView { backing: self.backing_class.bootstrap_methods_attr(), i: self.bootstrap_method_attr_index as usize }
    }
}

#[derive(Debug)]
pub struct ModuleView {
    //todo
}

#[derive(Debug)]
pub struct PackageView {
    //todo
}

#[derive(Debug)]
pub struct InvalidConstantView {
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
    InvalidConstant(InvalidConstantView),
    LiveObject(usize),
}

impl ConstantInfoView {
    pub fn unwrap_class(&self) -> &ClassPoolElemView {
        match self {
            ConstantInfoView::Class(c) => { c }
            _ => panic!(),
        }
    }
}