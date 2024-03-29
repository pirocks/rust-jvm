use std::sync::Arc;

use wtf8::Wtf8Buf;

use rust_jvm_common::classfile::{AttributeType, BootstrapMethod, CPIndex, EnclosingMethod, InnerClass, InnerClasses, SourceFile};
use rust_jvm_common::compressed_classfile::class_names::{CClassName, CompressedClassName};
use rust_jvm_common::compressed_classfile::compressed_types::CPRefType;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::compressed_classfile::string_pool::{CCString, CompressedClassfileStringPool};
use rust_jvm_common::descriptor_parser::parse_class_name;

use crate::view::{ClassBackedView, ClassView};
use crate::view::constant_info_view::{ConstantInfoView, DoubleView, FloatView, IntegerView, LongView, MethodHandleView, MethodTypeView, StringView};

#[derive(Clone)]
pub struct BootstrapMethodIterator<'cl> {
    pub(crate) view: BootstrapMethodsView<'cl>,
    pub(crate) i: usize,
}

impl<'cl> Iterator for BootstrapMethodIterator<'cl> {
    type Item = BootstrapMethodView<'cl>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.view.get_bootstrap_methods_raw().len() {
            return None;
        }
        let res = BootstrapMethodView { backing: self.view.clone(), i: self.i };
        self.i += 1;
        res.into()
    }
}

#[derive(Clone)]
pub struct BootstrapMethodsView<'cl> {
    pub(crate) backing_class: &'cl ClassBackedView,
    pub(crate) attr_i: usize,
}

impl BootstrapMethodsView<'_> {
    fn get_bootstrap_methods_raw(&self) -> &Vec<BootstrapMethod> {
        match &self.backing_class.underlying_class.attributes[self.attr_i].attribute_type {
            AttributeType::BootstrapMethods(bm) => &bm.bootstrap_methods,
            _ => panic!(),
        }
    }

    pub fn bootstrap_methods(&self) -> BootstrapMethodIterator {
        BootstrapMethodIterator { view: self.clone(), i: 0 }
    }
}

#[derive(Clone)]
pub struct BootstrapMethodView<'cl> {
    pub(crate) backing: BootstrapMethodsView<'cl>,
    pub(crate) i: usize,
}

impl BootstrapMethodView<'_> {
    fn get_raw(&self) -> &BootstrapMethod {
        let bootstrap_methods = self.backing.get_bootstrap_methods_raw();
        &bootstrap_methods[self.i]
    }

    pub fn bootstrap_method_ref(&self) -> MethodHandleView {
        let i = self.get_raw().bootstrap_method_ref;
        let res = self.backing.backing_class.constant_pool_view(i as usize);
        match res {
            ConstantInfoView::MethodHandle(mh) => mh,
            _ => panic!(),
        }
    }

    pub fn bootstrap_args(&self) -> BootstrapArgViewIterator {
        BootstrapArgViewIterator {
            backing_class: self.backing.backing_class,
            bootstrap_args: self.get_raw().bootstrap_arguments.clone(),
            i: 0,
        }
    }
}

pub struct BootstrapArgViewIterator<'cl> {
    backing_class: &'cl ClassBackedView,
    bootstrap_args: Vec<CPIndex>,
    //myabe get rid of clone for this but its not really an issue
    i: usize,
}

impl<'cl> Iterator for BootstrapArgViewIterator<'cl> {
    type Item = BootstrapArgView<'cl>;

    fn next(&mut self) -> Option<Self::Item> {
        let arg = *self.bootstrap_args.get(self.i)?;
        let res = match self.backing_class.constant_pool_view(arg as usize) {
            ConstantInfoView::Integer(i) => BootstrapArgView::Integer(i),
            ConstantInfoView::MethodType(mt) => BootstrapArgView::MethodType(mt),
            ConstantInfoView::MethodHandle(mh) => BootstrapArgView::MethodHandle(mh),
            ConstantInfoView::String(s) => BootstrapArgView::String(s),
            _other => {
                unimplemented!()
            }
        }
            .into();
        self.i += 1;
        res
    }
}

//CONSTANT_String_info,  CONSTANT_Class_info,CONSTANT_Integer_info, CONSTANT_Long_info,
// CONSTANT_Float_info, CONSTANT_Double_info,CONSTANT_MethodHandle_info, or CONSTANT_MethodType_info
pub enum BootstrapArgView<'cl> {
    String(StringView<'cl>),
    Class(Arc<ClassBackedView>),
    Integer(IntegerView),
    Long(LongView),
    Float(FloatView),
    Double(DoubleView),
    MethodHandle(MethodHandleView<'cl>),
    MethodType(MethodTypeView<'cl>),
}

#[allow(dead_code)]
pub struct EnclosingMethodView<'l> {
    pub(crate) backing_class: &'l ClassBackedView,
    pub(crate) i: usize,
}

impl<'l> EnclosingMethodView<'l> {
    fn raw(&self) -> &EnclosingMethod {
        match &self.backing_class.underlying_class.attributes[self.i].attribute_type {
            AttributeType::EnclosingMethod(im) => im,
            _ => panic!(),
        }
    }

    pub fn class_name(&self, pool: &CompressedClassfileStringPool) -> CPRefType {
        self.backing_class.constant_pool_view(self.raw().class_index as usize).unwrap_class().class_ref_type(pool)
    }

    pub fn method_name(&self, pool: &CompressedClassfileStringPool) -> Option<MethodName> {
        if self.raw().method_index == 0 {
            return None;
        }
        Some(MethodName(pool.add_name(self.backing_class.underlying_class.name_and_type_extractor(self.raw().method_index).0, false)))
    }

    pub fn method_desc(&self, pool: &CompressedClassfileStringPool) -> Option<CCString> {
        if self.raw().method_index == 0 {
            return None;
        }
        Some(pool.add_name(self.backing_class.underlying_class.name_and_type_extractor(self.raw().method_index).1, false))
    }
}

pub struct InnerClassesView<'l> {
    pub(crate) backing_class: &'l ClassBackedView,
    pub(crate) i: usize,
}

impl InnerClassesView<'_> {
    fn raw(&self) -> &InnerClasses {
        match &self.backing_class.underlying_class.attributes[self.i].attribute_type {
            AttributeType::InnerClasses(ic) => ic,
            _ => panic!(),
        }
    }

    pub fn classes(&self) -> impl Iterator<Item=InnerClassView> {
        self.raw().classes.iter().map(move |class| InnerClassView { backing_class: &self.backing_class, class })
    }
}

pub struct InnerClassView<'l> {
    backing_class: &'l ClassBackedView,
    class: &'l InnerClass,
}

impl InnerClassView<'_> {
    pub fn inner_name(&self, class_pool: &CompressedClassfileStringPool) -> CClassName {
        let inner_name_index = self.class.inner_name_index as usize;
        let inner_class_name = self.backing_class.underlying_class.constant_pool[inner_name_index].extract_string_from_utf8();
        CompressedClassName(class_pool.add_name(inner_class_name.into_string().unwrap(), false))
    }

    pub fn outer_name(&self, class_pool: &CompressedClassfileStringPool) -> CClassName {
        let outer_class_name_index = self.backing_class.underlying_class.extract_class_from_constant_pool(self.class.outer_class_info_index).name_index;
        let outer_class_name = self.backing_class.underlying_class.constant_pool[outer_class_name_index as usize].extract_string_from_utf8();
        CompressedClassName(class_pool.add_name(outer_class_name.as_str().unwrap(), false))
    }

    pub fn complete_name(&self, class_pool: &CompressedClassfileStringPool) -> Option<CClassName> {
        let inner_name_index = self.class.inner_name_index as usize;
        if inner_name_index == 0 {
            return None;
        }
        //todo clean this up by using inner and outer name functions and making them return options
        if self.class.outer_class_info_index == 0 {
            return None
        }
        let outer_class_name_index = self.backing_class.underlying_class.extract_class_from_constant_pool(self.class.outer_class_info_index).name_index;
        let outer_class_name = self.backing_class.underlying_class.constant_pool[outer_class_name_index as usize].extract_string_from_utf8();
        let inner_class_name = self.backing_class.underlying_class.constant_pool[inner_name_index].extract_string_from_utf8();
        let class_name = format!("{}${}", outer_class_name.as_str().unwrap(), inner_class_name.as_str().unwrap());
        parse_class_name(class_name.as_str()).unwrap_class_type();
        Some(CompressedClassName(class_pool.add_name(class_name, false)))
    }
}

pub struct SourceFileView<'l> {
    pub(crate) backing_class: &'l ClassBackedView,
    pub(crate) i: usize,
}

impl SourceFileView<'_> {
    fn source_file_attr(&self) -> &SourceFile {
        match &self.backing_class.underlying_class.attributes[self.i].attribute_type {
            AttributeType::SourceFile(sf) => sf,
            _ => panic!(),
        }
    }

    pub fn file(&self) -> Wtf8Buf {
        let si = self.source_file_attr().sourcefile_index;
        self.backing_class.underlying_class.constant_pool[si as usize].extract_string_from_utf8()
    }
}