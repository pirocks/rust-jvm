use rust_jvm_common::NativeJavaValue;
use rust_jvm_common::runtime_type::RuntimeType;


#[derive(Debug, Clone)]
pub enum BeforeState<'gc> {
    NoValidate,
    TopOfOperandStackIs{
        native_jv: NativeJavaValue<'gc>,
        rtype: RuntimeType
    },
}



pub mod interpreted_impls;