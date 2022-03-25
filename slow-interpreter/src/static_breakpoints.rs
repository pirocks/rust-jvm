use std::collections::{HashSet};

use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::loading::LoaderName;


pub struct StaticBreakpoints {
    points: HashSet<(CClassName, LoaderName, MethodName, CMethodDescriptor, ByteCodeOffset)>,
}

impl StaticBreakpoints {
    pub fn new() -> Self {
        let mut points = HashSet::new();
        points.insert((CClassName::string(), LoaderName::BootstrapLoader, MethodName::constructor_init(), CMethodDescriptor { arg_types: vec![CPDType::array(CPDType::CharType)], return_type: CPDType::VoidType }, ByteCodeOffset(7)));
        Self {
            points
        }
    }

    pub fn should_break(&self, class_name: CClassName, method_name: MethodName, method_desc: CMethodDescriptor, offset: ByteCodeOffset) -> bool {
        self.points.contains(&(class_name, LoaderName::BootstrapLoader, method_name, method_desc, offset))
    }
}
