use another_jit_vm::code_modification::{CodeModificationHandle, EditAction};
use another_jit_vm_ir::changeable_const::{ChangeableConst, ChangeableConstID};


// put struct edit action
// set editable const to correct field number




pub enum VMExitEditAction{
    EditFunctionCallTarget{
        action: EditAction
    },
    SkipExit{
        action: EditAction
    },
    EditFieldNumber{
        action: !
    },
    Double{
        inner: [VMExitEditAction;2]
    }
}



impl VMExitEditAction {
    pub fn edit(&self, code_modification_handle: &CodeModificationHandle){
        match self {
            VMExitEditAction::EditFunctionCallTarget { .. } => todo!(),
            VMExitEditAction::EditFieldNumber { .. } => todo!(),
            VMExitEditAction::SkipExit { .. } => todo!(),
            VMExitEditAction::Double { .. } => todo!(),
        }
    }
}




pub mod put_field{
    use std::collections::HashMap;
    use std::sync::Arc;
    use rust_jvm_common::compressed_classfile::CPDType;
    use rust_jvm_common::compressed_classfile::names::FieldName;
    use rust_jvm_common::loading::LoaderName;

    pub struct IREditablePutField{

        ir_index_base: IRInstructionIndex,

    }

    pub struct EditablePutField{
        field_name: FieldName,
        field_type: CPDType
    }


    pub struct PutFieldToEdit{
        class_to_load: HashMap<(CPDType,LoaderName), Vec<EditablePutField>>
    }
}