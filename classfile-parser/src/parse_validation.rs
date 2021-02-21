use std::collections::HashMap;
use std::ops::Range;

use rust_jvm_common::classfile::{ACC_ABSTRACT, ACC_ANNOTATION, ACC_ENUM, ACC_FINAL, ACC_INTERFACE, ACC_PRIVATE, ACC_PROTECTED, ACC_PUBLIC, ACC_SUPER, ACC_VOLATILE, Annotation, AnnotationDefault, AnnotationValue, ArrayValue, AttributeInfo, AttributeType, BootstrapMethod, BootstrapMethods, Class, Classfile, ClassInfoIndex, Code, ConstantInfo, ConstantKind, ElementValue, ElementValuePair, EnclosingMethod, EnumConstValue, Exceptions, FieldInfo, Fieldref, InterfaceMethodref, InvokeDynamic, LocalVariableTableEntry, LocalVariableTypeTableEntry, LocalVarTargetTableEntry, MethodInfo, MethodParameter, MethodParameters, Methodref, NameAndType, ReferenceKind, String_, TargetInfo, TypeAnnotation, TypePath, TypePathEntry, Utf8};
use sketch_jvm_version_of_utf8::ValidationError;

use crate::EXPECTED_CLASSFILE_MAGIC;
use crate::parse_validation::ClassfileError::{BadConstantPoolEntry, ExpectedClassEntry, ExpectedDoubleCPEntry, ExpectedFloatCPEntry, ExpectedIntegerCPEntry, ExpectedLongCPEntry, ExpectedNameAndType, ExpectedUtf8CPEntry, TooManyOfSameAttribute};

pub const MAX_ARRAY_DIMENSIONS: usize = 255;

pub struct ValidatorSettings {
    allowable_major_versions: Range<u16>,
    allowable_minor_versions: HashMap<u16, Range<u16>>,
}

pub enum ClassfileError {
    WrongMagic,
    BadMajorVersion,
    BadMinorVersion,
    BadConstantPoolEntry,
    ExpectedUtf8CPEntry,
    ExpectedIntegerCPEntry,
    ExpectedLongCPEntry,
    ExpectedDoubleCPEntry,
    ExpectedFloatCPEntry,
    BadNameInCP,
    TooManyArrayDimensionsInName,
    SomehowManagedToParseAnInMemoryOnlyFeatureOfClassfiles,
    InterfaceIsNotAbstract,
    AnnotationClassWhichIsNotAnInterface,
    FinalInterfacesAreNotAllowed,
    SuperInterfacesAreNotAllowed,
    EnumInterfacesAreNotAllowed,
    FinalAndAbstractAreIncompatible,
    PublicPrivateProtectedIncompatible,
    FinalAndVolatileIncompatible,
    ExpectedClassEntry,
    InvalidConstant,
    Java9FeatureNotSupported,
    ExpectedNameAndType,
    TooManyOfSameAttribute,
    AttributeOnWrongType,
    NativeOrAbstractCannotHaveCode,
    EmptyCode,
    MissingCodeAttribute,
    MissingExceptionsAttribute,
    BadPC,
    BadIndex,
    TypePathError,
    BadUTF8,
}

impl From<ValidationError> for ClassfileError {
    fn from(_: ValidationError) -> Self {
        Self::BadUTF8
    }
}

pub enum AttributeEnclosingType<'l> {
    Method(&'l MethodInfo),
    Code(&'l Code),
    Class(&'l Classfile),
    Field,
}


impl ValidatorSettings {
    pub fn validate_parsed(&self, c: &Classfile) -> Result<(), ClassfileError> {
        if c.magic == EXPECTED_CLASSFILE_MAGIC {
            return Result::Err(ClassfileError::WrongMagic);
        }
        if self.allowable_major_versions.contains(&c.major_version) {
            //if this unwrap fails that means that the validator settings are wringly created
            let allowable_minors = self.allowable_minor_versions.get(&c.major_version).unwrap();
            if !allowable_minors.contains(&c.minor_version) {
                return Result::Err(ClassfileError::BadMinorVersion);
            }
        } else {
            return Result::Err(ClassfileError::BadMajorVersion);
        }
        let mut skip_next = true;
        for ci in &c.constant_pool {
            if skip_next {
                skip_next = false;
            } else {
                self.validate_constant_info(ci, c, &mut skip_next)?;
            }
        }

        if c.access_flags & ACC_INTERFACE > 0 {
            //from the spec:
            // If the ACC_INTERFACE flag is set, the ACC_ABSTRACT flag must also be set, and
            // the ACC_FINAL , ACC_SUPER , and ACC_ENUM flags set must not be set.
            if c.access_flags & ACC_ABSTRACT == 0 {
                return Result::Err(ClassfileError::InterfaceIsNotAbstract);
            }
            if c.access_flags & ACC_FINAL > 0 {
                return Result::Err(ClassfileError::FinalInterfacesAreNotAllowed);
            }
            if c.access_flags & ACC_SUPER > 0 {
                return Result::Err(ClassfileError::SuperInterfacesAreNotAllowed);
            }
            if c.access_flags & ACC_ENUM > 0 {
                return Result::Err(ClassfileError::EnumInterfacesAreNotAllowed);
            }
        } else {
            //from spec:
            //If the ACC_INTERFACE flag is not set, any of the other flags in Table 4.1-A may
            // be set except ACC_ANNOTATION . However, such a class file must not have both
            // its ACC_FINAL and ACC_ABSTRACT flags set (JLS §8.1.1.2).
            if c.access_flags & ACC_ANNOTATION > 0 {
                return Result::Err(ClassfileError::AnnotationClassWhichIsNotAnInterface);
            }
            if (c.access_flags & ACC_FINAL > 0) && (c.access_flags & ACC_ABSTRACT > 0) {
                return Result::Err(ClassfileError::FinalAndAbstractAreIncompatible);
            }
        }
        self.index_check(c.this_class, c)?;
        self.index_check(c.super_class, c)?;
        for interface in &c.interfaces {
            self.is_class_check(*interface, c)?;
        }
        for f in &c.fields {
            self.validate_field_info(f, c)?;
        }
        for f in &c.methods {
            self.validate_method_info(f, c)?;
        }
        let mut attribute_validation_context = AttributeValidationContext::default();
        for attr in &c.attributes {
            self.validate_attribute(&mut attribute_validation_context, attr, c, &AttributeEnclosingType::Class(c))?;
        }
        Result::Ok(())
    }

    pub fn index_check(&self, cpi: u16, c: &Classfile) -> Result<(), ClassfileError> {
        if cpi as usize >= c.constant_pool.len() {
            return Result::Err(ClassfileError::BadConstantPoolEntry);
        }
        Result::Ok(())
    }

    pub fn is_utf8_check<'l>(&self, cpi: u16, c: &'l Classfile) -> Result<&'l Utf8, ClassfileError> {
        self.index_check(cpi, c)?;
        match &c.constant_pool[cpi as usize].kind {
            ConstantKind::Utf8(utf8) => Result::Ok(utf8),
            _ => Result::Err(ClassfileError::ExpectedUtf8CPEntry)
        }
    }

    pub fn is_class_check<'l>(&self, cpi: u16, c: &'l Classfile) -> Result<&'l Class, ClassfileError> {
        self.index_check(cpi, c)?;
        match &c.constant_pool[cpi as usize].kind {
            ConstantKind::Class(class) => Result::Ok(class),
            _ => Result::Err(ClassfileError::ExpectedClassEntry)
        }
    }

    pub fn is_name_and_type_check<'l>(&self, cpi: u16, c: &'l Classfile) -> Result<&'l NameAndType, ClassfileError> {
        self.index_check(cpi, c)?;
        match &c.constant_pool[cpi as usize].kind {
            ConstantKind::NameAndType(nt) => Result::Ok(nt),
            _ => Result::Err(ClassfileError::ExpectedNameAndType)
        }
    }

    pub fn validate_constant_info(&self, ci: &ConstantInfo, c: &Classfile, ignore_next: &mut bool) -> Result<(), ClassfileError> {
        match &ci.kind {
            ConstantKind::Utf8(_) => {
                //nothing to validate, if it was successfully converted to a rust string it is utf8.
            }
            ConstantKind::Integer(_) | ConstantKind::Float(_) => {
                //nothing to validate. Any bytes here are valid.
            }
            ConstantKind::Long(_) | ConstantKind::Double(_) => {
                *ignore_next = true;
                // I completely agree with the spec on this:
                //In retrospect, making 8-byte constants take two constant pool entries was a poor choice.
            }
            ConstantKind::Class(class_info) => self.validate_class_info_impl(&c, &class_info)?,
            ConstantKind::String(string) => self.validate_string(c, string)?,
            ConstantKind::Fieldref(fr) => self.validate_field_ref(c, fr)?,
            ConstantKind::Methodref(mr) => self.validate_method_ref(c, mr)?,
            ConstantKind::InterfaceMethodref(imr) => self.validate_interface_method_ref(c, imr)?,
            ConstantKind::NameAndType(nt) => self.validate_name_and_type(c, nt)?,
            ConstantKind::MethodHandle(mh) => {
                self.index_check(mh.reference_index, c)?;
                match mh.reference_kind {
                    ReferenceKind::GetField |
                    ReferenceKind::GetStatic |
                    ReferenceKind::PutField |
                    ReferenceKind::PutStatic => {
                        match &c.constant_pool[mh.reference_index as usize].kind {
                            ConstantKind::Fieldref(field_ref) => {
                                self.validate_field_ref(c, field_ref)?;
                            }
                            _ => return Err(ClassfileError::BadConstantPoolEntry)
                        }
                    }
                    ReferenceKind::InvokeVirtual |
                    ReferenceKind::NewInvokeSpecial => {
                        match &c.constant_pool[mh.reference_index as usize].kind {
                            ConstantKind::Methodref(method_ref) => {
                                self.validate_method_ref(c, method_ref)?;
                            }
                            _ => return Err(ClassfileError::BadConstantPoolEntry)
                        }
                    }
                    ReferenceKind::InvokeStatic |
                    ReferenceKind::InvokeSpecial => {
                        match &c.constant_pool[mh.reference_index as usize].kind {
                            ConstantKind::Methodref(method_ref) => {
                                self.validate_method_ref(c, method_ref)?;
                            }
                            ConstantKind::InterfaceMethodref(method_ref) => {
                                if c.major_version < 52 {
                                    return Err(BadConstantPoolEntry);
                                }
                                self.validate_interface_method_ref(c, method_ref)?;
                            }
                            _ => return Err(ClassfileError::BadConstantPoolEntry)
                        }
                    }
                    ReferenceKind::InvokeInterface => {
                        match &c.constant_pool[mh.reference_index as usize].kind {
                            ConstantKind::InterfaceMethodref(method_ref) => {
                                self.validate_interface_method_ref(c, method_ref)?;
                            }
                            _ => return Err(ClassfileError::BadConstantPoolEntry)
                        }
                    }
                }
            }
            ConstantKind::MethodType(mt) => { self.is_utf8_check(mt.descriptor_index, c)?; }
            ConstantKind::Dynamic(_) => {
                return Result::Err(ClassfileError::Java9FeatureNotSupported);
            }
            ConstantKind::InvokeDynamic(InvokeDynamic { bootstrap_method_attr_index, name_and_type_index }) => {
                self.index_check(*name_and_type_index, c)?;
                match &c.constant_pool[*name_and_type_index as usize].kind {
                    ConstantKind::NameAndType(nt) => {
                        self.validate_name_and_type(c, nt)?;
                    }
                    _ => return Err(ClassfileError::BadConstantPoolEntry)
                }
                let bootstrap_attribute = c.attributes.iter().find_map(|attr| {
                    match &attr.attribute_type {
                        AttributeType::BootstrapMethods(attr) => Some(attr),
                        _ => None
                    }
                }).ok_or(ClassfileError::BadConstantPoolEntry)?;
                if *bootstrap_method_attr_index as usize >= bootstrap_attribute.bootstrap_methods.len() {
                    return Err(BadConstantPoolEntry);
                }
            }
            ConstantKind::InvalidConstant(_) => {
                return Result::Err(ClassfileError::InvalidConstant);
            }
            ConstantKind::LiveObject(_) => {
                return Result::Err(ClassfileError::SomehowManagedToParseAnInMemoryOnlyFeatureOfClassfiles);
            }
        }
        *ignore_next = false;
        Result::Ok(())
    }

    pub fn validate_string(&self, c: &Classfile, string: &String_) -> Result<(), ClassfileError> {
        self.is_utf8_check(string.string_index, c)?;
        Result::Ok(())
    }

    pub fn validate_name_and_type(&self, c: &Classfile, nt: &NameAndType) -> Result<(), ClassfileError> {
        self.is_utf8_check(nt.name_index, c)?;
        // descriptor validation happens in a separate crate used by validator/class view
        self.is_utf8_check(nt.descriptor_index, c)?;
        Result::Ok(())
    }

    pub fn validate_interface_method_ref(&self, c: &Classfile, fr: &InterfaceMethodref) -> Result<(), ClassfileError> {
        //the spec says that some further validation should be performed to prevent imr calls
        // on <clinit>. This is currently the responsibility of the class verifier.
        self.is_class_check(fr.class_index, c)?;
        self.is_name_and_type_check(fr.nt_index, c)?;
        Result::Ok(())
    }

    pub fn validate_method_ref(&self, c: &Classfile, fr: &Methodref) -> Result<(), ClassfileError> {
        self.is_class_check(fr.class_index, c)?;
        self.is_name_and_type_check(fr.name_and_type_index, c)?;
        Result::Ok(())
    }

    pub fn validate_field_ref(&self, c: &Classfile, fr: &Fieldref) -> Result<(), ClassfileError> {
        self.is_class_check(fr.class_index, c)?;
        self.is_name_and_type_check(fr.name_and_type_index, c)?;
        Result::Ok(())
    }

    pub fn validate_class_info(&self, c: &Classfile, i: u16) -> Result<(), ClassfileError> {
        match &c.constant_pool[i as usize].kind {
            ConstantKind::Class(class_info) => {
                self.validate_class_info_impl(&c, &class_info)
            }
            _ => return Err(ExpectedClassEntry)
        }
    }

    fn validate_class_info_impl(&self, c: &Classfile, class_info: &Class) -> Result<(), ClassfileError> {
        self.index_check(class_info.name_index, c)?;
        //from the spec:
        //The value of the name_index item must be a valid index into the
        // constant_pool table. The constant_pool entry at that index must be a
        // CONSTANT_Utf8_info structure (§4.4.7) representing a valid binary class or
        // interface name encoded in internal form (§4.2.1).
        match &c.constant_pool[class_info.name_index as usize].kind {
            ConstantKind::Utf8(utf8) => {
                let name_string = &utf8.string;
                self.validate_class_name(&name_string)?;
            }
            _ => return Result::Err(ClassfileError::ExpectedUtf8CPEntry)
        }
        Result::Ok(())
    }

    pub fn validate_class_name(&self, name: &str) -> Result<(), ClassfileError> {
        //from the spec:
        //Method names are further constrained so that, with the exception of the special
        // method names <init> and <clinit> (§2.9), they must not contain the ASCII
        // characters < or > (that is, left angle bracket or right angle bracket).
        if name.contains('<') || name.contains('>') {
            if !(name == "<clinit>" || name == "<init>") {
                return Result::Err(ClassfileError::BadNameInCP);
            }
        }
        //from the spec:
        // An array type descriptor is valid only if it represents 255 or fewer dimensions.

        if name.chars().filter(|c| *c == '[').count() > MAX_ARRAY_DIMENSIONS {
            return Result::Err(ClassfileError::TooManyArrayDimensionsInName);
        }
        Result::Ok(())
    }

    pub fn validate_field_info(&self, f: &FieldInfo, c: &Classfile) -> Result<(), ClassfileError> {
        //from spec:
        //Fields of classes may set any of the flags in Table 4.5-A. However, each
        // field of a class may have at most one of its ACC_PUBLIC , ACC_PRIVATE , and
        // ACC_PROTECTED flags set (JLS §8.3.1), and must not have both its ACC_FINAL
        // and ACC_VOLATILE flags set (JLS §8.3.1.4).
        let public = f.access_flags & ACC_PUBLIC > 0;
        let private = f.access_flags & ACC_PRIVATE > 0;
        let protected = f.access_flags & ACC_PROTECTED > 0;
        let final_ = f.access_flags & ACC_FINAL > 0;
        let volatile = f.access_flags & ACC_VOLATILE > 0;
        if (public && private) || (private && protected) || (public && protected) || (!public && !private && !protected) {
            return Result::Err(ClassfileError::PublicPrivateProtectedIncompatible);
        }
        if final_ && volatile {
            return Result::Err(ClassfileError::FinalAndVolatileIncompatible);
        }
        self.is_utf8_check(f.descriptor_index, c)?;
        self.is_utf8_check(f.name_index, c)?;
        let mut attribute_validation_context = AttributeValidationContext::default();
        for attribute in &f.attributes {
            self.validate_attribute(&mut attribute_validation_context, attribute, c, &AttributeEnclosingType::Field)?;
        }
        Result::Ok(())
    }

    pub fn validate_method_info(&self, m: &MethodInfo, c: &Classfile) -> Result<(), ClassfileError> {
        //from spec:
        //Methods of classes may have any of the flags in Table 4.6-A set. However,
        // each method of a class may have at most one of its ACC_PUBLIC , ACC_PRIVATE ,
        // and ACC_PROTECTED flags set (JLS §8.4.3).
        let public = m.access_flags & ACC_PUBLIC > 0;
        let private = m.access_flags & ACC_PRIVATE > 0;
        let protected = m.access_flags & ACC_PROTECTED > 0;
        if (public && private) || (private && protected) || (public && protected) || (!public && !private && !protected) {
            return Result::Err(ClassfileError::PublicPrivateProtectedIncompatible);
        }
        //there are number of other constraints on method access_flags, these should be handled by classfile verifier.
        self.is_utf8_check(m.name_index, c)?;
        self.is_utf8_check(m.descriptor_index, c)?;
        let mut attribute_validation_context = AttributeValidationContext::default();
        for attr in &m.attributes {
            self.validate_attribute(&mut attribute_validation_context, attr, c, &AttributeEnclosingType::Method(m))?;
        }
        if !m.is_native() && !m.is_abstract() {
            if !attribute_validation_context.has_been_code {
                return Err(ClassfileError::MissingCodeAttribute);
            }
        }
        Result::Ok(())
    }


    pub fn validate_attribute(&self, attribute_validation_context: &mut AttributeValidationContext, a: &AttributeInfo, c: &Classfile, attr: &AttributeEnclosingType) -> Result<(), ClassfileError> {
        match &a.attribute_type {
            AttributeType::SourceFile(sourcefile) => {
                self.validate_utf8(c, sourcefile.sourcefile_index)?;
                if attribute_validation_context.has_been_source {
                    return Err(ClassfileError::TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_source = true;
                match attr {
                    AttributeEnclosingType::Class(_) => {}
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            AttributeType::SourceDebugExtension(_) => {
                if attribute_validation_context.has_been_source_debug_extension {
                    return Err(ClassfileError::TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_source_debug_extension = true;
                match attr {
                    AttributeEnclosingType::Class(_) => {}
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            AttributeType::LineNumberTable(lnt) => {
                match attr {
                    AttributeEnclosingType::Code(c) => {
                        for lnte in &lnt.line_number_table {
                            if lnte.start_pc as usize >= c.code_raw.len() {
                                return Err(ClassfileError::BadPC);
                            }
                        }
                    }
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            AttributeType::LocalVariableTable(lvt) => {
                match attr {
                    AttributeEnclosingType::Code(code) => {
                        for LocalVariableTableEntry { start_pc, length, name_index, descriptor_index, .. } in &lvt.local_variable_table {
                            ValidatorSettings::validate_pcs(code, start_pc, length)?;
                            self.validate_utf8(c, *name_index)?;
                            self.validate_utf8(c, *descriptor_index)?;
                        }
                    }
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            AttributeType::LocalVariableTypeTable(lvtt) => {
                match attr {
                    AttributeEnclosingType::Code(code) => {
                        for LocalVariableTypeTableEntry { start_pc, length, name_index, descriptor_index, .. } in &lvtt.type_table {
                            ValidatorSettings::validate_pcs(code, start_pc, length)?;
                            self.validate_utf8(c, *name_index)?;
                            self.validate_utf8(c, *descriptor_index)?;
                        }
                    }
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            AttributeType::Deprecated(_) => {
                match attr {
                    AttributeEnclosingType::Code(_) => return Err(ClassfileError::AttributeOnWrongType),
                    _ => {}
                }
            }
            AttributeType::InnerClasses(ic) => {
                for x in &ic.classes {
                    self.validate_is_class(c, x.inner_class_info_index)?;
                    self.validate_is_class(c, x.outer_class_info_index)?;
                }
                match attr {
                    AttributeEnclosingType::Class(_) => {}
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
                if attribute_validation_context.has_been_inner_class {
                    return Result::Err(ClassfileError::TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_inner_class = true;
            }
            AttributeType::EnclosingMethod(encm) => {
                self.validate_enclosing_method(attribute_validation_context, &c, attr, &encm)?;
            }
            AttributeType::BootstrapMethods(BootstrapMethods { bootstrap_methods }) => {
                match attr {
                    AttributeEnclosingType::Class(_) => {}
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
                for BootstrapMethod { bootstrap_method_ref, bootstrap_arguments } in bootstrap_methods {
                    self.index_check(*bootstrap_method_ref, c)?;
                    match c.constant_pool[*bootstrap_method_ref as usize].kind {
                        ConstantKind::MethodHandle(_) => {}
                        _ => return Err(ClassfileError::BadConstantPoolEntry)
                    }
                    for bootstrap_arg in bootstrap_arguments {
                        self.index_check(*bootstrap_arg, c)?;
                        match c.constant_pool[*bootstrap_arg as usize].kind {
                            ConstantKind::Integer(_) |
                            ConstantKind::Float(_) |
                            ConstantKind::Long(_) |
                            ConstantKind::Double(_) |
                            ConstantKind::Class(_) |
                            ConstantKind::String(_) |
                            ConstantKind::MethodHandle(_) |
                            ConstantKind::MethodType(_) => {}
                            _ => return Err(ClassfileError::BadConstantPoolEntry),
                        }
                    }
                }
                if attribute_validation_context.has_been_bootstrap_methods {
                    return Err(TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_bootstrap_methods = true;
            }
            AttributeType::NestHost(_) => {}
            AttributeType::NestMembers(_) => {}
            AttributeType::ConstantValue(_) => {
                ValidatorSettings::validate_constant_value(attribute_validation_context)?;
            }
            AttributeType::Code(code) => {
                self.validate_code(attribute_validation_context, &c, attr, &code)?;
            }
            AttributeType::Exceptions(exc) => {
                self.validate_exceptions(attribute_validation_context, &c, attr, exc)?;
            }
            AttributeType::RuntimeVisibleParameterAnnotations(annotations) => {
                match attr {
                    AttributeEnclosingType::Method(_) => {}
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
                for annotation in annotations.parameter_annotations.iter().flat_map(|param| param.iter()) {
                    self.validate_annotation(&c, annotation)?;
                }
                if attribute_validation_context.has_been_runtime_visible_parameter_annotations {
                    return Err(TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_runtime_visible_parameter_annotations = true;
            }
            AttributeType::RuntimeInvisibleParameterAnnotations(annotations) => {
                match attr {
                    AttributeEnclosingType::Method(_) => {}
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
                for annotation in annotations.parameter_annotations.iter().flat_map(|param| param.iter()) {
                    self.validate_annotation(&c, annotation)?;
                }
                if attribute_validation_context.has_been_runtime_invisible_parameter_annotations {
                    return Err(TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_runtime_invisible_parameter_annotations = true;
            }
            AttributeType::AnnotationDefault(AnnotationDefault { default_value }) => {
                match attr {
                    AttributeEnclosingType::Method(_) => {}
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
                if attribute_validation_context.has_been_annotation_default {
                    return Err(TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_annotation_default = true;
                self.validate_element_value(c, default_value)?;
            }
            AttributeType::MethodParameters(MethodParameters { parameters }) => {
                match attr {
                    AttributeEnclosingType::Method(_) => {}
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
                if attribute_validation_context.has_been_method_parameters {
                    return Err(TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_method_parameters = true;
                for MethodParameter { name_index, .. } in parameters {
                    self.validate_utf8(c, *name_index)?;
                }
            }
            AttributeType::Synthetic(_) => {
                ValidatorSettings::validate_synthetic(attr)?;
            }
            AttributeType::Signature(sig) => {
                let index = sig.signature_index;
                self.validate_utf8(&c, index)?;
            }
            AttributeType::RuntimeVisibleAnnotations(annotations) => {
                match attr {
                    AttributeEnclosingType::Code(_) => return Err(ClassfileError::AttributeOnWrongType),
                    _ => {}
                }
                for annotation in &annotations.annotations {
                    self.validate_annotation(&c, annotation)?;
                }
            }
            AttributeType::RuntimeInvisibleAnnotations(annotations) => {
                match attr {
                    AttributeEnclosingType::Code(_) => return Err(ClassfileError::AttributeOnWrongType),
                    _ => {}
                }
                for annotation in &annotations.annotations {
                    self.validate_annotation(&c, annotation)?;
                }
            }
            AttributeType::StackMapTable(_) => {
                //should validate the ptype pointers here, but since they have already been converted to ptypes...
            }
            AttributeType::RuntimeVisibleTypeAnnotations(annotations) => {
                if attribute_validation_context.has_been_runtime_visible_type_annotations {
                    return Err(TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_runtime_visible_type_annotations = true;
                for type_annotation in &annotations.annotations {
                    self.validate_type_annotation(c, attr, type_annotation)?;
                }
            }
            AttributeType::RuntimeInvisibleTypeAnnotations(annotations) => {
                if attribute_validation_context.has_been_runtime_invisible_type_annotations {
                    return Err(TooManyOfSameAttribute);
                }
                attribute_validation_context.has_been_runtime_invisible_type_annotations = true;
                for type_annotation in &annotations.annotations {
                    self.validate_type_annotation(c, attr, type_annotation)?;
                }
            }
            AttributeType::Unknown => {}
        }
        Result::Ok(())
    }

    fn validate_type_annotation(&self, c: &Classfile, attr: &AttributeEnclosingType, type_annotation: &TypeAnnotation) -> Result<(), ClassfileError> {
        let TypeAnnotation { target_type, target_path: TypePath { path }, type_index, element_value_pairs } = type_annotation;
        self.validate_utf8(c, *type_index)?;
        match target_type {
            TargetInfo::TypeParameterTarget { .. } => {}
            TargetInfo::SuperTypeTarget { supertype_index } => {
                if *supertype_index != 65535u16 {
                    if *supertype_index as usize >= c.interfaces.len() {
                        return Err(ClassfileError::BadIndex);
                    }
                }
            }
            TargetInfo::TypeParameterBoundTarget { .. } => {}
            TargetInfo::EmptyTarget => {}
            TargetInfo::FormalParameterTarget { .. } => {}
            TargetInfo::ThrowsTarget { throws_type_index } => {
                match attr {
                    AttributeEnclosingType::Method(method) => {
                        let exceptions = method.exception_attribute().ok_or(ClassfileError::MissingExceptionsAttribute)?;
                        if *throws_type_index as usize >= exceptions.exception_index_table.len() {
                            return Err(ClassfileError::BadIndex);
                        }
                    }
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            TargetInfo::LocalVarTarget { table } => {
                match attr {
                    AttributeEnclosingType::Code(code) => {
                        for LocalVarTargetTableEntry { start_pc, length, .. } in table {
                            ValidatorSettings::validate_pcs(code, start_pc, length)?;
                        }
                    }
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            TargetInfo::CatchTarget { exception_table_entry } => {
                match attr {
                    AttributeEnclosingType::Method(method) => {
                        let exceptions = method.exception_attribute().ok_or(ClassfileError::MissingExceptionsAttribute)?;
                        if *exception_table_entry as usize >= exceptions.exception_index_table.len() {
                            return Err(ClassfileError::BadIndex);
                        }
                    }
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            TargetInfo::OffsetTarget { .. } => {
                //this amounts to bytecode validation.
                // out of scope
            }
            TargetInfo::TypeArgumentTarget { .. } => {
                //this amounts to bytecode validation.
                // out of scope
            }
        }
        for TypePathEntry { type_path_kind, type_argument_index } in path {
            if *type_argument_index != 0 && *type_path_kind != 3 {
                return Err(ClassfileError::TypePathError);
            }
            if *type_path_kind > 3 {
                return Err(ClassfileError::TypePathError);
            }
        }
        for ElementValuePair { element_name_index, value } in element_value_pairs {
            self.validate_utf8(c, *element_name_index)?;
            self.validate_element_value(c, value)?;
        }
        Ok(())
    }

    fn validate_pcs(code: &&Code, start_pc: &u16, length: &u16) -> Result<(), ClassfileError> {
        if *start_pc as usize >= code.code_raw.len() {
            return Err(ClassfileError::BadPC);
        }
        if (*start_pc + *length) as usize > code.code_raw.len() {
            return Err(ClassfileError::BadPC);
        }
        Ok(())
    }

    fn validate_annotation(&self, c: &Classfile, annotation: &Annotation) -> Result<(), ClassfileError> {
        self.validate_utf8(c, annotation.type_index)?;
        for ElementValuePair { element_name_index, value } in &annotation.element_value_pairs {
            self.validate_utf8(c, *element_name_index)?;
            self.validate_element_value(c, value)?;
        }
        Ok(())
    }

    fn validate_element_value(&self, c: &Classfile, value: &ElementValue) -> Result<(), ClassfileError> {
        match value {
            ElementValue::Byte(index) => self.validate_integer(c, *index)?,
            ElementValue::Char(index) => self.validate_integer(c, *index)?,
            ElementValue::Double(index) => self.validate_double(c, *index)?,
            ElementValue::Float(index) => self.validate_float(c, *index)?,
            ElementValue::Int(index) => self.validate_integer(c, *index)?,
            ElementValue::Long(index) => self.validate_long(c, *index)?,
            ElementValue::Short(index) => self.validate_integer(c, *index)?,
            ElementValue::Boolean(index) => self.validate_integer(c, *index)?,
            ElementValue::String(index) => self.validate_utf8(c, *index)?,
            ElementValue::EnumType(EnumConstValue { type_name_index, const_name_index }) => {
                self.validate_utf8(c, *type_name_index)?;
                self.validate_utf8(c, *const_name_index)?;
            }
            ElementValue::Class(ClassInfoIndex { class_info_index }) => {
                self.validate_class_info(c, *class_info_index)?;
            }
            ElementValue::AnnotationType(AnnotationValue { annotation }) => self.validate_annotation(c, &annotation)?,
            ElementValue::ArrayType(ArrayValue { values }) => {
                for value in values {
                    self.validate_element_value(c, value)?;
                }
            }
        }
        Ok(())
    }

    fn validate_enclosing_method(&self, attribute_validation_context: &mut AttributeValidationContext, c: &Classfile, attr: &AttributeEnclosingType, encm: &EnclosingMethod) -> Result<(), ClassfileError> {
        self.validate_is_class(c, encm.class_index)?;
        self.index_check(encm.method_index, c)?;
        match &c.constant_pool[encm.method_index as usize].kind {
            ConstantKind::NameAndType(nt) => {
                self.validate_name_and_type(c, nt)?;
            }
            _ => return Err(ExpectedNameAndType)
        }
        match attr {
            AttributeEnclosingType::Class(_) => {}
            _ => return Err(ClassfileError::AttributeOnWrongType)
        }
        if attribute_validation_context.has_been_enclosing_method {
            return Err(ClassfileError::TooManyOfSameAttribute);
        }
        attribute_validation_context.has_been_enclosing_method = true;
        Ok(())
    }

    fn validate_synthetic(attr: &AttributeEnclosingType) -> Result<(), ClassfileError> {
        //doesn't say anything about only one
        match attr {
            AttributeEnclosingType::Class(_) |
            AttributeEnclosingType::Field |
            AttributeEnclosingType::Method(_) => {}
            _ => return Err(ClassfileError::AttributeOnWrongType)
        }
        Ok(())
    }

    fn validate_utf8(&self, c: &Classfile, index: u16) -> Result<(), ClassfileError> {
        self.index_check(index, c)?;
        match &c.constant_pool[index as usize].kind {
            ConstantKind::Utf8(_) => {}
            _ => return Err(ExpectedUtf8CPEntry)
        }
        Ok(())
    }

    fn validate_integer(&self, c: &Classfile, index: u16) -> Result<(), ClassfileError> {
        self.index_check(index, c)?;
        match &c.constant_pool[index as usize].kind {
            ConstantKind::Integer(_) => {}
            _ => return Err(ExpectedIntegerCPEntry)
        }
        Ok(())
    }

    fn validate_long(&self, c: &Classfile, index: u16) -> Result<(), ClassfileError> {
        self.index_check(index, c)?;
        match &c.constant_pool[index as usize].kind {
            ConstantKind::Long(_) => {}
            _ => return Err(ExpectedLongCPEntry)
        }
        Ok(())
    }

    fn validate_double(&self, c: &Classfile, index: u16) -> Result<(), ClassfileError> {
        self.index_check(index, c)?;
        match &c.constant_pool[index as usize].kind {
            ConstantKind::Integer(_) => {}
            _ => return Err(ExpectedDoubleCPEntry)
        }
        Ok(())
    }
    fn validate_float(&self, c: &Classfile, index: u16) -> Result<(), ClassfileError> {
        self.index_check(index, c)?;
        match &c.constant_pool[index as usize].kind {
            ConstantKind::Integer(_) => {}
            _ => return Err(ExpectedFloatCPEntry)
        }
        Ok(())
    }

    fn validate_exceptions(&self, attribute_validation_context: &mut AttributeValidationContext, c: &Classfile, attr: &AttributeEnclosingType, exc: &Exceptions) -> Result<(), ClassfileError> {
        match attr {
            AttributeEnclosingType::Method(_) => {}
            _ => return Result::Err(ClassfileError::AttributeOnWrongType)
        }
        for index in &exc.exception_index_table {
            self.validate_is_class(c, *index)?;
        }

        if attribute_validation_context.has_been_exceptions {
            return Result::Err(ClassfileError::TooManyOfSameAttribute);
        }
        attribute_validation_context.has_been_exceptions = true;
        Ok(())
    }

    fn validate_code(&self, attribute_validation_context: &mut AttributeValidationContext, c: &Classfile, attr: &AttributeEnclosingType, code: &Code) -> Result<(), ClassfileError> {
        match attr {
            AttributeEnclosingType::Method(m) => {
                if m.is_native() || m.is_abstract() {
                    return Result::Err(ClassfileError::NativeOrAbstractCannotHaveCode);
                }
            }
            _ => return Result::Err(ClassfileError::AttributeOnWrongType)
        }
        if attribute_validation_context.has_been_code {
            return Result::Err(ClassfileError::TooManyOfSameAttribute);
        }
        attribute_validation_context.has_been_code = true;
        let Code { attributes, code_raw, exception_table, .. } = code;
        if code_raw.len() == 0 {
            return Result::Err(ClassfileError::EmptyCode);
        }
        for exception in exception_table {
            self.validate_is_class(c, exception.catch_type)?;
            //everything else here is the verifiers problem
        }
        let mut new_attribute_validation_context = AttributeValidationContext::default();
        for attribute in attributes {
            self.validate_attribute(&mut new_attribute_validation_context, attribute, c, &AttributeEnclosingType::Code(code))?;
        }
        Ok(())
    }

    fn validate_constant_value(attribute_validation_context: &mut AttributeValidationContext) -> Result<(), ClassfileError> {
//so the spec says:
        //If the ACC_STATIC flag in the access_flags item of the field_info structure is set,
        // then the field represented by the field_info structure is assigned the value
        // represented  by  its  ConstantValue  attribute  as  part  of  the  initialization
        // of  the class or interface declaring the field (§5.5). This occurs prior to the invocation
        // of the class or interface initialization method of that class or interface (§2.9).
        // •  Otherwise, the Java Virtual Machine must silently ignore the attribute
        //so we ignore this other than at most one constraint.
        if attribute_validation_context.has_been_constant_value {
            return Result::Err(ClassfileError::TooManyOfSameAttribute);
        }
        attribute_validation_context.has_been_constant_value = true;
        Ok(())
    }

    fn validate_is_class(&self, c: &Classfile, i: u16) -> Result<(), ClassfileError> {
        if i == 0 {
            return Ok(());
        }
        self.validate_class_info(c, i)
    }
}


pub struct AttributeValidationContext {
    has_been_constant_value: bool,
    has_been_code: bool,
    has_been_exceptions: bool,
    has_been_enclosing_method: bool,
    has_been_inner_class: bool,
    has_been_source_debug_extension: bool,
    has_been_source: bool,
    has_been_runtime_visible_parameter_annotations: bool,
    has_been_runtime_invisible_parameter_annotations: bool,
    has_been_runtime_visible_type_annotations: bool,
    has_been_runtime_invisible_type_annotations: bool,
    has_been_annotation_default: bool,
    has_been_bootstrap_methods: bool,
    has_been_method_parameters: bool,
}

impl Default for AttributeValidationContext {
    fn default() -> Self {
        AttributeValidationContext {
            has_been_constant_value: false,
            has_been_code: false,
            has_been_exceptions: false,
            has_been_enclosing_method: false,
            has_been_inner_class: false,
            has_been_source_debug_extension: false,
            has_been_source: false,
            has_been_runtime_visible_parameter_annotations: false,
            has_been_runtime_invisible_parameter_annotations: false,
            has_been_runtime_visible_type_annotations: false,
            has_been_runtime_invisible_type_annotations: false,
            has_been_annotation_default: false,
            has_been_bootstrap_methods: false,
            has_been_method_parameters: false,
        }
    }
}