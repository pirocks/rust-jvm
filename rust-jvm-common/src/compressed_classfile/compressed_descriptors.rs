use std::fmt::{Display, Formatter};
use itertools::Itertools;
use crate::compressed_classfile::compressed_types::{CompressedParsedDescriptorType, CPDType};
use crate::compressed_classfile::string_pool::CompressedClassfileStringPool;
use crate::descriptor_parser::{FieldDescriptor, MethodDescriptor};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct CompressedMethodDescriptor {
    pub arg_types: Vec<CompressedParsedDescriptorType>,
    pub return_type: CompressedParsedDescriptorType,
}

impl CompressedMethodDescriptor {
    pub fn empty_args(return_type: CPDType) -> Self {
        Self { arg_types: vec![], return_type }
    }
    pub fn void_return(arg_types: Vec<CPDType>) -> Self {
        Self { arg_types, return_type: CPDType::VoidType }
    }
    pub fn from_legacy(md: MethodDescriptor, pool: &CompressedClassfileStringPool) -> Self {
        let MethodDescriptor { parameter_types, return_type } = md;
        Self {
            arg_types: parameter_types.into_iter().map(|ptype| CPDType::from_ptype(&ptype, pool)).collect_vec(),
            return_type: CPDType::from_ptype(&return_type, pool),
        }
    }

    pub fn jvm_representation(&self, string_pool: &CompressedClassfileStringPool) -> String {
        format!("({}){}", self.arg_types.iter().map(|arg| arg.jvm_representation(string_pool)).join(""), self.return_type.jvm_representation(string_pool))
    }

    pub fn mangled_representation<'gc>(&self, string_pool: &'gc CompressedClassfileStringPool) -> MangledDisplayWrapper<'gc,'_> {
        MangledDisplayWrapper {
            string_pool,
            desc: self
        }
    }

    pub fn java_source_representation(&self, _string_pool: &CompressedClassfileStringPool) -> String {
        todo!()
    }

    pub fn count_local_vars_needed(&self) -> u16 {
        self.arg_types.iter().map(|arg| {
            match arg {
                CompressedParsedDescriptorType::BooleanType => 1,
                CompressedParsedDescriptorType::ByteType => 1,
                CompressedParsedDescriptorType::ShortType => 1,
                CompressedParsedDescriptorType::CharType => 1,
                CompressedParsedDescriptorType::IntType => 1,
                CompressedParsedDescriptorType::LongType => 2,
                CompressedParsedDescriptorType::FloatType => 1,
                CompressedParsedDescriptorType::DoubleType => 2,
                CompressedParsedDescriptorType::VoidType => panic!(),
                CompressedParsedDescriptorType::Class(_) => 1,
                CompressedParsedDescriptorType::Array { .. } => 1,
            }
        }).sum()
    }
}

pub struct MangledDisplayWrapper<'gc, 'l> {
    string_pool: &'gc CompressedClassfileStringPool,
    desc: &'l CompressedMethodDescriptor
}

impl Display for MangledDisplayWrapper<'_,'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}", mangling_escape("("))?;
        for (i,arg) in self.desc.arg_types.iter().enumerate() {
            //todo finish this
            write!(f, "{}", arg.mangled_representation(self.string_pool))?;
            if i != self.desc.arg_types.len() - 1{
                write!(f, "{}",mangling_escape(";"))?;
            }
        }
        // write!(f, "{}", mangling_escape(")"))?;
        Ok(())
    }
}

pub fn mangling_escape(s: impl AsRef<str>) -> String {
    //todo need to handle unicode but shouldn't be an issue for now.
    s.as_ref().replace("_", "_1").replace(";", "_2").replace("[", "_3").replace("(", "").replace(")", "").replace("$", "_00024").replace("/", "_")
}

pub type CFieldDescriptor = CompressedFieldDescriptor;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CompressedFieldDescriptor(pub CompressedParsedDescriptorType);

impl CompressedFieldDescriptor {
    pub fn from_legacy(fd: FieldDescriptor, pool: &CompressedClassfileStringPool) -> Self {
        let FieldDescriptor { field_type } = fd;
        Self(CPDType::from_ptype(&field_type, pool))
    }
}
