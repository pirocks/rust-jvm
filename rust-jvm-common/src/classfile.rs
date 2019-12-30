use std::hash::Hasher;
use crate::unified_types::UnifiedType;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SourceFile {
    //todo
    pub sourcefile_index:CPIndex
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
    //todo
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
pub struct Code {
    pub attributes: Vec<AttributeInfo>,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code_raw: Vec<u8>,
    pub code: Vec<Instruction>,
    pub exception_table: Vec<ExceptionTableElem>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ExceptionTableElem {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Exceptions {
    //todo
    pub exception_index_table: Vec<u16>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeVisibleParameterAnnotations {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeInvisibleParameterAnnotations {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct AnnotationDefault {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodParameters {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Synthetic {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Deprecated {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Signature {
    //todo
    pub signature_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeVisibleAnnotations {
    //todo
    pub annotations: Vec<Annotation>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeInvisibleAnnotations {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LineNumberTable {
    //todo
    pub line_number_table: Vec<LineNumberTableEntry>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LocalVariableTable {
    //todo
    pub local_variable_table: Vec<LocalVariableTableEntry>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LocalVariableTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LocalVariableTypeTable {
    //todo
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
    pub array_type: UnifiedType
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct UninitializedVariableInfo {
    pub offset: u16
}

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
pub struct SameFrame {
    pub offset_delta: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SameLocals1StackItemFrame {
    pub offset_delta: u16,
    pub stack: UnifiedType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SameLocals1StackItemFrameExtended {
    pub offset_delta: u16,
    pub stack: UnifiedType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ChopFrame {
    pub offset_delta: u16,
    pub k_frames_to_chop: u8,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SameFrameExtended {
    pub offset_delta: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct AppendFrame {
    pub offset_delta: u16,
    pub locals: Vec<UnifiedType>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct FullFrame {
    pub offset_delta: u16,
    pub number_of_locals: u16,
    pub locals: Vec<UnifiedType>,
    pub number_of_stack_items: u16,
    pub stack: Vec<UnifiedType>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum StackMapFrame {
    SameFrame(SameFrame),
    SameLocals1StackItemFrame(SameLocals1StackItemFrame),
    SameLocals1StackItemFrameExtended(SameLocals1StackItemFrameExtended),
    ChopFrame(ChopFrame),
    SameFrameExtended(SameFrameExtended),
    AppendFrame(AppendFrame),
    FullFrame(FullFrame),
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct StackMapTable {
    pub entries: Vec<StackMapFrame>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeVisibleTypeAnnotations {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeInvisibleTypeAnnotations {
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct NestMembers {
    pub classes: Vec<u16>
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


type CPIndex = u16;

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
pub struct IInc {
    pub index: u8,
    pub const_: i8,
}


#[derive(Debug)]
#[derive(Eq)]
pub struct Utf8 {
    pub length: u16,
    pub string: String,
}


impl PartialEq for Utf8 {
    fn eq(&self, other: &Self) -> bool {
        return self.length == other.length &&
            self.string == other.string;
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Integer {
    //unimplemented!()
    pub bytes: u32
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Float {
    pub bytes: u32
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Long {
    //unimplemented!()
    pub low_bytes: u32,
    pub high_bytes: u32,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Double {
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Class {
    //unimplemented!()
    pub name_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct String_ {
    //unimplemented!()
    pub string_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Fieldref {
    //unimplemented!()
    pub class_index: CPIndex,
    pub name_and_type_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Methodref {
    pub class_index: CPIndex,
    pub name_and_type_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InterfaceMethodref {
    pub class_index: CPIndex,
    pub nt_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct NameAndType {
    pub name_index: CPIndex,
    pub descriptor_index: CPIndex,
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

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InvokeDynamic {
    pub bootstrap_method_attr_index: CPIndex,
    pub name_and_type_index: CPIndex,
}

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
//#[derive(Copy, Clone)]
pub enum ConstantKind {
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
    InvokeDynamic(InvokeDynamic),
    Module(Module),
    Package(Package),
    InvalidConstant(InvalidConstant),
}


#[derive(Debug)]
#[derive(Eq)]
//#[derive(Copy, Clone)]
pub struct ConstantInfo {
    pub kind: ConstantKind,
}

impl PartialEq for ConstantInfo {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub attribute_type: AttributeType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: CPIndex,
    pub descriptor_index: CPIndex,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: CPIndex,
    pub descriptor_index: CPIndex,
    pub attributes: Vec<AttributeInfo>,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InvokeInterface {
    pub index: u16,
    pub count: u8,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LookupSwitch {
    pub pairs: Vec<(i32, i32)>,
    pub default: i32,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MultiNewArray {}


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
pub struct TableSwitch {
    pub default: i32,
    pub low: i32,
    pub high: i32,
    pub offsets: Vec<i32>,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Wide {}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Instruction {
    pub offset: usize,
    pub instruction: InstructionInfo,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[derive(Eq, PartialEq)]
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


//#[repr(u16)]
//pub enum ClassAccessFlags {
//TODO THIS NEEDS TO BE DIFFERENT FOR DIFFERNT TYPES
//maybe not but at very least is incomplete
pub const ACC_PUBLIC: u16 = 0x0001;
pub const ACC_PRIVATE: u16 = 0x0002;
pub const ACC_PROTECTED: u16 = 0x0004;
pub const ACC_STATIC: u16 = 0x0008;
pub const ACC_FINAL: u16 = 0x0010;
pub const ACC_SUPER: u16 = 0x0020;
pub const ACC_BRIDGE: u16 = 0x0040;
pub const ACC_VOLATILE: u16 = 0x0040;
pub const ACC_TRANSIENT: u16 = 0x0080;
pub const ACC_NATIVE: u16 = 0x0100;
pub const ACC_INTERFACE: u16 = 0x0200;
pub const ACC_ABSTRACT: u16 = 0x0400;
pub const ACC_STRICT: u16 = 0x0800;
pub const ACC_SYNTHETIC: u16 = 0x1000;
pub const ACC_ANNOTATION: u16 = 0x2000;
pub const ACC_ENUM: u16 = 0x4000;
pub const ACC_MODULE: u16 = 0x8000;
//}




#[derive(Debug)]
//#[derive(Eq)]
//#[derive(Copy, Clone)]
pub struct Classfile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantInfo>,
    pub access_flags: u16,
    pub this_class: CPIndex,
    pub super_class: CPIndex,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

impl std::cmp::PartialEq for Classfile {
    fn eq(&self, other: &Self) -> bool {
        self.magic == other.magic &&
            self.minor_version == other.minor_version &&
            self.major_version == other.major_version &&
            self.constant_pool == other.constant_pool &&
            self.access_flags == other.access_flags &&
            self.this_class == other.this_class &&
            self.super_class == other.super_class &&
            self.interfaces == other.interfaces &&
            self.fields == other.fields &&
            self.methods == other.methods &&
            self.attributes == other.attributes
    }
}

impl std::hash::Hash for Classfile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.magic);
        state.write_u16(self.minor_version);
        state.write_u16(self.major_version);
        //todo constant_pool
        state.write_u16(self.access_flags);
        state.write_u16(self.this_class);
        state.write_u16(self.super_class);
        //todo interfaces
        //todo fields
        //todo methods
        //todo attributes
    }
}
