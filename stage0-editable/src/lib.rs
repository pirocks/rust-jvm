use another_jit_vm::code_modification::{CodeModificationHandle, EditAction};




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

