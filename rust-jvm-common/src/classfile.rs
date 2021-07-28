use std::hash::Hasher;
use std::io::Write;

use num_derive::FromPrimitive;
use wtf8::Wtf8Buf;

use sketch_jvm_version_of_utf8::Utf8OrWtf8;

use crate::classnames::class_name;
use crate::ptype::PType;

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct SourceFile {
    pub sourcefile_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct InnerClasses {
    pub classes: Vec<InnerClass>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct EnclosingMethod {
    pub class_index: CPIndex,
    pub method_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct SourceDebugExtension {
    pub debug_extension: Vec<u8>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct BootstrapMethods {
    pub bootstrap_methods: Vec<BootstrapMethod>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct NestHost {
    pub host_class_index: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct ConstantValue {
    pub constant_value_index: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct Code {
    pub attributes: Vec<AttributeInfo>,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code_raw: Vec<u8>,
    pub code: Vec<Instruction>,
    pub exception_table: Vec<ExceptionTableElem>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct ExceptionTableElem {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct Exceptions {
    pub exception_index_table: Vec<u16>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct RuntimeVisibleParameterAnnotations {
    pub parameter_annotations: Vec<Vec<Annotation>>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct RuntimeInvisibleParameterAnnotations {
    pub parameter_annotations: Vec<Vec<Annotation>>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct AnnotationDefault {
    pub default_value: ElementValue,
}


#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct MethodParameter {
    pub name_index: u16,
    pub access_flags: u16,
}


#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct MethodParameters {
    pub parameters: Vec<MethodParameter>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct Synthetic {}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct Deprecated {}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct Signature {
    pub signature_index: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct RuntimeVisibleAnnotations {
    pub annotations: Vec<Annotation>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct RuntimeInvisibleAnnotations {
    pub annotations: Vec<Annotation>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct LineNumberTable {
    pub line_number_table: Vec<LineNumberTableEntry>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct LocalVariableTable {
    pub local_variable_table: Vec<LocalVariableTableEntry>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct LocalVariableTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct LocalVariableTypeTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct LocalVariableTypeTable {
    pub type_table: Vec<LocalVariableTypeTableEntry>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct ObjectVariableInfo {
    pub cpool_index: Option<u16>,
    pub class_name: String,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct ArrayVariableInfo {
    pub array_type: PType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Hash)]
pub struct UninitializedVariableInfo {
    pub offset: u16,
}

impl Clone for UninitializedVariableInfo {
    fn clone(&self) -> Self {
        UninitializedVariableInfo { offset: self.offset }
    }
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
#[derive(Eq, PartialEq, Clone)]
pub struct SameFrame {
    pub offset_delta: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct SameLocals1StackItemFrame {
    pub offset_delta: u16,
    pub stack: PType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct SameLocals1StackItemFrameExtended {
    pub offset_delta: u16,
    pub stack: PType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct ChopFrame {
    pub offset_delta: u16,
    pub k_frames_to_chop: u8,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct SameFrameExtended {
    pub offset_delta: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct AppendFrame {
    pub offset_delta: u16,
    pub locals: Vec<PType>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct FullFrame {
    pub offset_delta: u16,
    pub number_of_locals: u16,
    pub locals: Vec<PType>,
    pub number_of_stack_items: u16,
    pub stack: Vec<PType>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
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
#[derive(Eq, PartialEq, Clone)]
pub struct StackMapTable {
    pub entries: Vec<StackMapFrame>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct LocalVarTargetTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub index: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub enum TargetInfo {
    TypeParameterTarget {
        type_parameter_index: u8
    },
    SuperTypeTarget {
        supertype_index: u16
    },
    TypeParameterBoundTarget {
        type_parameter_index: u8,
        bound_index: u8,
    },
    EmptyTarget,
    FormalParameterTarget {
        formal_parameter_index: u8
    },
    ThrowsTarget {
        throws_type_index: u16
    },
    LocalVarTarget {
        table: Vec<LocalVarTargetTableEntry>
    },
    CatchTarget {
        exception_table_entry: u16
    },
    OffsetTarget {
        offset: u16
    },
    TypeArgumentTarget {
        offset: u16,
        type_argument_index: u8,
    },
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct TypePathEntry {
    pub type_path_kind: u8,
    pub type_argument_index: u8,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct TypePath {
    pub path: Vec<TypePathEntry>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct TypeAnnotation {
    pub target_type: TargetInfo,
    pub target_path: TypePath,
    pub type_index: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}


#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct RuntimeVisibleTypeAnnotations {
    pub annotations: Vec<TypeAnnotation>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct RuntimeInvisibleTypeAnnotations {
    pub annotations: Vec<TypeAnnotation>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct NestMembers {
    pub classes: Vec<u16>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub enum AttributeType {
    SourceFile(SourceFile),
    InnerClasses(InnerClasses),
    EnclosingMethod(EnclosingMethod),
    SourceDebugExtension(SourceDebugExtension),
    BootstrapMethods(BootstrapMethods),
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
    Unknown,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Clone)]
pub struct BootstrapMethod {
    pub bootstrap_method_ref: u16,
    pub bootstrap_arguments: Vec<BootstrapArg>,
}


type BootstrapArg = u16;


#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct InnerClass {
    pub inner_class_info_index: CPIndex,
    pub outer_class_info_index: CPIndex,
    pub inner_name_index: CPIndex,
    pub inner_class_access_flags: CPIndex,
}


pub type CPIndex = u16;

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct EnumConstValue {
    pub type_name_index: CPIndex,
    pub const_name_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct ClassInfoIndex {
    pub class_info_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct AnnotationValue {
    pub annotation: Annotation,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct ArrayValue {
    pub values: Vec<ElementValue>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
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
#[derive(Eq, PartialEq, Clone)]
pub struct ElementValuePair {
    pub element_name_index: CPIndex,
    pub value: ElementValue,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct Annotation {
    pub type_index: u16,
    pub num_element_value_pairs: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct IInc {
    pub index: u16,
    pub const_: i16,
}


#[derive(Debug)]
#[derive(Eq)]
pub struct Utf8 {
    pub length: u16,
    pub string: Wtf8Buf,
}


impl PartialEq for Utf8 {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length &&
            self.string == other.string
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Integer {
    //unimplemented!()
    pub bytes: u32,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Float {
    pub bytes: u32,
    //unimplemented!()
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Long {
    pub low_bytes: u32,
    pub high_bytes: u32,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Double {
    pub low_bytes: u32,
    pub high_bytes: u32,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Class {
    //unimplemented!()
    pub name_index: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct String_ {
    //unimplemented!()
    pub string_index: u16,
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
#[derive(Clone)]
pub enum ReferenceKind {
    //1 REF_getField getfield C.f:T
    // 2 REF_getStatic getstatic C.f:T
    // 3 REF_putField putfield C.f:T
    // 4 REF_putStatic putstatic C.f:T
    // 5 REF_invokeVirtual invokevirtual C.m:(A*)T
    // 6 REF_invokeStatic invokestatic C.m:(A*)T
    // 7 REF_invokeSpecial invokespecial C.m:(A*)T
    // 8 REF_newInvokeSpecial new
    // C;
    // dup;
    // C.<init>:(A*)V
    // 9 REF_invokeInterface invokeinterface C.m:(A*)T
    // invokespeci
    GetField,
    GetStatic,
    PutField,
    PutStatic,
    InvokeVirtual,
    InvokeStatic,
    InvokeSpecial,
    NewInvokeSpecial,
    InvokeInterface,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodHandle {
    pub reference_kind: ReferenceKind,
    pub reference_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodType {
    pub descriptor_index: CPIndex,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InvokeDynamic {
    pub bootstrap_method_attr_index: CPIndex,
    pub name_and_type_index: CPIndex,
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
    InvokeDynamic(InvokeDynamic),
    InvalidConstant(InvalidConstant),
    LiveObject(usize),//live object pool index
}


#[derive(Debug)]
#[derive(Eq)]
pub struct ConstantInfo {
    pub kind: ConstantKind,
}

impl PartialEq for ConstantInfo {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}


#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
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
#[derive(FromPrimitive)]
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
#[derive(Copy, Clone)]
pub enum Wide {
    Iload(WideIload),
    Fload(WideFload),
    Aload(WideAload),
    Lload(WideLload),
    Dload(WideDload),
    Istore(WideIstore),
    Fstore(WideFstore),
    Astore(WideAstore),
    Lstore(WideLstore),
    Dstore(WideDstore),
    Ret(WideRet),
    IInc(IInc),
}
//iload, fload, aload, lload, dload, istore, fstore, astore,
// lstore, dstore, or ret

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideIload {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideFload {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideAload {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideLload {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideDload {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideIstore {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideFstore {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideAstore {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideLstore {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideDstore {
    pub index: u16,

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct WideRet {
    pub index: u16,
}


#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct Instruction {
    pub offset: u16,
    pub size: u16,
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
    if_icmpne(i16),
    if_icmplt(i16),
    if_icmpge(i16),
    if_icmpgt(i16),
    if_icmple(i16),
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


pub const ACC_PUBLIC: u16 = 0x0001;
pub const ACC_PRIVATE: u16 = 0x0002;
pub const ACC_PROTECTED: u16 = 0x0004;
pub const ACC_STATIC: u16 = 0x0008;
pub const ACC_FINAL: u16 = 0x0010;
pub const ACC_SUPER: u16 = 0x0020;
pub const ACC_BRIDGE: u16 = 0x0040;
pub const ACC_VOLATILE: u16 = 0x0040;
pub const ACC_TRANSIENT: u16 = 0x0080;
pub const ACC_VARARGS: u16 = 0x0080;
pub const ACC_NATIVE: u16 = 0x0100;
pub const ACC_INTERFACE: u16 = 0x0200;
pub const ACC_ABSTRACT: u16 = 0x0400;
pub const ACC_STRICT: u16 = 0x0800;
pub const ACC_SYNTHETIC: u16 = 0x1000;
pub const ACC_ANNOTATION: u16 = 0x2000;
pub const ACC_ENUM: u16 = 0x4000;
pub const ACC_MODULE: u16 = 0x8000;

pub const REF_GET_FIELD: u8 = 1;
pub const REF_GET_STATIC: u8 = 2;
pub const REF_PUT_FIELD: u8 = 3;
pub const REF_PUT_STATIC: u8 = 4;
pub const REF_INVOKE_VIRTUAL: u8 = 5;
pub const REF_INVOKE_STATIC: u8 = 6;
pub const REF_INVOKE_SPECIAL: u8 = 7;
pub const REF_NEW_INVOKE_SPECIAL: u8 = 8;
pub const REF_INVOKE_INTERFACE: u8 = 9;

#[derive(Debug)]
pub struct Classfile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    //todo look at this for code size reduction/simplification opturnities
    pub constant_pool: Vec<ConstantInfo>,
    pub access_flags: u16,
    pub this_class: CPIndex,
    pub super_class: CPIndex,
    pub interfaces: Vec<Interface>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

pub type Interface = u16;

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
        state.write(class_name(self).get_referred_name().as_bytes());
        state.write_u32(self.magic);
        state.write_u16(self.minor_version);
        state.write_u16(self.major_version);
        state.write_u16(self.access_flags);
        state.write_u16(self.this_class);
        state.write_u16(self.super_class);
        for interface in &self.interfaces {
            state.write_u16(*interface)
        }
    }
}


impl From<ConstantKind> for ConstantInfo {
    fn from(kind: ConstantKind) -> Self {
        Self { kind }
    }
}