use java5_verifier::{InferredFrame, SimplifiedVType};
use rust_jvm_common::classfile::InstructionInfo::fadd;
use rust_jvm_common::vtype::VType;
use verification::verifier::Frame;

pub enum SunkVerifierFrames {
    FullFrame(Frame),
    PartialInferredFrame(java5_verifier::InferredFrame),
}

impl SunkVerifierFrames {

    pub fn try_unwrap_full_frame(&self) -> Option<&Frame> {
        match self {
            SunkVerifierFrames::FullFrame(full_frame) => Some(full_frame),
            SunkVerifierFrames::PartialInferredFrame(_) => None
        }
    }

    pub fn unwrap_full_frame(&self) -> &Frame {
        self.try_unwrap_full_frame().unwrap()
    }

    pub fn try_unwrap_partial_inferred_frame(&self) -> Option<&InferredFrame>{
        match self {
            SunkVerifierFrames::FullFrame(_) => None,
            SunkVerifierFrames::PartialInferredFrame(inferred_frame) => Some(inferred_frame)
        }
    }

    pub fn unwrap_partial_inferred_frame(&self) -> &InferredFrame{
        self.try_unwrap_partial_inferred_frame().unwrap()
    }

    pub fn stack_depth_no_tops(&self) -> usize {
        match self {
            SunkVerifierFrames::FullFrame(frame) => {
                assert!(frame.stack_map.iter().all(|types| !matches!(types, VType::TopType)));
                frame.stack_map.len()
            },
            SunkVerifierFrames::PartialInferredFrame(frame) => {
                assert!(frame.operand_stack.iter().all(|types| !matches!(types, SimplifiedVType::Top)));
                frame.operand_stack.len()
            }
        }
    }


    pub fn is_category_2_no_tops(&self) -> Vec<bool> {
        match self {
            SunkVerifierFrames::FullFrame(frame) => {
                assert!(frame.stack_map.iter().all(|types| !matches!(types, VType::TopType)));
                frame.stack_map.iter().map(|vtype| is_type_2_computational_type(vtype)).collect()
            },
            SunkVerifierFrames::PartialInferredFrame(frame) => {
                frame.operand_stack.iter().map(|vtype|match vtype {
                    SimplifiedVType::OneWord => false,
                    SimplifiedVType::TwoWord => true,
                    SimplifiedVType::Top => panic!()
                }).collect()
            }
        }
    }
}

fn is_type_2_computational_type(vtype: &VType) -> bool {
    match vtype {
        VType::DoubleType => true,
        VType::FloatType => false,
        VType::IntType => false,
        VType::LongType => true,
        VType::Class(_) => false,
        VType::ArrayReferenceType(_) => false,
        VType::VoidType => false,
        VType::TopType => false,
        VType::NullType => false,
        VType::Uninitialized(_) => false,
        VType::UninitializedThis => false,
        VType::UninitializedThisOrClass(_) => false,
        VType::TwoWord => true,
        VType::OneWord => false,
        VType::Reference => false,
        VType::UninitializedEmpty => false
    }
}
