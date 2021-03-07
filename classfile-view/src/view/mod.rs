use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, RwLock};

use descriptor_parser::MethodDescriptor;
use rust_jvm_common::classfile::{ACC_ABSTRACT, ACC_FINAL, ACC_INTERFACE, ACC_NATIVE, ACC_PRIVATE, ACC_PROTECTED, ACC_PUBLIC, ACC_STATIC, ACC_SYNTHETIC, ACC_VARARGS, AttributeType, Classfile, ConstantKind};
use rust_jvm_common::classnames::{class_name, ClassName};

use crate::view::attribute_view::{BootstrapMethodsView, EnclosingMethodView, SourceFileView};
use crate::view::constant_info_view::{ClassPoolElemView, ConstantInfoView, DoubleView, FieldrefView, FloatView, IntegerView, InterfaceMethodrefView, InvokeDynamicView, LongView, MethodHandleView, MethodrefView, MethodTypeView, NameAndTypeView, StringView, Utf8View};
use crate::view::field_view::{FieldIterator, FieldView};
use crate::view::interface_view::InterfaceIterator;
use crate::view::method_view::{MethodIterator, MethodView};

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
    fn is_synthetic(&self) -> bool {
        self.access_flags() & ACC_SYNTHETIC > 0
    }
}


pub trait ClassView: HasAccessFlags {
    fn name(&self) -> ClassName;
    fn super_name(&self) -> Option<ClassName>;
    fn methods(&self) -> MethodIterator;
    fn method_view_i(&self, i: usize) -> MethodView;
    fn num_methods(&self) -> usize;
    fn constant_pool_size(&self) -> usize;
    fn constant_pool_view(&self, i: usize) -> ConstantInfoView;
    fn field(&self, i: usize) -> FieldView;
    fn fields(&self) -> FieldIterator;
    fn interfaces(&self) -> InterfaceIterator;
    fn num_fields(&self) -> usize;
    fn num_interfaces(&self) -> usize;
    fn bootstrap_methods_attr(&self) -> BootstrapMethodsView;
    fn sourcefile_attr(&self) -> Option<SourceFileView>;
    fn enclosing_method_view(&self) -> Option<EnclosingMethodView>;

    fn lookup_method(&self, name: &str, desc: &MethodDescriptor) -> Option<MethodView>;
    fn lookup_method_name(&self, name: &str) -> Vec<MethodView>;
}

#[derive(Debug)]
pub struct ClassBackedView {
    backing_class: Arc<Classfile>,
    method_index: RwLock<Option<Arc<MethodIndex>>>,
    descriptor_index: RwLock<Vec<Option<MethodDescriptor>>>,
}


impl ClassBackedView {
    pub fn from(c: Arc<Classfile>) -> ClassBackedView {
        ClassBackedView { backing_class: c.clone(), method_index: RwLock::new(None), descriptor_index: RwLock::new(vec![None; c.methods.len()]) }
    }

    fn backing_class(&self) -> Arc<Classfile> {
        self.backing_class.clone()
    }

    fn method_index(&self) -> Arc<MethodIndex> {
        let read_guard = self.method_index.read().unwrap();
        match read_guard.as_ref() {
            None => {
                let res = MethodIndex::new(self);
                std::mem::drop(read_guard);
                self.method_index.write().unwrap().replace(Arc::new(res));
                self.method_index()
            }
            Some(index) => { index.clone() }
        }
    }
}

impl ClassView for ClassBackedView {
    fn name(&self) -> ClassName {
        class_name(&self.backing_class)
    }
    fn super_name(&self) -> Option<ClassName> {
        self.backing_class.super_class_name()
    }
    fn methods(&self) -> MethodIterator {
        MethodIterator { class_view: self, i: 0 }
    }
    fn method_view_i(&self, i: usize) -> MethodView {
        MethodView { class_view: self, method_i: i }
    }
    fn num_methods(&self) -> usize {
        self.backing_class.methods.len()
    }
    fn constant_pool_size(&self) -> usize {
        self.backing_class.constant_pool.len()
    }
    fn constant_pool_view(&self, i: usize) -> ConstantInfoView {
        let backing_class = self.backing_class.clone();
        match &self.backing_class.constant_pool[i].kind {
            ConstantKind::Utf8(utf8) => ConstantInfoView::Utf8(Utf8View { str: utf8.string.clone() }),
            ConstantKind::Integer(i) => ConstantInfoView::Integer(IntegerView { int: i.bytes as i32 }),
            ConstantKind::Float(f) => ConstantInfoView::Float(FloatView {
                float: f32::from_bits(f.bytes)
            }),
            ConstantKind::Long(l) => ConstantInfoView::Long(LongView {
                long: ((l.high_bytes as u64) << 32 | l.low_bytes as u64) as i64
            }),
            ConstantKind::Double(d) => ConstantInfoView::Double(DoubleView {
                double: f64::from_bits((d.high_bytes as u64) << 32 | d.low_bytes as u64)
            }),
            ConstantKind::Class(c) => ConstantInfoView::Class(ClassPoolElemView { backing_class, name_index: c.name_index as usize }),
            ConstantKind::String(s) => ConstantInfoView::String(StringView { class_view: self, string_index: s.string_index as usize }),
            ConstantKind::Fieldref(_) => ConstantInfoView::Fieldref(FieldrefView { class_view: self, i }),
            ConstantKind::Methodref(_) => ConstantInfoView::Methodref(MethodrefView { class_view: self, i }),
            ConstantKind::InterfaceMethodref(_) => ConstantInfoView::InterfaceMethodref(InterfaceMethodrefView { class_view: self, i }),
            ConstantKind::NameAndType(_) => ConstantInfoView::NameAndType(NameAndTypeView { class_view: self, i }),
            ConstantKind::MethodHandle(_) => ConstantInfoView::MethodHandle(MethodHandleView { class_view: self, i }),
            ConstantKind::MethodType(_) => ConstantInfoView::MethodType(MethodTypeView { class_view: self, i }),
            ConstantKind::InvokeDynamic(id) => ConstantInfoView::InvokeDynamic(InvokeDynamicView {
                class_view: self,
                bootstrap_method_attr_index: id.bootstrap_method_attr_index,
                name_and_type_index: id.name_and_type_index,
            }),
            ConstantKind::InvalidConstant(_) => panic!(),
            ConstantKind::LiveObject(idx) => ConstantInfoView::LiveObject(*idx)
        }
    }
    fn field(&self, i: usize) -> FieldView {
        FieldView::from(self, i)
    }
    fn fields(&self) -> FieldIterator {
        FieldIterator { backing_class: &self, i: 0 }
    }
    fn interfaces(&self) -> InterfaceIterator {
        InterfaceIterator { view: &self, i: 0 }
    }
    fn num_fields(&self) -> usize {
        self.backing_class.fields.len()
    }
    fn num_interfaces(&self) -> usize {
        self.backing_class.interfaces.len()
    }
    fn bootstrap_methods_attr(&self) -> BootstrapMethodsView {
        let (i, _) = self.backing_class.attributes.iter().enumerate().find(|(i, x)| {
            match &x.attribute_type {
                AttributeType::BootstrapMethods(bm) => true,
                _ => false
            }
        }).unwrap();
        BootstrapMethodsView { backing_class: self, attr_i: i }
    }
    fn sourcefile_attr(&self) -> Option<SourceFileView> {
        let i = self.backing_class.attributes.iter().enumerate().flat_map(|(i, x)| {
            match &x.attribute_type {
                AttributeType::SourceFile(_) => Some(i),
                _ => None
            }
        }).next()?;
        Some(SourceFileView { backing_class: self, i })
    }
    fn enclosing_method_view(&self) -> Option<EnclosingMethodView> {
        self.backing_class.attributes.iter().enumerate().find(|(_i, attr)| {
            matches!(attr.attribute_type, AttributeType::EnclosingMethod(_))
        }).map(|(i, _)| { EnclosingMethodView { backing_class: ClassBackedView::from(self.backing_class.clone()), i } })
    }

    fn lookup_method(&self, name: &str, desc: &MethodDescriptor) -> Option<MethodView> {
        self.method_index().lookup(self, name, desc)
    }
    fn lookup_method_name(&self, name: &str) -> Vec<MethodView> {
        self.method_index().lookup_method_name(self, name)
    }
}

type MethodName = String;

#[derive(Debug)]
pub struct MethodIndex {
    index: HashMap<MethodName, HashMap<MethodDescriptor, usize>>,
}

impl MethodIndex {
    fn new(c: &ClassBackedView) -> Self {
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
    fn lookup<'cl>(&self, c: &'cl ClassBackedView, name: &str, desc: &MethodDescriptor) -> Option<MethodView<'cl>> {
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
    fn lookup_method_name<'cl>(&self, c: &'cl ClassBackedView, name: &str) -> Vec<MethodView<'cl>> {
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


impl HasAccessFlags for ClassBackedView {
    fn access_flags(&self) -> u16 {
        self.backing_class.access_flags
    }
}


pub enum PrimitiveView {
    Byte,
    Boolean,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Void,
}

impl HasAccessFlags for PrimitiveView {
    fn access_flags(&self) -> u16 {
        0x3F6 //value found experimentally w/ hotspot.
    }
}

impl ClassView for PrimitiveView {
    fn name(&self) -> ClassName {
        match self {
            PrimitiveView::Byte => ClassName::raw_byte(),
            PrimitiveView::Boolean => ClassName::raw_boolean(),
            PrimitiveView::Short => ClassName::raw_short(),
            PrimitiveView::Char => ClassName::raw_char(),
            PrimitiveView::Int => ClassName::raw_int(),
            PrimitiveView::Long => ClassName::raw_long(),
            PrimitiveView::Float => ClassName::raw_float(),
            PrimitiveView::Double => ClassName::raw_double(),
            PrimitiveView::Void => ClassName::raw_void()
        }
    }

    fn super_name(&self) -> Option<ClassName> {
        None
    }

    fn methods(&self) -> MethodIterator {
        todo!()
    }

    fn method_view_i(&self, i: usize) -> MethodView {
        panic!()
    }

    fn num_methods(&self) -> usize {
        0
    }

    fn constant_pool_size(&self) -> usize {
        0
    }

    fn constant_pool_view(&self, i: usize) -> ConstantInfoView {
        panic!()
    }

    fn field(&self, i: usize) -> FieldView {
        panic!()
    }

    fn fields(&self) -> FieldIterator {
        todo!()
    }

    fn interfaces(&self) -> InterfaceIterator {
        todo!()
    }

    fn num_fields(&self) -> usize {
        0
    }

    fn num_interfaces(&self) -> usize {
        0
    }

    fn bootstrap_methods_attr(&self) -> BootstrapMethodsView {
        todo!()
    }

    fn sourcefile_attr(&self) -> Option<SourceFileView> {
        None
    }

    fn enclosing_method_view(&self) -> Option<EnclosingMethodView> {
        None
    }

    fn lookup_method(&self, name: &str, desc: &MethodDescriptor) -> Option<MethodView> {
        None
    }

    fn lookup_method_name(&self, name: &str) -> Vec<MethodView> {
        vec![]
    }
}

pub struct ArrayView {
    sub: Arc<dyn ClassView>
}

impl HasAccessFlags for ArrayView {
    fn access_flags(&self) -> u16 {
        todo!()
    }
}

impl ClassView for ArrayView {
    fn name(&self) -> ClassName {
        todo!()
    }

    fn super_name(&self) -> Option<ClassName> {
        None
    }

    fn methods(&self) -> MethodIterator {
        todo!()
    }

    fn method_view_i(&self, i: usize) -> MethodView {
        todo!()
    }

    fn num_methods(&self) -> usize {
        todo!()
    }

    fn constant_pool_size(&self) -> usize {
        todo!()
    }

    fn constant_pool_view(&self, i: usize) -> ConstantInfoView {
        todo!()
    }

    fn field(&self, i: usize) -> FieldView {
        todo!()
    }

    fn fields(&self) -> FieldIterator {
        todo!()
    }

    fn interfaces(&self) -> InterfaceIterator {
        todo!()
    }

    fn num_fields(&self) -> usize {
        todo!()
    }

    fn num_interfaces(&self) -> usize {
        todo!()
    }


    fn bootstrap_methods_attr(&self) -> BootstrapMethodsView {
        todo!()
    }

    fn sourcefile_attr(&self) -> Option<SourceFileView> {
        todo!()
    }

    fn enclosing_method_view(&self) -> Option<EnclosingMethodView> {
        todo!()
    }

    fn lookup_method(&self, name: &str, desc: &MethodDescriptor) -> Option<MethodView> {
        todo!()
    }

    fn lookup_method_name(&self, name: &str) -> Vec<MethodView> {
        todo!()
    }
}


pub mod attribute_view;
pub mod interface_view;
pub mod field_view;
pub mod constant_info_view;
pub mod method_view;
pub mod ptype_view;