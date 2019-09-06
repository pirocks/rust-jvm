use classfile::AttributeInfo;
use classfile::parsing_util::ParsingContext;

pub struct SourceFile{
    //todo
}

pub struct InnerClasses{
    //todo
}

pub struct EnclosingMethod{
    //todo
}

pub struct SourceDebugExtension{
    //todo
}

pub struct BootstrapMethods{
    //todo
}

pub struct Module{
    //todo
}

pub struct NestHost{
    //todo
}

pub struct ConstantValue{
    //todo
}

pub struct Code{
    //todo
}

pub struct Exceptions{
    //todo
}

pub struct RuntimeVisibleParameterAnnotations{
    //todo
}

pub struct RuntimeInvisibleParameterAnnotations{
    //todo
}

pub struct AnnotationDefault{
    //todo
}

pub struct MethodParameters{
    //todo
}

pub struct Synthetic{
    //todo
}

pub struct Deprecated{
    //todo
}

pub struct Signature{
    //todo
}

pub struct RuntimeVisibleAnnotations{
    //todo
}

pub struct RuntimeInvisibleAnnotations{
    //todo
}

pub struct LineNumberTable{
    //todo
}

pub struct LocalVariableTable{
    //todo
}

pub struct LocalVariableTypeTable{
    //todo
}

pub struct StackMapTable{
    //todo
}

pub struct RuntimeVisibleTypeAnnotations{
    //todo
}

pub struct RuntimeInvisibleTypeAnnotations{
    //todo
}

pub enum AttributeType{
    SourceFile,
    InnerClasses,
    EnclosingMethod,
    SourceDebugExtension,
    BootstrapMethods,
    Module,
    NestHost,
    ConstantValue,
    Code,
    Exceptions,
    RuntimeVisibleParameterAnnotations,
    RuntimeInvisibleParameterAnnotations,
    AnnotationDefault,
    MethodParameters,
    Synthetic,
    Deprecated,
    Signature,
    RuntimeVisibleAnnotations,
    RuntimeInvisibleAnnotations,
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    StackMapTable,
    RuntimeVisibleTypeAnnotations,
    RuntimeInvisibleTypeAnnotations
}

pub fn parse_attributes(p :& mut ParsingContext, num_attributes: u16 ) -> Vec<AttributeInfo>{
    return todo!()
}