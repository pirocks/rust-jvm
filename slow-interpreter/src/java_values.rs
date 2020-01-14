pub enum JavaValue{
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(bool),
    Char(char),

    Float(f32),
    Double(f64),

    Array(Vec<JavaValue>),
    Object(Object),

    Top//should never be interacted with by the bytecode
}

pub struct Object {
    class_pointer: Arc<RuntimeClass>,//I guess this never changes so uneeded?
    fields : Map<String,JavaValue>
}