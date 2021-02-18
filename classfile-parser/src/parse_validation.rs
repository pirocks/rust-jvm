use std::collections::HashMap;
use std::ops::Range;

use rust_jvm_common::classfile::{ACC_ABSTRACT, ACC_ANNOTATION, ACC_ENUM, ACC_FINAL, ACC_INTERFACE, ACC_PRIVATE, ACC_PROTECTED, ACC_PUBLIC, ACC_SUPER, ACC_SYNTHETIC, ACC_VOLATILE, AttributeInfo, AttributeType, Class, Classfile, Code, ConstantInfo, ConstantKind, EnclosingMethod, Exceptions, FieldInfo, Fieldref, InterfaceMethodref, LocalVariableTableEntry, MethodInfo, Methodref, NameAndType, String_, Utf8};

use crate::EXPECTED_CLASSFILE_MAGIC;
use crate::parse_validation::ClassfileError::{ExpectedNameAndType, ExpectedUtf8CPEntry};

pub const MAX_ARRAY_DIMENSIONS: usize = 255;

pub struct ValidatorSettings {
    allowable_major_versions: Range<u16>,
    allowable_minor_versions: HashMap<u16, Range<u16>>,
}

pub enum ClassfileError {
    WrongMagic,
    BadMajorVersion,
    BadMinorVersion,
    BadConstantPoolIndex,
    ExpectedUtf8CPEntry,
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
    BadPC,
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
            self.validate_attribute(&mut attribute_validation_context, attr, c, &AttributeEnclosingType::Class)?;
        }
        Result::Ok(())
    }

    pub fn index_check(&self, cpi: u16, c: &Classfile) -> Result<(), ClassfileError> {
        if cpi as usize >= c.constant_pool.len() {
            return Result::Err(ClassfileError::BadConstantPoolIndex);
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
                //todo so this isn't quite correct this is a modified utf8 which may not translate to rust utf-8
            }
            ConstantKind::Integer(_) | ConstantKind::Float(_) => {
                //nothing to validate. Any bytes here are valid.
            }
            ConstantKind::Long(_) | ConstantKind::Double(_) => {
                *ignore_next = true;
                // I completely agree with the spec on this:
                //In retrospect, making 8-byte constants take two constant pool entries was a poor choice.
            }
            ConstantKind::Class(class_info) => self.validate_class_info(&c, &class_info)?,
            ConstantKind::String(string) => self.validate_string(c, string)?,
            ConstantKind::Fieldref(fr) => self.validate_field_ref(c, fr)?,
            ConstantKind::Methodref(mr) => self.validate_method_ref(c, mr)?,
            ConstantKind::InterfaceMethodref(imr) => self.validate_interface_method_ref(c, imr)?,
            ConstantKind::NameAndType(nt) => self.validate_name_and_type(c, nt)?,
            ConstantKind::MethodHandle(_mh) => {
                //todo part of invoke_dynamic, which is a work in progress
            }
            ConstantKind::MethodType(mt) => { self.is_utf8_check(mt.descriptor_index, c)?; }
            ConstantKind::Dynamic(_) => {
                return Result::Err(ClassfileError::Java9FeatureNotSupported);
            }
            ConstantKind::InvokeDynamic(_) => {
                //todo part of invoke_dynamic, which is a work in progress
            }
            ConstantKind::Module(_) => {
                return Result::Err(ClassfileError::Java9FeatureNotSupported);
            }
            ConstantKind::Package(_) => {
                return Result::Err(ClassfileError::Java9FeatureNotSupported);
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

    pub fn validate_class_info(&self, c: &Classfile, class_info: &Class) -> Result<(), ClassfileError> {
        self.index_check(class_info.name_index, c)?;
        //from the spec:
        //The value of the name_index item must be a valid index into the
        // constant_pool table. The constant_pool entry at that index must be a
        // CONSTANT_Utf8_info structure (§4.4.7) representing a valid binary class or
        // interface name encoded in internal form (§4.2.1).
        match &c.constant_pool[class_info.name_index as usize].kind {
            ConstantKind::Utf8(utf8) => {
                //todo validate the utf8 before use
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
        //todo finish up attribute validation implementation
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
                        for lnte in lnt.line_number_table {
                            if lnte.start_pc >= c.code_raw.len() as u16 {
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
                        for LocalVariableTableEntry { start_pc, length, name_index, descriptor_index, .. } in lvt.local_variable_table {
                            if start_pc >= code.code_raw.len() as u16 {
                                return Err(ClassfileError::BadPC);
                            }
                            if start_pc + length > code.code_raw.len() as u16 {
                                return Err(ClassfileError::BadPC);
                            }
                            self.validate_utf8(c, name_index)?;
                            self.validate_utf8(c, descriptor_index)?;
                        }
                    }
                    _ => return Err(ClassfileError::AttributeOnWrongType)
                }
            }
            AttributeType::LocalVariableTypeTable(_) |
            AttributeType::Deprecated(_) => {
                //from spec:
                //Six attributes are not critical to correct interpretation of the class file by either
                // the Java Virtual Machine or the class libraries of the Java SE platform, but are
                // useful for tools:
                // • SourceFile
                // • SourceDebugExtension
                // • LineNumberTable
                // • LocalVariableTable
                // • LocalVariableTypeTable
                // • Deprecated
                // in other words we don't validate or use these for now. todo
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
            }
            AttributeType::EnclosingMethod(encm) => {
                self.validate_enclosing_method(attribute_validation_context, &c, attr, &encm)
            }
            AttributeType::BootstrapMethods(_) => {}
            AttributeType::Module(_) => {}
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
            AttributeType::RuntimeVisibleParameterAnnotations(_) => {}
            AttributeType::RuntimeInvisibleParameterAnnotations(_) => {}
            AttributeType::AnnotationDefault(_) => {}
            AttributeType::MethodParameters(_) => {}
            AttributeType::Synthetic(_) => {
                ValidatorSettings::validate_synthetic(attr)?;
            }
            AttributeType::Signature(sig) => {
                let index = sig.signature_index;
                self.validate_utf8(&c, index)?;
            }
            AttributeType::RuntimeVisibleAnnotations(_) => {}
            AttributeType::RuntimeInvisibleAnnotations(_) => {}
            AttributeType::StackMapTable(_) => {
                //todo should validate the ptype pointers here, but since they have already been converted to ptypes...
            }
            AttributeType::RuntimeVisibleTypeAnnotations(_) => {}
            AttributeType::RuntimeInvisibleTypeAnnotations(_) => {}
        }
        Result::Ok(())
    }

    fn validate_enclosing_method(&self, attribute_validation_context: &mut AttributeValidationContext, ,c: &Classfile, attr: &AttributeEnclosingType, encm: &EnclosingMethod) -> Result<(), ClassfileError> {
        self.validate_is_class(c, encm.class_index)?;
        self.index_check(encm.method_index, c)?;
        match &c.constant_pool[encm.method_index as usize].kind {
            ConstantKind::NameAndType(nt) => {
                self.validate_name_and_type(c, nt)?;
            }
            _ => Err(ExpectedNameAndType)
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
        self.index_check(i, c)?;
        match &c.constant_pool[i as usize].kind {
            ConstantKind::Class(class) => {
                self.validate_class_info(c, class)
            }
            _ => return Result::Err(ClassfileError::ExpectedClassEntry)
        }
    }
}


pub struct AttributeValidationContext {
    has_been_constant_value: bool,
    has_been_code: bool,
    //todo make sure this is checked
    has_been_exceptions: bool,
    has_been_enclosing_method: bool,
    has_been_inner_class: bool,
    has_been_source_debug_extension: bool,
    has_been_source: bool,
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
        }
    }
}