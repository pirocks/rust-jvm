use std::collections::HashSet;
use std::sync::Arc;

use bimap::BiMap;
use by_address::ByAddress;
use itertools::Itertools;

use another_jit_vm_ir::ir_stack::IsOpaque;
use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jobject};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::opaque_id_table::OpaqueID;

use crate::ir_to_java_layer::java_stack::{OpaqueFrameIdOrMethodID, RuntimeJavaStackFrameMut, RuntimeJavaStackFrameRef};
use crate::java_values::{GcManagedObject, JavaValue};
use crate::jvm_state::JVMState;
use crate::NewJavaValue;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RuntimeClassClassId(usize);

/// If the frame is opaque then this data is optional.
/// This data would typically be present in a native function call, but not be present in JVMTI frames
#[derive(Debug, Clone)]
pub struct OpaqueFrameOptional<'gc> {
    pub class_pointer: Arc<RuntimeClass<'gc>>,
    pub method_i: CPIndex,
}

///This data is only present in non-native frames,
/// program counter is not meaningful in a native frame
#[derive(Debug, Clone)]
pub struct NonNativeFrameData {
    pub pc: u16,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: i32,
}

#[derive(Clone)]
pub struct JavaFramePush<'gc, 'k> {
    pub(crate) method_id: MethodId,
    pub(crate) local_vars: Vec<NewJavaValue<'gc, 'k>>,
    pub(crate) operand_stack: Vec<NewJavaValue<'gc, 'k>>,
}

#[derive(Clone)]
pub struct NativeFramePush<'gc, 'k> {
    pub(crate) method_id: MethodId,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
    pub(crate) local_vars: Vec<NewJavaValue<'gc, 'k>>,
    pub(crate) operand_stack: Vec<NewJavaValue<'gc, 'k>>,
}

#[derive(Clone)]
pub struct OpaqueFramePush {
    pub(crate) opaque_id: OpaqueID,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
}

#[derive(Clone)]
pub enum StackEntryPush<'gc, 'k> {
    Java(JavaFramePush<'gc, 'k>),
    // a native function call frame
    Native(NativeFramePush<'gc, 'k>),
    Opaque(OpaqueFramePush),
}

impl<'gc, 'k> StackEntryPush<'gc, 'k> {
    pub fn new_native_frame(jvm: &'gc JVMState<'gc>, class_pointer: Arc<RuntimeClass<'gc>>, method_i: u16, args: Vec<NewJavaValue<'gc, 'k>>) -> NativeFramePush<'gc, 'k> {
        let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer, method_i);
        NativeFramePush {
            method_id,
            native_local_refs: vec![HashSet::new()],
            local_vars: args,
            operand_stack: vec![],
        }
    }

    pub fn new_java_frame(jvm: &'gc JVMState<'gc>, class_pointer: Arc<RuntimeClass<'gc>>, method_i: u16, args: Vec<NewJavaValue<'gc, 'k>>) -> JavaFramePush<'gc, 'k> {
        let max_locals = class_pointer.view().method_view_i(method_i).code_attribute().unwrap().max_locals;
        assert_eq!(args.len(), max_locals as usize);
        let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer.clone(), method_i);
        // assert!(jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 }).is_some());
        let _loader = jvm.classes.read().unwrap().get_initiating_loader(&class_pointer);
        let mut guard = jvm.method_table.write().unwrap();
        let _method_id = guard.get_method_id(class_pointer.clone(), method_i);
        let class_view = class_pointer.view();
        let method_view = class_view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        let operand_stack = (0..code.max_stack).map(|_| NewJavaValue::Top).collect_vec();
        JavaFramePush {
            method_id,
            local_vars: args,
            operand_stack,
        }
    }

    pub fn new_completely_opaque_frame(jvm: &'gc JVMState<'gc>, loader: LoaderName, operand_stack: Vec<JavaValue<'gc>>, debug_str: &'static str) -> OpaqueFramePush {
        //need a better name here
        assert!(operand_stack.is_empty());
        assert_eq!(loader, LoaderName::BootstrapLoader);// loader should be set from thread loader for new threads
        let opaque_id = jvm.opaque_ids.write().unwrap().new_opaque_id(debug_str);
        OpaqueFramePush {
            opaque_id,
            native_local_refs: vec![],
        }
    }
}


#[derive(Debug, Clone)]
pub enum StackEntry {
    Java {
        method_id: MethodId,
    },
    Native {
        // a native function call frame
        method_id: MethodId,
    },
    Opaque {
        opaque_id: OpaqueID,
    },
}
//
// #[derive(Debug, Clone)]
// pub struct StackEntry<'gc> {
//     pub(crate) loader: LoaderName,
//     pub(crate) opaque_frame_id: Option<u64>,
//     pub(crate) opaque_frame_optional: Option<OpaqueFrameOptional<'gc>>,
//     pub(crate) non_native_data: Option<NonNativeFrameData>,
//     pub(crate) local_vars: Vec<JavaValue<'gc>>,
//     pub(crate) operand_stack: Vec<JavaValue<'gc>>,
//     pub(crate) native_local_refs: Vec<HashSet<jobject>>,
// }

pub struct StackEntryMut<'gc, 'l> {
    pub frame_view: RuntimeJavaStackFrameMut<'gc, 'l>,
}


#[derive(Clone)]
pub enum LocalVarsRef<'gc, 'l, 'k> {
    /*    LegacyInterpreter {
        vars: &'l Vec<JavaValue>
    },*/
    Jit {
        frame_view: &'k RuntimeJavaStackFrameRef<'gc, 'l>,
        jvm: &'gc JVMState<'gc>,
        pc: Option<ByteCodeOffset>,
    },
}

impl<'gc> LocalVarsRef<'gc, '_, '_> {

    pub fn num_vars(&self) -> u16 {
        match self {
            LocalVarsRef::Jit { frame_view, jvm, .. } => {
                let method_id = frame_view.ir_ref.method_id().expect("local vars should have method id probably");
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let num_args = method_view.code_attribute().map(|code| code.max_locals).unwrap_or(method_view.num_args());
                num_args
            }
        }
    }
}

pub enum OperandStackRef<'gc, 'l, 'k> {
    /*LegacyInterpreter {
        operand_stack: &'l Vec<JavaValue>
    },*/
    Jit {
        frame_view: &'k RuntimeJavaStackFrameRef<'gc, 'l>,
        jvm: &'gc JVMState<'gc>,
        pc: Option<ByteCodeOffset>,
    },
}

impl<'gc, 'l, 'k> OperandStackRef<'gc, 'l, 'k> {

}

pub struct StackEntryRef<'gc, 'l> {
    pub(crate) frame_view: RuntimeJavaStackFrameRef<'gc, 'l>,
    //todo in future may want to pass ir instr index info back down to ir layer, esp is relevant data is in registers
    pub(crate) pc: Option<ByteCodeOffset>,// option is for opaque frames or similar.
}

impl<'gc, 'l> StackEntryRef<'gc, 'l> {
    pub fn loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
        let method_id = match self.frame_view.ir_ref.method_id() {
            Ok(x) => x,
            Err(IsOpaque {}) => {
                //opaque frame todo, should lookup loader by opaque id
                return LoaderName::BootstrapLoader;
            }
        };
        let rc = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().0;
        jvm.classes.read().unwrap().get_initiating_loader(&rc)
    }

    pub fn try_class_pointer(&self, jvm: &'gc JVMState<'gc>) -> Result<Arc<RuntimeClass<'gc>>, IsOpaque> {
        let method_id = self.frame_view.ir_ref.method_id()?;
        let rc = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().0;
        Ok(rc)
    }

    pub fn is_native_method(&self) -> bool {
        match self.frame_view.ir_ref.method_id() {
            Err(IsOpaque {}) => false,
            Ok(method_id) => {
                self.frame_view.jvm.is_native_by_method_id(method_id)
            }
        }
    }

    pub fn is_opaque(&self) -> bool {
        let opaque_frame_or_method_id = OpaqueFrameIdOrMethodID::from_native(self.frame_view.ir_ref.raw_method_id());
        opaque_frame_or_method_id.is_opaque()
    }


    pub fn native_local_refs(&self) -> &mut Vec<BiMap<ByAddress<GcManagedObject<'gc>>, jobject>> {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => todo!("{:?}", entry),*/
            StackEntryRef::Jit { frame_view, .. } => todo!("{:?}", frame_view),
        }*/
        todo!()
    }

    pub fn local_vars<'k>(&'k self, jvm: &'gc JVMState<'gc>) -> LocalVarsRef<'gc, 'l, 'k> {
        LocalVarsRef::Jit {
            frame_view: &self.frame_view,
            jvm,
            pc: self.pc,
        }
    }
}

impl<'gc> StackEntry {
    pub fn class_pointer(&self) -> &Arc<RuntimeClass<'gc>> {
        todo!()
        /*&match self.opaque_frame_optional.as_ref() {
            Some(x) => x,
            None => {
                unimplemented!()
            }
        }
            .class_pointer*/
    }

    pub fn try_class_pointer(&self, jvm: &'gc JVMState<'gc>) -> Option<Arc<RuntimeClass<'gc>>> {
        match self {
            StackEntry::Native { method_id } |
            StackEntry::Java { method_id } => {
                let (rc, _method_id) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                return Some(rc);
            }
            StackEntry::Opaque { .. } => None,
        }
    }

    pub fn local_vars(&self) -> &Vec<JavaValue<'gc>> {
        todo!()
        /*&self.local_vars*/
    }

    pub fn local_vars_mut(&mut self) -> &mut Vec<JavaValue<'gc>> {
        todo!()
        /*&mut self.local_vars*/
    }

    pub fn operand_stack_mut(&mut self) -> &mut Vec<JavaValue<'gc>> {
        todo!()
        /*&mut self.operand_stack*/
    }

    pub fn operand_stack(&self) -> &Vec<JavaValue<'gc>> {
        todo!()
        /*&self.operand_stack*/
    }

    pub fn pc_mut(&mut self) -> &mut u16 {
        todo!()
        /*&mut self.non_native_data.as_mut().unwrap().pc*/
    }

    pub fn pc(&self) -> u16 {
        self.try_pc().unwrap()
    }

    pub fn try_pc(&self) -> Option<u16> {
        todo!()
        /*self.non_native_data.as_ref().map(|x| x.pc)*/
    }

    //todo a lot of duplication here between mut and non-mut variants
    pub fn pc_offset_mut(&mut self) -> &mut i32 {
        todo!()
        /*&mut self.non_native_data.as_mut().unwrap().pc_offset*/
    }

    pub fn pc_offset(&self) -> i32 {
        todo!()
        /*self.non_native_data.as_ref().unwrap().pc_offset*/
    }

    pub fn method_i(&self) -> CPIndex {
        todo!()
        /*self.opaque_frame_optional.as_ref().unwrap().method_i*/
    }

    pub fn try_method_i(&self) -> Option<CPIndex> {
        todo!()
        /*self.opaque_frame_optional.as_ref().map(|x| x.method_i)*/
    }

    pub fn is_native(&self) -> bool {
        let method_i = match self.try_method_i() {
            None => return true,
            Some(i) => i,
        };
        self.class_pointer().view().method_view_i(method_i).is_native()
    }

}