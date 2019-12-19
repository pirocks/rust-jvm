use crate::classfile::{ConstantInfo, ConstantKind};

//todo this should go at top
pub fn extract_string_from_utf8(utf8: &ConstantInfo) -> String {
    match &(utf8).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        }
        other => {
            dbg!(other);
            panic!()
        }
    }
}
