use std::io;
use rust_jvm_common::unified_types::UnifiedType;
use std::io::Write;
use rust_jvm_common::loading::BOOTSTRAP_LOADER_NAME;
use rust_jvm_common::classnames::get_referred_name;

pub fn write_type_prolog(type_: &UnifiedType, w: &mut dyn Write) -> Result<(), io::Error>{
    match type_{
        UnifiedType::ByteType => {
            write!(w,"int")?;
        },
        UnifiedType::CharType => {
            write!(w,"int")?;
        },
        UnifiedType::DoubleType => {
            write!(w,"double")?;
        },
        UnifiedType::FloatType => {
            write!(w,"float")?;
        },
        UnifiedType::IntType => {
            write!(w,"int")?;
        },
        UnifiedType::LongType => {
            write!(w,"long")?;
        },
        UnifiedType::ShortType => {
            write!(w,"int")?;
        },
        UnifiedType::BooleanType => {
            write!(w,"int")?;
        },
        UnifiedType::Class(ref_) => {
//            if context.state.using_bootstrap_loader {
                write!(w,"class('")?;
                write!(w,"{}",get_referred_name(ref_))?;
                write!(w,"',{})",BOOTSTRAP_LOADER_NAME)?;
//            } else {
//                unimplemented!()
//            }
        },
        UnifiedType::ArrayReferenceType(arr) => {
            write!(w, "arrayOf(")?;
            write_type_prolog(&arr.sub_type, w)?;
            write!(w, ")")?;
        },
        UnifiedType::VoidType => {
            write!(w,"void")?;
        },
        _ => {panic!("Case wasn't coverred with non-unified types")}
    }
    Ok(())
}