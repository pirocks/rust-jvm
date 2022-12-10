use rust_jvm_common::opaque_id_table::OpaqueID;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum OpaqueFrameIdOrMethodID {
    Opaque {
        opaque_id: OpaqueID,
    },
    Method {
        method_id: u64
    },
}

impl OpaqueFrameIdOrMethodID {
    pub fn to_native(&self) -> i64 {
        match self {
            OpaqueFrameIdOrMethodID::Opaque { opaque_id } => {
                -((opaque_id.0 + 1) as i64)
            }
            OpaqueFrameIdOrMethodID::Method { method_id } => {
                *method_id as i64
            }
        }
    }

    pub fn from_native(native: i64) -> Self {
        if native < 0 {
            Self::Opaque { opaque_id: OpaqueID(((-native) as u64) - 1) }
        } else {
            Self::Method { method_id: native as u64 }
        }
    }

    pub fn is_opaque(&self) -> bool {
        match self {
            OpaqueFrameIdOrMethodID::Opaque { .. } => true,
            OpaqueFrameIdOrMethodID::Method { .. } => false
        }
    }
}

