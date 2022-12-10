use std::ffi::c_void;
use std::sync::Arc;

use wtf8::Wtf8Buf;

use jvmti_jni_bindings::{jint};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use rust_jvm_common::JavaThreadId;

use crate::{AllocatedHandle, JavaValueCommon, JVMState, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::java_values::JavaValue;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::stdlib::java::lang::class_loader::ClassLoader;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::lang::thread_group::JThreadGroup;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::threading::java_thread::JavaThread;
use crate::utils::run_static_or_virtual;

pub struct JThread<'gc> {
    jvm: &'gc JVMState<'gc>,
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_thread(&self) -> JThread<'gc> {
        todo!()
    }

    pub fn try_cast_thread(&self) -> Option<JThread<'gc>> {
        todo!()
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_thread(self, jvm: &'gc JVMState<'gc>) -> JThread<'gc> {
        let normal_object = self.unwrap_object_nonnull().unwrap_normal_object();
        // assert_eq!(normal_object.as_allocated_obj().runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool), CPDType::from(CClassName::thread()).jvm_representation(&jvm.string_pool));
        JThread { jvm, normal_object }
    }

    pub fn try_cast_thread(self, jvm: &'gc JVMState<'gc>) -> Option<JThread<'gc>> {
        let normal_object = self.unwrap_object()?.unwrap_normal_object();
        // assert_eq!(normal_object.as_allocated_obj().runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool), CPDType::from(CClassName::thread()).jvm_representation(&jvm.string_pool));
        Some(JThread { jvm, normal_object }.into())
    }
}

impl Clone for JThread<'_> {
    fn clone(&self) -> Self {
        let jvm = self.jvm;
        // assert_eq!(self.normal_object.duplicate_discouraged().as_allocated_obj().runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool), CPDType::from(CClassName::thread()).jvm_representation(&jvm.string_pool));
        JThread { jvm, normal_object: self.normal_object.duplicate_discouraged() }
    }
}

impl<'gc> JThread<'gc> {

    pub fn tid(&self, jvm: &'gc JVMState<'gc>) -> JavaThreadId {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        self.normal_object.get_var(jvm, &thread_class, FieldName::field_tid()).as_njv().unwrap_long_strict()
    }

    pub fn run<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
        let args = vec![self.normal_object.new_java_value()];
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_run(), &CompressedMethodDescriptor::empty_args(CPDType::VoidType), args)?;
        Ok(())
    }

    pub fn exit<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        let args = vec![self.new_java_value()];
        let desc = CompressedMethodDescriptor::empty_args(CPDType::VoidType);
        run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_exit(), &desc, args)?;
        Ok(())
    }

    pub fn try_name(&self, jvm: &'gc JVMState<'gc>) -> Option<JString<'gc>> {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        self.normal_object.get_var(jvm, &thread_class, FieldName::field_name()).cast_string()
    }

    pub fn name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
        self.try_name(jvm).unwrap()
    }

    pub fn priority(&self, jvm: &'gc JVMState<'gc>) -> i32 {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        self.normal_object.get_var(jvm, &thread_class, FieldName::field_priority()).unwrap_int()
    }

    fn top_level_rc(&self) -> Arc<RuntimeClass<'gc>> {
        assert_inited_or_initing_class(&self.normal_object.jvm, CClassName::thread().into())
    }

    fn thread_class(&self) -> Arc<RuntimeClass<'gc>> {
        self.top_level_rc()
    }

    pub fn set_priority(&self, priority: i32) {
        let thread_class = self.thread_class();
        self.normal_object.set_var(&thread_class, FieldName::field_priority(), NewJavaValue::Int(priority));
    }

    pub fn daemon(&self, jvm: &'gc JVMState<'gc>) -> bool {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        self.normal_object.get_var(jvm, &thread_class, FieldName::field_daemon()).unwrap_int() != 0
    }

    pub fn set_thread_status(&self, jvm: &'gc JVMState<'gc>, thread_status: jint) {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        self.normal_object.set_var(&thread_class, FieldName::field_threadStatus(), NewJavaValue::Int(thread_status));
    }

    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, thread_group: JThreadGroup<'gc>, thread_name: String) -> Result<JThread<'gc>, WasException<'gc>> {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        let thread_object = NewJavaValueHandle::Object(AllocatedHandle::NormalObject(new_object(jvm, int_state, &thread_class, false)));
        let thread_name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(thread_name))?;
        run_constructor(jvm, int_state, thread_class, vec![thread_object.as_njv(), thread_group.new_java_value_handle().as_njv(), thread_name.new_java_value_handle().as_njv()], &CMethodDescriptor::void_return(vec![CClassName::thread_group().into(), CClassName::string().into()]))?;
        Ok(thread_object.cast_thread(jvm))
    }

    pub fn get_java_thread(&self, jvm: &'gc JVMState<'gc>) -> Arc<JavaThread<'gc>> {
        self.try_get_java_thread(jvm).unwrap()
    }

    pub fn try_get_java_thread(&self, jvm: &'gc JVMState<'gc>) -> Option<Arc<JavaThread<'gc>>> {
        let tid = self.tid(jvm);
        jvm.thread_state.try_get_thread_by_tid(tid)
    }

    pub fn get_context_class_loader<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Option<ClassLoader<'gc>>, WasException<'gc>> {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        let mut args = vec![];
        args.push(self.new_java_value());
        let res = run_static_or_virtual(
            jvm,
            int_state,
            &thread_class,
            MethodName::method_getContextClassLoader(),
            &CompressedMethodDescriptor::empty_args(CClassName::classloader().into()),
            args,
        )?.unwrap();
        if res.as_njv().unwrap_object().is_none() {
            return Ok(None);
        }
        Ok(res.unwrap_object().unwrap().cast_class_loader().into())
    }

    pub fn get_inherited_access_control_context(&self, jvm: &'gc JVMState<'gc>) -> JThread<'gc> {
        let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
        self.normal_object.get_var(jvm,&thread_class, FieldName::field_inheritedAccessControlContext()).cast_thread(jvm)
    }

    pub fn notify_object_change(&self, jvm: &'gc JVMState<'gc>) {
        jvm.monitor_for(self.normal_object.ptr.as_ptr() as *const c_void).notify_all(jvm).unwrap();
    }

}

impl<'gc> NewAsObjectOrJavaValue<'gc> for JThread<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
