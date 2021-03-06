use std::sync::Arc;

use rust_jvm_common::classfile::{Classfile, ConstantKind, CPIndex, Fieldref, InterfaceMethodref, MethodHandle, Methodref, MethodType, NameAndType, ReferenceKind};
use rust_jvm_common::descriptor_parser::{MethodDescriptor, parse_class_name, parse_method_descriptor};
use rust_jvm_common::ptype::PType;

use crate::view::{ClassBackedView, ClassView};
use crate::view::attribute_view::BootstrapMethodView;
use crate::view::ptype_view::{PTypeView, ReferenceTypeView};

#[derive(Debug)]
pub struct Utf8View {
    pub str: String
}

#[derive(Debug)]
pub struct IntegerView {
    pub int: i32
}

#[derive(Debug)]
pub struct FloatView {
    pub float: f32
}

#[derive(Debug)]
pub struct LongView {
    pub long: i64
}

#[derive(Debug)]
pub struct DoubleView {
    pub double: f64
}

#[derive(Debug)]
pub struct ClassPoolElemView {
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) name_index: usize,
}

impl ClassPoolElemView {
    pub fn class_ref_type(&self) -> ReferenceTypeView {
        let name_str = self.backing_class.constant_pool[self.name_index].extract_string_from_utf8();
        let type_ = PTypeView::from_ptype(&parse_class_name(&name_str));
        type_.unwrap_ref_type().clone()
    }
}

#[derive(Debug)]
pub struct StringView<'l> {
    pub(crate) class_view: &'l ClassBackedView,
    pub(crate) string_index: usize,
}

impl StringView<'_> {
    pub fn string(&self) -> String {
        self.class_view.backing_class.constant_pool[self.string_index].extract_string_from_utf8()
    }
}

#[derive(Debug)]
pub struct FieldrefView<'cl> {
    pub(crate) class_view: &'cl ClassBackedView,
    pub(crate) i: usize,
}

impl FieldrefView<'_> {
    fn field_ref(&self) -> &Fieldref {
        match &self.class_view.backing_class.constant_pool[self.i].kind {
            ConstantKind::Fieldref(fr) => &fr,
            _ => panic!(),
        }
    }
    pub fn class(&self) -> String {
        let class_index = self.field_ref().class_index;
        let name_index = self.class_view.backing_class.extract_class_from_constant_pool(class_index).name_index as usize;
        self.class_view.backing_class.constant_pool[name_index].extract_string_from_utf8()
    }
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView { class_view: self.class_view, i: self.field_ref().name_and_type_index as usize }
    }
}


#[derive(Debug)]
pub struct MethodrefView<'cl> {
    pub(crate) class_view: &'cl ClassBackedView,
    pub(crate) i: usize,
}

impl MethodrefView<'_> {
    fn get_raw(&self) -> &Methodref {
        match &self.class_view.backing_class.constant_pool[self.i].kind {
            ConstantKind::Methodref(mf) => &mf,
            c => {
                dbg!(c);
                panic!()
            }
        }
    }

    pub fn class(&self) -> ReferenceTypeView {
        let class_index = self.get_raw().class_index;
        PTypeView::from_ptype(&PType::Ref(self.class_view.backing_class.extract_class_from_constant_pool_name(class_index))).unwrap_ref_type().clone()
    }
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView {
            class_view: self.class_view,
            i: self.get_raw().name_and_type_index as usize,
        }
    }
}

#[derive(Debug)]
pub struct InterfaceMethodrefView<'cl> {
    pub(crate) class_view: &'cl ClassBackedView,
    pub(crate) i: usize,
}

impl InterfaceMethodrefView<'_> {
    fn interface_method_ref(&self) -> &InterfaceMethodref {
        match &self.class_view.backing_class.constant_pool[self.i].kind {
            ConstantKind::InterfaceMethodref(imr) => {
                &imr
            }
            _ => panic!(),
        }
    }
    pub fn class(&self) -> ReferenceTypeView {
        PTypeView::from_ptype(&PType::Ref(self.class_view.backing_class.extract_class_from_constant_pool_name(self.interface_method_ref().class_index))).unwrap_ref_type().clone()
    }
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView { class_view: self.class_view, i: self.interface_method_ref().nt_index as usize }
    }
}

#[derive(Debug)]
pub struct NameAndTypeView<'cl> {
    pub(crate) class_view: &'cl ClassBackedView,
    pub(crate) i: usize,
}

impl NameAndTypeView<'_> {
    fn name_and_type(&self) -> &NameAndType {
        match &self.class_view.backing_class.constant_pool[self.i as usize].kind {
            ConstantKind::NameAndType(nt) => {
                &nt
            }
            _ => panic!()
        }
    }

    pub fn name(&self) -> String {
        self.class_view.backing_class.constant_pool[self.name_and_type().name_index as usize].extract_string_from_utf8()
    }
    pub fn desc_str(&self) -> String {
        self.class_view.backing_class.constant_pool[self.name_and_type().descriptor_index as usize].extract_string_from_utf8()
    }
    pub fn desc_method(&self) -> MethodDescriptor {
        //todo this is incorrect, name and types aren't always method descirpotrs
        let desc_str = self.class_view.backing_class.constant_pool[self.name_and_type().descriptor_index as usize].extract_string_from_utf8();
        parse_method_descriptor(desc_str.as_str()).unwrap()//in future parse
    }
}

#[derive(Debug, Clone)]
pub struct MethodHandleView<'l> {
    pub(crate) class_view: &'l ClassBackedView,
    pub i: usize,
}

impl MethodHandleView<'_> {
    fn get_raw(&self) -> &MethodHandle {
        match &self.class_view.backing_class.constant_pool[self.i].kind {
            ConstantKind::MethodHandle(mh) => {
                mh
            }
            _ => panic!(),
        }
    }

    pub fn get_reference_kind(&self) -> ReferenceKind {
        self.get_raw().reference_kind.clone()
    }

    pub fn get_reference_data(&self) -> ReferenceInvokeKind {
        match self.get_raw().reference_kind {
            ReferenceKind::GetField => unimplemented!(),
            ReferenceKind::GetStatic => unimplemented!(),
            ReferenceKind::PutField => unimplemented!(),
            ReferenceKind::PutStatic => unimplemented!(),
            ReferenceKind::InvokeVirtual => unimplemented!(),
            ReferenceKind::InvokeStatic => {
                assert!(self.class_view.backing_class.major_version >= 52);
                //if the class file
                // version number is 52.0 or above, the constant_pool entry at that
                // index must be either a CONSTANT_Methodref_info structure or a
                // CONSTANT_InterfaceMethodref_info structure (§4.4.2) representing a
                // class's or interface's method for which a method handle is to be created.
                let reference_idx = self.get_raw().reference_index as usize;
                let invoke_static = match &self.class_view.backing_class.constant_pool[reference_idx].kind {
                    ConstantKind::Methodref(_) => InvokeStatic::Method(MethodrefView { class_view: self.class_view, i: reference_idx }),
                    ConstantKind::InterfaceMethodref(_) => InvokeStatic::Interface(InterfaceMethodrefView { class_view: self.class_view, i: reference_idx }),
                    ck => {
                        dbg!(ck);
                        panic!()
                    }
                };
                ReferenceInvokeKind::InvokeStatic(invoke_static)
            }
            ReferenceKind::InvokeSpecial => {
                assert!(self.class_view.backing_class.major_version >= 52);
                let reference_idx = self.get_raw().reference_index as usize;
                let invoke_special = match &self.class_view.backing_class.constant_pool[reference_idx].kind {
                    ConstantKind::Methodref(_) => InvokeSpecial::Method(MethodrefView { class_view: self.class_view, i: reference_idx }),
                    ConstantKind::InterfaceMethodref(_) => InvokeSpecial::Interface(InterfaceMethodrefView { class_view: self.class_view, i: reference_idx }),
                    ck => {
                        dbg!(ck);
                        panic!()
                    }
                };
                ReferenceInvokeKind::InvokeSpecial(invoke_special)
            }
            ReferenceKind::NewInvokeSpecial => unimplemented!(),
            ReferenceKind::InvokeInterface => unimplemented!(),
        }
    }
}

pub enum ReferenceInvokeKind<'cl> {
    InvokeStatic(InvokeStatic<'cl>),
    InvokeSpecial(InvokeSpecial<'cl>),
}

pub enum InvokeStatic<'cl> {
    Interface(InterfaceMethodrefView<'cl>),
    //todo should this be a thing
    Method(MethodrefView<'cl>),
}


pub enum InvokeSpecial<'cl> {
    Interface(InterfaceMethodrefView<'cl>),
    //todo should this be a thing
    Method(MethodrefView<'cl>),
}

#[derive(Debug)]
pub struct MethodTypeView<'cl> {
    pub(crate) class_view: &'cl ClassBackedView,
    pub i: usize,
}

impl MethodTypeView<'_> {
    pub fn get_descriptor(&self) -> String {
        let desc_i = self.method_type().descriptor_index;
        self.class_view.backing_class.constant_pool[desc_i as usize].extract_string_from_utf8()
    }

    fn method_type(&self) -> &MethodType {
        if let ConstantKind::MethodType(mt) = &self.class_view.backing_class.constant_pool[self.i].kind {
            mt
        } else { panic!() }
    }
}

#[derive(Debug)]
pub struct InvokeDynamicView<'cl> {
    pub(crate) class_view: &'cl ClassBackedView,
    pub(crate) bootstrap_method_attr_index: u16,
    pub(crate) name_and_type_index: CPIndex,
}

impl InvokeDynamicView<'_> {
    pub fn name_and_type(&self) -> NameAndTypeView {
        NameAndTypeView {
            class_view: self.class_view,
            i: self.name_and_type_index as usize,
        }
    }
    //todo this is wrong, there are multiple bootstrap methods.
    pub fn bootstrap_method(&self) -> BootstrapMethodView {
        BootstrapMethodView { backing: self.class_view.bootstrap_methods_attr().unwrap_or_else(|| todo!()), i: self.bootstrap_method_attr_index as usize }
    }
}

#[derive(Debug)]
pub enum ConstantInfoView<'cl> {
    Utf8(Utf8View),
    Integer(IntegerView),
    Float(FloatView),
    Long(LongView),
    Double(DoubleView),
    Class(ClassPoolElemView),
    String(StringView<'cl>),
    Fieldref(FieldrefView<'cl>),
    Methodref(MethodrefView<'cl>),
    InterfaceMethodref(InterfaceMethodrefView<'cl>),
    NameAndType(NameAndTypeView<'cl>),
    MethodHandle(MethodHandleView<'cl>),
    MethodType(MethodTypeView<'cl>),
    InvokeDynamic(InvokeDynamicView<'cl>),
    LiveObject(usize),
}

impl ConstantInfoView<'_> {
    pub fn unwrap_class(&self) -> &ClassPoolElemView {
        match self {
            ConstantInfoView::Class(c) => { c }
            _ => panic!(),
        }
    }
}