use std::sync::Arc;

use jvmti_jni_bindings::{jboolean, jint};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{AllocatedHandle, JavaValueCommon, JVMState, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::interpreter_util::{new_object, run_constructor};
use crate::java_values::JavaValue;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub struct JThreadGroup<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn try_cast_thread_group(&self) -> Option<JThreadGroup<'gc>> {
        todo!()
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_thread_group(self) -> JThreadGroup<'gc> {
        JThreadGroup { normal_object: self.unwrap_object_nonnull().unwrap_normal_object() }
    }

    pub fn try_cast_thread_group(self) -> Option<JThreadGroup<'gc>> {
        /*match self.try_unwrap_normal_object() {
            Some(normal_object) => {
                if normal_object.objinfo.class_pointer.view().name() == CClassName::thread_group().into() {
                    return JThreadGroup { normal_object: self.unwrap_object_nonnull() }.into();
                }
                None
            }
            None => None,
        }*/
        todo!()
    }
}

impl Clone for JThreadGroup<'_> {
    fn clone(&self) -> Self {
        JThreadGroup { normal_object: self.normal_object.duplicate_discouraged() }
    }
}

impl<'gc> JThreadGroup<'gc> {
    pub fn init<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, thread_group_class: Arc<RuntimeClass<'gc>>) -> Result<JThreadGroup<'gc>, WasException<'gc>> {
        let thread_group_object = NewJavaValueHandle::Object(AllocatedHandle::NormalObject(new_object(jvm, int_state, &thread_group_class, false)));
        run_constructor(jvm, int_state, thread_group_class, vec![thread_group_object.as_njv()], &CMethodDescriptor::void_return(vec![]))?;
        Ok(thread_group_object.cast_thread_group())
    }

    pub fn name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_name()).cast_string_maybe_null().expect("thread group null name")
    }

    pub fn daemon(&self, jvm: &'gc JVMState<'gc>) -> jboolean {
        self.normal_object.get_var_top_level(jvm, FieldName::field_daemon()).unwrap_bool_strict()
    }

    pub fn max_priority(&self, jvm: &'gc JVMState<'gc>) -> jint {
        self.normal_object.get_var_top_level(jvm, FieldName::field_maxPriority()).unwrap_int()
    }

    pub fn parent(&self, jvm: &'gc JVMState<'gc>) -> Option<JThreadGroup<'gc>> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_parent()).try_cast_thread_group()
    }


}

impl<'gc> NewAsObjectOrJavaValue<'gc> for JThreadGroup<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
