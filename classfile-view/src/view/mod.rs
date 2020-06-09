use std::sync::{Arc, RwLock};
use crate::view::method_view::{MethodIterator, MethodView};
use rust_jvm_common::classfile::{ACC_FINAL, ACC_STATIC, ACC_NATIVE, ACC_PUBLIC, ACC_PRIVATE, ACC_PROTECTED, ACC_ABSTRACT, Classfile, ACC_INTERFACE, ConstantKind, AttributeType, ACC_VARARGS, ACC_SYNTHETIC};
use rust_jvm_common::classnames::{ClassName, class_name};
use crate::view::constant_info_view::{ConstantInfoView, ClassPoolElemView, NameAndTypeView, MethodrefView, StringView, IntegerView, FieldrefView, InterfaceMethodrefView, InvokeDynamicView, FloatView, LongView, DoubleView, MethodHandleView};
use crate::view::field_view::{FieldIterator, FieldView};
use crate::view::interface_view::InterfaceIterator;
use crate::view::attribute_view::{EnclosingMethodView, BootstrapMethodsView, SourceFileView};
use std::collections::HashMap;
use descriptor_parser::MethodDescriptor;
use std::iter::FromIterator;
use std::mem::transmute;


pub trait HasAccessFlags {
    fn access_flags(&self) -> u16;
    fn is_static(&self) -> bool {
        self.access_flags() & ACC_STATIC > 0
    }
    fn is_final(&self) -> bool {
        self.access_flags() & ACC_FINAL > 0
    }
    fn is_native(&self) -> bool {
        self.access_flags() & ACC_NATIVE > 0
    }
    fn is_varargs(&self) -> bool {
        self.access_flags() & ACC_VARARGS > 0
    }
    fn is_public(&self) -> bool {
        self.access_flags() & ACC_PUBLIC > 0
    }
    fn is_private(&self) -> bool {
        self.access_flags() & ACC_PRIVATE > 0
    }
    fn is_protected(&self) -> bool {
        self.access_flags() & ACC_PROTECTED > 0
    }
    fn is_abstract(&self) -> bool {
        self.access_flags() & ACC_ABSTRACT > 0
    }
    fn is_interface(&self) -> bool {
        self.access_flags() & ACC_INTERFACE > 0
    }
    fn is_synthetic(&self) -> bool{
        self.access_flags() & ACC_SYNTHETIC > 0
    }
}

#[derive(Debug)]
pub struct ClassView {
    backing_class: Arc<Classfile>,
    method_index: RwLock<Option<Arc<MethodIndex>>>,
    descriptor_index: RwLock<Vec<Option<MethodDescriptor>>>
}


impl ClassView {
    pub fn from(c: Arc<Classfile>) -> ClassView {
        ClassView { backing_class: c.clone(), method_index: RwLock::new(None), descriptor_index: RwLock::new(vec![None;c.methods.len()]) }
    }
    pub fn name(&self) -> ClassName {
        class_name(&self.backing_class)
    }
    pub fn super_name(&self) -> Option<ClassName> {
        self.backing_class.super_class_name()
    }
    pub fn methods(&self) -> MethodIterator {
        MethodIterator { class_view: self, i: 0 }
    }
    pub fn method_view_i(&self, i: usize) -> MethodView {
        MethodView { class_view: self, method_i: i }
    }
    pub fn num_methods(&self) -> usize {
        self.backing_class.methods.len()
    }
    pub fn constant_pool_view(&self, i: usize) -> ConstantInfoView {
        let backing_class = self.backing_class.clone();
        match &self.backing_class.constant_pool[i].kind {
            ConstantKind::Utf8(_) => unimplemented!(),
            ConstantKind::Integer(i) => ConstantInfoView::Integer(IntegerView { int: i.bytes as i32 }),//todo
            ConstantKind::Float(f) => ConstantInfoView::Float(FloatView {
                float: unsafe {
                    transmute(f.bytes)
                }
            }),//todo
            ConstantKind::Long(l) => ConstantInfoView::Long(LongView {
                long: unsafe {
                    transmute((l.high_bytes as u64) << 32 | l.low_bytes as u64)
                }
            }),//todo
            ConstantKind::Double(d) => ConstantInfoView::Double(DoubleView {
                double: unsafe {
                    transmute((d.high_bytes as u64) << 32 | d.low_bytes as u64)
                }
            }),//todo
            ConstantKind::Class(c) => ConstantInfoView::Class(ClassPoolElemView { backing_class, name_index: c.name_index as usize }),
            ConstantKind::String(s) => ConstantInfoView::String(StringView { class_view: self, string_index: s.string_index as usize }),//todo
            ConstantKind::Fieldref(_) => ConstantInfoView::Fieldref(FieldrefView { class_view:self , i }),
            ConstantKind::Methodref(_) => ConstantInfoView::Methodref(MethodrefView { class_view: self, i }),
            ConstantKind::InterfaceMethodref(_) => ConstantInfoView::InterfaceMethodref(InterfaceMethodrefView { class_view: self, i }),
            ConstantKind::NameAndType(_) => ConstantInfoView::NameAndType(NameAndTypeView { class_view: self, i }),
            ConstantKind::MethodHandle(_) => ConstantInfoView::MethodHandle(MethodHandleView { class_view: self, i }),
            ConstantKind::MethodType(_) => unimplemented!(),
            ConstantKind::Dynamic(_) => unimplemented!(),
            ConstantKind::InvokeDynamic(id) => ConstantInfoView::InvokeDynamic(InvokeDynamicView {
                class_view: self,
                bootstrap_method_attr_index: id.bootstrap_method_attr_index,
                name_and_type_index: id.name_and_type_index,
            }),
            ConstantKind::Module(_) => unimplemented!(),
            ConstantKind::Package(_) => unimplemented!(),
            ConstantKind::InvalidConstant(_) => unimplemented!(),
            ConstantKind::LiveObject(idx) => ConstantInfoView::LiveObject(*idx)
        }
    }
    pub fn field(&self, i: usize) -> FieldView {
        FieldView::from(self, i)
    }
    pub fn fields(&self) -> FieldIterator {
        FieldIterator { backing_class: &self, i: 0 }
    }
    pub fn interfaces(&self) -> InterfaceIterator {
        InterfaceIterator { view: &self, i: 0 }
    }
    pub fn num_fields(&self) -> usize {
        self.backing_class.fields.len()
    }
    pub fn num_interfaces(&self) -> usize {
        self.backing_class.interfaces.len()
    }
    pub fn backing_class(&self) -> Arc<Classfile> {
        self.backing_class.clone()
    }
    pub fn bootstrap_methods_attr(&self) -> BootstrapMethodsView {
        let (i, _) = self.backing_class.attributes.iter().enumerate().flat_map(|(i, x)| {
            match &x.attribute_type {
                AttributeType::BootstrapMethods(bm) => Some((i, bm)),
                _ => None
            }
        }).next().unwrap();
        BootstrapMethodsView { backing_class: self, attr_i: i }
    }
    pub fn sourcefile_attr(&self) -> SourceFileView {
        let i= self.backing_class.attributes.iter().enumerate().flat_map(|(i, x)| {
            match &x.attribute_type {
                AttributeType::SourceFile(_) => Some(i),
                _ => None
            }
        }).next().unwrap();
        SourceFileView { backing_class: self, i }
    }
    pub fn enclosing_method_view(&self) -> Option<EnclosingMethodView> {
        self.backing_class.attributes.iter().enumerate().find(|(_i, attr)| {
            match attr.attribute_type {
                AttributeType::EnclosingMethod(_) => true,
                _ => false,
            }
        }).map(|(i, _)| { EnclosingMethodView { backing_class: ClassView::from(self.backing_class.clone()), i } })
    }

    pub fn lookup_method(&self, name: &String, desc: &MethodDescriptor) -> Option<MethodView> {
        self.method_index().lookup(self,name,desc)
    }
    pub fn lookup_method_name(&self, name: &String) -> Vec<MethodView>{
        self.method_index().lookup_method_name(self,name)
    }

    fn method_index(&self) -> Arc<MethodIndex> {
        let read_guard = self.method_index.read().unwrap();
        match read_guard.as_ref() {
            None => {
                let res = MethodIndex::new(self);
                std::mem::drop(read_guard);
                self.method_index.write().unwrap().replace(Arc::new(res).into());
                self.method_index()
            }
            Some(index) => { index.clone() }
        }
    }
}

type MethodName = String;

#[derive(Debug)]
pub struct MethodIndex {
    index: HashMap<MethodName, HashMap<MethodDescriptor, usize>>,
}

impl MethodIndex {
    fn new(c: &ClassView) -> Self {
        let mut res = Self { index: HashMap::new() };
        for method_view in c.methods() {
            let name = method_view.name();
            let parsed_desc = method_view.desc();
            let method_i = method_view.method_i;
            match res.index.get_mut(&name) {
                None => {
                    let new_hashmap = HashMap::from_iter(vec![(parsed_desc, method_i)].into_iter());
                    res.index.insert(name, new_hashmap);
                }
                Some(method_descriptors) => {
                    method_descriptors.insert(parsed_desc, method_i);
                }
            }
        }
        res
    }
    fn lookup<'cl>(&self,c: &'cl ClassView, name: &String, desc: &MethodDescriptor) -> Option<MethodView<'cl>> {
        self.index.get(name)
            .and_then(|x| x.get(desc))
            .map(
                |method_i|
                    MethodView {
                        class_view: c,
                        method_i: *method_i,
                    }
            )
    }
    fn lookup_method_name<'cl>(&self, c: &'cl ClassView, name: &String) -> Vec<MethodView<'cl>> {
        self.index.get(name)
            .map(
                |methods|
                    methods.values().map(|method_i| {
                        MethodView {
                            class_view: c,
                            method_i: *method_i,
                        }
                    }).collect::<Vec<MethodView>>()
            ).unwrap_or(vec![])
    }
}


impl HasAccessFlags for ClassView {
    fn access_flags(&self) -> u16 {
        self.backing_class.access_flags
    }
}

pub mod attribute_view;
pub mod interface_view;
pub mod field_view;
pub mod constant_info_view;
pub mod method_view;
pub mod ptype_view;