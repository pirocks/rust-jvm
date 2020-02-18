use std::hash::Hasher;
use crate::string_pool::{StringPoolEntry, StringPool};
use std::sync::Arc;
use std::mem::transmute;
use descriptor_parser::{parse_field_descriptor, FieldDescriptor, MethodDescriptor, parse_method_descriptor, DescriptorOwned};
use rust_jvm_common::unified_types::PType;
use rust_jvm_common::classfile::{ConstantKind, ACC_STATIC, ACC_FINAL, ACC_PUBLIC, ACC_PRIVATE, ACC_ABSTRACT, UninitializedVariableInfo, ACC_INTERFACE, ACC_NATIVE, ACC_PROTECTED, NestMembers, Exceptions, Code, RuntimeInvisibleParameterAnnotations, RuntimeVisibleParameterAnnotations, AnnotationDefault, MethodParameters, Synthetic, Deprecated, Signature, RuntimeVisibleAnnotations, RuntimeInvisibleAnnotations, LineNumberTable, LocalVariableTable, LocalVariableTypeTable, StackMapTable, RuntimeVisibleTypeAnnotations, RuntimeInvisibleTypeAnnotations};
use rust_jvm_common::classnames::ClassName;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SourceFile {
    //todo
    pub sourcefile: Arc<StringPoolEntry>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InnerClasses {
    //todo
    pub classes: Vec<InnerClass>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct EnclosingMethod {
    pub class_index: CPIndex,
    pub method_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SourceDebugExtension {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct BootstrapMethods {
    //todo
    pub bootstrap_methods: Vec<BootstrapMethod>
}

//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//pub struct Module {
//    //todo
//}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct NestHost {
    //todo
    pub host_class_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ConstantValue {
    //todo
    pub constant_value_index: u16
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ObjectVariableInfo {
    pub cpool_index: Option<u16>,
    pub class_name: String,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayVariableInfo {
    pub array_type: PType
}
//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//#[derive(Hash)]
//pub struct UninitializedVariableInfo {
//    pub offset: u16
//}
//
//impl Clone for UninitializedVariableInfo {
//    fn clone(&self) -> Self {
//        UninitializedVariableInfo { offset: self.offset }
//    }
//}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitializedThis,
    Object(ObjectVariableInfo),
    Uninitialized(UninitializedVariableInfo),
    Array(ArrayVariableInfo),

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum AttributeType {
    SourceFile(SourceFile),
    InnerClasses(InnerClasses),
    EnclosingMethod(EnclosingMethod),
    SourceDebugExtension(SourceDebugExtension),
    BootstrapMethods(BootstrapMethods),
    Module(Module),
    NestHost(NestHost),
    NestMembers(NestMembers),
    ConstantValue(ConstantValue),
    Code(Code),
    Exceptions(Exceptions),
    RuntimeVisibleParameterAnnotations(RuntimeVisibleParameterAnnotations),
    RuntimeInvisibleParameterAnnotations(RuntimeInvisibleParameterAnnotations),
    AnnotationDefault(AnnotationDefault),
    MethodParameters(MethodParameters),
    Synthetic(Synthetic),
    Deprecated(Deprecated),
    Signature(Signature),
    RuntimeVisibleAnnotations(RuntimeVisibleAnnotations),
    RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotations),
    LineNumberTable(LineNumberTable),
    LocalVariableTable(LocalVariableTable),
    LocalVariableTypeTable(LocalVariableTypeTable),
    StackMapTable(StackMapTable),
    RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotations),
    RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotations),
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct BootstrapMethod {
    pub bootstrap_method_ref: u16,
    pub bootstrap_arguments: Vec<BootstrapArg>,
}


type BootstrapArg = u16;


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InnerClass {
    pub inner_class_info_index: CPIndex,
    pub outer_class_info_index: CPIndex,
    pub inner_name_index: CPIndex,
    pub inner_class_access_flags: CPIndex,
}


pub type CPIndex = u16;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct EnumConstValue {
    pub type_name_index: CPIndex,
    pub const_name_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ClassInfoIndex {
    pub class_info_index: CPIndex
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct AnnotationValue {
    pub annotation: Annotation
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayValue {
    pub values: Vec<ElementValue>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum ElementValue {
    Byte(CPIndex),
    Char(CPIndex),
    Double(CPIndex),
    Float(CPIndex),
    Int(CPIndex),
    Long(CPIndex),
    Short(CPIndex),
    Boolean(CPIndex),
    String(CPIndex),
    EnumType(EnumConstValue),
    Class(ClassInfoIndex),
    AnnotationType(AnnotationValue),
    ArrayType(ArrayValue),
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ElementValuePair {
    pub element_name_index: CPIndex,
    pub value: ElementValue,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Annotation {
    pub type_index: u16,
    pub num_element_value_pairs: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub struct IInc {
    pub index: u8,
    pub const_: i8,
}


#[derive(Debug)]
#[derive(Eq)]
pub struct Utf8 {
    entry: Arc<StringPoolEntry>
}

impl Utf8 {
    pub fn new<'cl, 'l>(s: &'l String, pool: &'l mut StringPool) -> Self {
        Utf8 { entry: pool.get_or_add(s.clone()) }
    }
}


impl PartialEq for Utf8 {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entry, &other.entry)
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Integer {
    //unimplemented!()
    pub bytes: u32
}

#[derive(Debug)]
pub struct Float {
    pub val: f32
}

impl Eq for Float {}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Long {
    pub val: i64
}

#[derive(Debug)]
pub struct Double {
    pub val: f64
}

impl Eq for Double {}

impl PartialEq for Double {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Class {
    pub name: Arc<StringPoolEntry>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct String_ {
    pub str: Arc<StringPoolEntry>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Fieldref {
    pub class_name: ClassName,
    pub name_and_type: NameAndType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Methodref {
    pub class_name: ClassName,
    pub name_and_type_index: NameAndType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InterfaceMethodref {
    pub class: ClassName,
    pub nt: NameAndType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub struct NameAndType {
    pub name: Arc<StringPoolEntry>,
    pub field_type: DescriptorOwned,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodHandle {
    pub reference_kind: u8,
    pub reference_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodType {
    pub descriptor_index: CPIndex
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Dynamic {
    //todo
}
//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//pub struct InvokeDynamic<'l> {
////    pub bootstrap_method_attr_index: CPIndex,
////    pub name_and_type_index: CPIndex,
////todo not in java8
//
//}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Module {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Package {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InvalidConstant {}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum ConstantInfo {
    Utf8(Utf8),
    Integer(Integer),
    Float(Float),
    Long(Long),
    Double(Double),
    Class(Class),
    String(String_),
    Fieldref(Fieldref),
    Methodref(Methodref),
    InterfaceMethodref(InterfaceMethodref),
    NameAndType(NameAndType),
    MethodHandle(MethodHandle),
    MethodType(MethodType),
    Dynamic(Dynamic),
    //    InvokeDynamic(InvokeDynamic<'cl>),//todo not in java8
    Module(Module),
    Package(Package),
    InvalidConstant(InvalidConstant),
}

impl ConstantInfo {
    pub fn unwrap_class(&self) -> &Class {
        match self {
            ConstantInfo::Class(c) => c,
            _ => panic!()
        }
    }
}


fn from_stage_1_constant_pool(
    constant_pool_stage_1: &Vec<rust_jvm_common::classfile::ConstantInfo>,
    string_pool: &mut StringPool,
) -> Vec<ConstantInfo> {
    let mut res_pool = vec![];
    for (_, x) in constant_pool_stage_1.iter().enumerate() {
        res_pool.push(ConstantInfo::from_stage_1(x, constant_pool_stage_1, string_pool));
    };
    res_pool
}

impl From<Integer> for ConstantInfo {
    fn from(i: Integer) -> Self {
        ConstantInfo::Integer(i)
    }
}

impl From<Double> for ConstantInfo {
    fn from(d: Double) -> Self {
        ConstantInfo::Double(d)
    }
}

impl From<Float> for ConstantInfo {
    fn from(f: Float) -> Self {
        ConstantInfo::Float(f)
    }
}

impl From<Utf8> for ConstantInfo {
    fn from(u: Utf8) -> Self {
        ConstantInfo::Utf8(u)
    }
}

impl From<Long> for ConstantInfo {
    fn from(u: Long) -> Self {
        ConstantInfo::Long(u)
    }
}

impl From<Class> for ConstantInfo {
    fn from(c: Class) -> Self {
        ConstantInfo::Class(c)
    }
}

impl From<String_> for ConstantInfo {
    fn from(s: String_) -> Self {
        ConstantInfo::String(s)
    }
}

impl From<crate::classfile::Fieldref> for crate::classfile::ConstantInfo {
    fn from(f: crate::classfile::Fieldref) -> crate::classfile::ConstantInfo {
        crate::classfile::ConstantInfo::Fieldref(f)
    }
}

impl ConstantInfo {
    fn from_stage_1<'l>(
        stage_1: &'l rust_jvm_common::classfile::ConstantInfo,
        constant_pool_stage_1: &'l Vec<rust_jvm_common::classfile::ConstantInfo>,
        string_pool: &mut StringPool,
    ) -> Self {
        match &stage_1.kind {
            ConstantKind::Utf8(utf8) => {
                Utf8::new(&utf8.string, string_pool).into()
            }
            ConstantKind::Integer(i) => {
                Integer { bytes: i.bytes }.into()
            }
            ConstantKind::Float(f) => {
                Float { val: unsafe { transmute(f.bytes) } }.into()//todo this may/may not be correct
            }
            ConstantKind::Long(l) => {
                Long { val: (((l.high_bytes as u64) << 32) | (l.low_bytes as u64)) as i64 }.into()//todo is magic constant ok?
            }
            ConstantKind::Double(d) => {
                Double {
                    val: unsafe { transmute(((d.high_bytes as u64) << 32) | (d.low_bytes as u64)) }
                }.into()
            }
            ConstantKind::Class(c) => {
                let class_name = constant_pool_stage_1[c.name_index as usize].extract_string_from_utf8();
                let class_name_entry = string_pool.get_or_add(class_name);
                Class { name: class_name_entry }.into()
            }
            ConstantKind::String(s) => {
                let str = constant_pool_stage_1[s.string_index as usize].extract_string_from_utf8();
                let str_entry = string_pool.get_or_add(str);
                String_ { str: str_entry }.into()
            }
            ConstantKind::Fieldref(fr) => {
                let name_index = match &constant_pool_stage_1[fr.class_index as usize].kind {
                    ConstantKind::Class(c) => c.name_index,
                    _ => panic!()
                };
                let class_name = constant_pool_stage_1[name_index as usize].extract_string_from_utf8();
                let class_name_entry = string_pool.get_or_add(class_name);
                let name_and_type = match &constant_pool_stage_1[fr.name_and_type_index as usize].kind {
                    ConstantKind::NameAndType(nt) => {
                        let desc_str = (constant_pool_stage_1[nt.descriptor_index as usize]).extract_string_from_utf8();
                        let parsed_field = parse_field_descriptor(&desc_str).unwrap();

                        let class_name = constant_pool_stage_1[nt.name_index as usize].extract_string_from_utf8();
                        let class_name_entry = string_pool.get_or_add(class_name);

                        NameAndType { name: class_name_entry, field_type: DescriptorOwned::Field(parsed_field) }
                    }
                    _ => panic!(),
                };
                Fieldref { class_name: ClassName::SharedStr(class_name_entry), name_and_type }.into()
            }
            ConstantKind::Methodref(_) => { todo!() }
            ConstantKind::InterfaceMethodref(_) => { todo!() }
            ConstantKind::NameAndType(_) => { todo!() }
            ConstantKind::MethodHandle(_) => { todo!() }
            ConstantKind::MethodType(_) => { todo!() }
            ConstantKind::Dynamic(_) => { todo!() }
            ConstantKind::InvokeDynamic(_) => { todo!() }
            ConstantKind::Module(_) => { todo!() }
            ConstantKind::Package(_) => { todo!() }
            ConstantKind::InvalidConstant(_) => { todo!() }
        }
    }
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub attribute_type: AttributeType,
}

impl From<&rust_jvm_common::classfile::AttributeType> for AttributeType {
    fn from(t: &rust_jvm_common::classfile::AttributeType) -> Self {
        match t {
            rust_jvm_common::classfile::AttributeType::SourceFile(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::InnerClasses(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::EnclosingMethod(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::SourceDebugExtension(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::BootstrapMethods(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::Module(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::NestHost(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::NestMembers(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::ConstantValue(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::Code(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::Exceptions(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::RuntimeVisibleParameterAnnotations(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::RuntimeInvisibleParameterAnnotations(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::AnnotationDefault(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::MethodParameters(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::Synthetic(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::Deprecated(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::Signature(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::RuntimeVisibleAnnotations(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::RuntimeInvisibleAnnotations(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::LineNumberTable(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::LocalVariableTable(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::LocalVariableTypeTable(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::StackMapTable(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::RuntimeVisibleTypeAnnotations(_) => unimplemented!(),
            rust_jvm_common::classfile::AttributeType::RuntimeInvisibleTypeAnnotations(_) => unimplemented!(),
        }
    }
}

impl From<&rust_jvm_common::classfile::AttributeInfo> for AttributeInfo {
    fn from(a: &rust_jvm_common::classfile::AttributeInfo) -> Self {
        AttributeInfo {
            attribute_name_index: a.attribute_name_index,
            attribute_length: a.attribute_length,
            attribute_type: (&a.attribute_type).into(),
        }
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name: Arc<StringPoolEntry>,
    pub descriptor: FieldDescriptor,
    pub attributes: Vec<AttributeInfo>,
}

impl FieldInfo {
    pub fn is_final(&self) -> bool {
        self.access_flags & ACC_FINAL > 0
    }
    pub fn is_static(&self) -> bool {
        self.access_flags & ACC_STATIC > 0
    }
    pub fn is_abstract(&self) -> bool {
        self.access_flags & ACC_ABSTRACT > 0
    }

    pub fn is_private(&self) -> bool {
        self.access_flags & ACC_PRIVATE > 0
    }

    pub fn is_protected(&self) -> bool {
        self.access_flags & ACC_PROTECTED > 0
    }

    pub fn is_public(&self) -> bool {
        self.access_flags & ACC_PUBLIC > 0
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodInfo {
    access_flags: u16,
    pub name: Arc<StringPoolEntry>,
    pub descriptor: MethodDescriptor,
    pub attributes: Vec<AttributeInfo>,
}

impl MethodInfo {
    pub fn is_final(&self) -> bool {
        self.access_flags & ACC_FINAL > 0
    }
    pub fn is_static(&self) -> bool {
        self.access_flags & ACC_STATIC > 0
    }
    pub fn is_native(&self) -> bool {
        self.access_flags & ACC_NATIVE > 0
    }
    pub fn is_abstract(&self) -> bool {
        self.access_flags & ACC_ABSTRACT > 0
    }

    pub fn is_private(&self) -> bool {
        self.access_flags & ACC_PRIVATE > 0
    }

    pub fn is_protected(&self) -> bool {
        self.access_flags & ACC_PROTECTED > 0
    }

    pub fn is_public(&self) -> bool {
        self.access_flags & ACC_PUBLIC > 0
    }

    pub fn code_attribute(&self) -> Option<&Code>{
        if self.is_abstract() || self.is_native() {
            return None;
        }

        for attr in self.attributes.iter() {
            match &attr.attribute_type {
                AttributeType::Code(code) => {
                    return Some(code);
                }
                _ => {}
            }
        }
        //todo dup
        panic!("Method has no code attribute, which is unusual given code is sorta the point of a method.")
    }
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub struct InvokeInterface {
    pub index: u16,
    pub count: u8,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub struct LookupSwitch {
    pub pairs: Vec<(i32, i32)>,
    pub default: i32,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub struct MultiNewArray {
    pub index: CPIndex,
    pub dims: u8,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Atype {
    TBoolean = 4,
    TChar = 5,
    TFloat = 6,
    TDouble = 7,
    TByte = 8,
    TShort = 9,
    TInt = 10,
    TLong = 11,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub struct TableSwitch {
    pub default: i32,
    pub low: i32,
    pub high: i32,
    pub offsets: Vec<i32>,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub struct Wide {}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Instruction {
    pub offset: usize,
    //maybe in future all instructions are of size 1 in phase 2
    pub instruction: InstructionInfo,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub enum InstructionInfo {
    aaload,
    aastore,
    aconst_null,
    aload(u8),
    aload_0,
    aload_1,
    aload_2,
    aload_3,
    anewarray(CPIndex),
    areturn,
    arraylength,
    astore(u8),
    astore_0,
    astore_1,
    astore_2,
    astore_3,
    athrow,
    baload,
    bastore,
    bipush(u8),
    caload,
    castore,
    checkcast(CPIndex),
    d2f,
    d2i,
    d2l,
    dadd,
    daload,
    dastore,
    dcmpg,
    dcmpl,
    dconst_0,
    dconst_1,
    ddiv,
    dload(u8),
    dload_0,
    dload_1,
    dload_2,
    dload_3,
    dmul,
    dneg,
    drem,
    dreturn,
    dstore(u8),
    dstore_0,
    dstore_1,
    dstore_2,
    dstore_3,
    dsub,
    dup,
    dup_x1,
    dup_x2,
    dup2,
    dup2_x1,
    dup2_x2,
    f2d,
    f2i,
    f2l,
    fadd,
    faload,
    fastore,
    fcmpg,
    fcmpl,
    fconst_0,
    fconst_1,
    fconst_2,
    fdiv,
    fload(u8),
    fload_0,
    fload_1,
    fload_2,
    fload_3,
    fmul,
    fneg,
    frem,
    freturn,
    fstore(u8),
    fstore_0,
    fstore_1,
    fstore_2,
    fstore_3,
    fsub,
    getfield(CPIndex),
    getstatic(CPIndex),
    goto_(i16),
    goto_w(i32),
    i2b,
    i2c,
    i2d,
    i2f,
    i2l,
    i2s,
    iadd,
    iaload,
    iand,
    iastore,
    iconst_m1,
    iconst_0,
    iconst_1,
    iconst_2,
    iconst_3,
    iconst_4,
    iconst_5,
    idiv,
    if_acmpeq(i16),
    if_acmpne(i16),
    if_icmpeq(i16),
    //todo dup
    if_icmpne(i16),
    //todo dup
    if_icmplt(i16),
    //todo dup
    if_icmpge(i16),
    //todo dup
    if_icmpgt(i16),
    //todo dup
    if_icmple(i16),
    //todo dup
    ifeq(i16),
    ifne(i16),
    iflt(i16),
    ifge(i16),
    ifgt(i16),
    ifle(i16),
    ifnonnull(i16),
    ifnull(i16),
    iinc(IInc),
    iload(u8),
    iload_0,
    iload_1,
    iload_2,
    iload_3,
    imul,
    ineg,
    instanceof(CPIndex),
    invokedynamic(CPIndex),
    invokeinterface(InvokeInterface),
    invokespecial(CPIndex),
    invokestatic(CPIndex),
    invokevirtual(CPIndex),
    ior,
    irem,
    ireturn,
    ishl,
    ishr,
    istore(u8),
    istore_0,
    istore_1,
    istore_2,
    istore_3,
    isub,
    iushr,
    ixor,
    jsr(i16),
    jsr_w(i32),
    l2d,
    l2f,
    l2i,
    ladd,
    laload,
    land,
    lastore,
    lcmp,
    lconst_0,
    lconst_1,
    ldc(u8),
    ldc_w(CPIndex),
    ldc2_w(CPIndex),
    ldiv,
    lload(u8),
    lload_0,
    lload_1,
    lload_2,
    lload_3,
    lmul,
    lneg,
    lookupswitch(LookupSwitch),
    lor,
    lrem,
    lreturn,
    lshl,
    lshr,
    lstore(u8),
    lstore_0,
    lstore_1,
    lstore_2,
    lstore_3,
    lsub,
    lushr,
    lxor,
    monitorenter,
    monitorexit,
    multianewarray(MultiNewArray),
    new(CPIndex),
    newarray(Atype),
    nop,
    pop,
    pop2,
    putfield(CPIndex),
    putstatic(CPIndex),
    ret(u8),
    return_,
    saload,
    sastore,
    sipush(u16),
    swap,
    tableswitch(TableSwitch),
    wide(Wide),
    EndOfCode,

}

#[derive(Debug)]
pub struct Classfile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool: Vec<ConstantInfo>,
    access_flags: u16,
    this_name: ClassName,
    super_name: Option<ClassName>,
    interfaces: Vec<Interface>,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
    attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct Interface {
    name: ClassName
}

impl Classfile {
    pub fn from_stage1(stage_1: &rust_jvm_common::classfile::Classfile, string_pool: &mut StringPool) -> Self {
        let this_name = ClassName::SharedStr(string_pool.get_or_add(stage_1.constant_pool[stage_1.this_class as usize].extract_string_from_utf8()));
        let super_name: Option<ClassName>;
        if stage_1.super_class == 0 {
            super_name = None;
        } else {
            super_name = ClassName::SharedStr(string_pool.get_or_add(stage_1.constant_pool[stage_1.super_class as usize].extract_string_from_utf8())).into();
        }
        let mut interfaces = vec![];
        for interface_name_i in &stage_1.interfaces {
            let interface_name = ClassName::SharedStr(string_pool.get_or_add(stage_1.extract_class_from_constant_pool_name(*interface_name_i)));
            interfaces.push(Interface { name: interface_name })
        }
        let mut fields = vec![];
        for field in &stage_1.fields {
            let res_field = Classfile::field_from_stage(&stage_1, string_pool, field);
            fields.push(res_field);
        }
        let mut methods = vec![];
        for method in &stage_1.methods {
            let res_method = Classfile::method_from_stage(stage_1, string_pool, method);
            methods.push(res_method);
        }
        Classfile {
            magic: stage_1.magic,
            minor_version: stage_1.minor_version,
            major_version: stage_1.major_version,
            constant_pool: from_stage_1_constant_pool(&stage_1.constant_pool, string_pool),//todo maybe minimize constant pool
            access_flags: stage_1.access_flags,
            this_name,
            super_name,
            interfaces,
            fields,
            methods,
            attributes: stage_1.attributes.iter().map(|x| x.into()).collect(),
        }
    }

    fn method_from_stage(
        stage_1: &rust_jvm_common::classfile::Classfile,
        string_pool: &mut StringPool,
        method: &rust_jvm_common::classfile::MethodInfo,
    ) -> MethodInfo {
        let name_str = stage_1.constant_pool[method.name_index as usize].extract_string_from_utf8();
        let desc_str = stage_1.constant_pool[method.descriptor_index as usize].extract_string_from_utf8();
        let descriptor = parse_method_descriptor(&desc_str).unwrap();
        let res_method = MethodInfo {
            access_flags: method.access_flags,
            name: string_pool.get_or_add(name_str),
            descriptor,
            attributes: method.attributes.iter().map(|x| x.into()).collect(),
        };
        res_method
    }

    fn field_from_stage(
        stage_1: &rust_jvm_common::classfile::Classfile,
        string_pool: &mut StringPool,
        field: &rust_jvm_common::classfile::FieldInfo,
    ) -> FieldInfo {
        let name_str = stage_1.constant_pool[field.name_index as usize].extract_string_from_utf8();
        let desc_str = stage_1.constant_pool[field.descriptor_index as usize].extract_string_from_utf8();
        let descriptor = parse_field_descriptor(&desc_str).unwrap();
        let res_field = FieldInfo {
            access_flags: field.access_flags,
            name: string_pool.get_or_add(name_str),
            descriptor,
            attributes: stage_1.attributes.iter().map(|x| x.into()).collect(),
        };
        res_field
    }

    pub fn is_static(&self) -> bool {
        self.access_flags & ACC_STATIC > 0
    }

    pub fn is_final(&self) -> bool {
        self.access_flags & ACC_FINAL > 0
    }

    pub fn is_public(&self) -> bool {
        self.access_flags & ACC_PUBLIC > 0
    }

    pub fn is_private(&self) -> bool {
        self.access_flags & ACC_PRIVATE > 0
    }

    pub fn is_abstract(&self) -> bool {
        self.access_flags & ACC_ABSTRACT > 0
    }

    pub fn is_interface(&self) -> bool {
        self.access_flags & ACC_INTERFACE > 0
    }

    pub fn super_name(&self) -> Option<ClassName> {
        self.super_name.clone()
    }

    pub fn name(&self) -> ClassName {
        self.this_name.clone()
    }

    pub fn get_constant_pool(&self) -> &Vec<ConstantInfo> {
        //todo this method will be phased out, since breaks encapsulation or something.
        &self.constant_pool
    }

    pub fn get_method_from_i(&self, i: usize) -> &MethodInfo {
        //todo phase out this method and all array index access
        &self.methods[i]
    }

    pub fn num_methods(&self) -> usize {
        self.methods.len()
    }

    pub fn methods(&self) -> &Vec<MethodInfo> {
        //todo phase out
        &self.methods
    }

    pub fn fields(&self) -> &Vec<FieldInfo> {
        //todo phase out
        &self.fields
    }

    pub fn old() -> Arc<rust_jvm_common::classfile::Classfile>{
        unimplemented!()
    }
}

impl std::cmp::PartialEq for Classfile {
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!()
//        self.magic == other.magic &&
//            self.minor_version == other.minor_version &&
//            self.major_version == other.major_version &&
//            self.constant_pool == other.constant_pool &&
//            self.access_flags == other.access_flags &&
//            self.this_class == other.this_class &&
//            self.super_class == other.super_class &&
//            self.interfaces == other.interfaces &&
//            self.fields == other.fields &&
//            self.methods == other.methods &&
//            self.attributes == other.attributes
    }
}

impl std::hash::Hash for Classfile {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!()
//        state.write_u32(self.magic);
//        state.write_u16(self.minor_version);
//        state.write_u16(self.major_version);
//        todo constant_pool
//        state.write_u16(self.access_flags);
//        state.write_u16(self.this_class);
//        state.write_u16(self.super_class);
//        for interface in &self.interfaces {
//            state.write_u16(*interface)
//        }
//        todo fields
        //todo methods
        //todo attributes
    }
}
