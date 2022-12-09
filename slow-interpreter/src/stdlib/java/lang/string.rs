use itertools::Itertools;
use wtf8::Wtf8Buf;

use jvmti_jni_bindings::{jchar};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use crate::{AllocatedHandle, JavaValueCommon, JVMState, NewJavaValue, UnAllocatedObject, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::interpreter_util::{new_object, run_constructor};
use crate::java_values::JavaValue;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::unallocated_objects::UnAllocatedObjectArray;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::utils::run_static_or_virtual;

pub struct JString<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl Clone for JString<'_> {
    fn clone(&self) -> Self {
        JString {
            normal_object: self.normal_object.duplicate_discouraged()
        }
    }
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_string(&self) -> Option<JString<'gc>> {
        todo!()
        /*Some(JString { normal_object: self.unwrap_object()? })*/
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_string(self) -> Option<JString<'gc>> {
        Some(JString { normal_object: self.unwrap_object()?.unwrap_normal_object() })
    }
}

impl<'gc> AllocatedHandle<'gc> {
    pub fn cast_string(self) -> JString<'gc> {
        JString { normal_object: self.unwrap_normal_object() }
    }
}

impl<'gc> JString<'gc> {
    pub fn to_rust_string(&self, jvm: &'gc JVMState<'gc>) -> String {
        let str_obj = &self.normal_object;
        let str_class_pointer = assert_inited_or_initing_class(jvm, CClassName::string().into());
        let temp = str_obj.get_var(jvm, &str_class_pointer, FieldName::field_value());
        let nonnull = temp.unwrap_object_nonnull();
        let chars = nonnull.unwrap_array();
        let borrowed_elems = chars.array_iterator();
        char::decode_utf16(borrowed_elems.map(|jv| jv.unwrap_char_strict())).collect::<Result<String, _>>().expect("really weird string encountered")
    }

    pub fn from_rust(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, rust_str: Wtf8Buf) -> Result<JString<'gc>, WasException<'gc>> {
        let string_class = check_initing_or_inited_class(jvm, int_state, CClassName::string().into()).unwrap(); //todo replace these unwraps
        let string_object = AllocatedHandle::NormalObject(new_object(jvm, int_state, &string_class, false));
        let elems = rust_str.to_ill_formed_utf16().map(|c| NewJavaValue::Char(c as u16)).collect_vec();
        let array_object = UnAllocatedObjectArray {
            whole_array_runtime_class: check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::CharType)).unwrap(),
            elems,
        };
        //todo what about check_inited_class for this array type
        let array = NewJavaValueHandle::Object(jvm.allocate_object(UnAllocatedObject::Array(array_object)));
        // dbg!(array.as_njv().to_handle_discouraged().unwrap_object_nonnull().unwrap_array().array_iterator().map(|elem| elem.unwrap_char_strict()).collect_vec());
        run_constructor(jvm, int_state, string_class, vec![string_object.new_java_value(), array.as_njv()], &CMethodDescriptor::void_return(vec![CPDType::array(CPDType::CharType)]))?;
        Ok(NewJavaValueHandle::Object(string_object).cast_string().expect("error creating string"))
    }

    pub fn intern<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<JString<'gc>, WasException<'gc>> {
        let string_class = check_initing_or_inited_class(jvm, int_state, CClassName::string().into())?;
        let args = vec![self.new_java_value()];
        let res = run_static_or_virtual(
            jvm,
            int_state,
            &string_class,
            MethodName::method_intern(),
            &CMethodDescriptor::empty_args(CClassName::string().into()),
            args,
        )?.unwrap();
        Ok(res.cast_string().expect("error interning strinng"))
    }

    pub fn value(&self, jvm: &'gc JVMState<'gc>) -> Vec<jchar> {
        let string_class = assert_inited_or_initing_class(jvm, CClassName::string().into());
        let mut res = vec![];
        for elem in self.normal_object.get_var(jvm, &string_class, FieldName::field_value()).unwrap_object_nonnull().unwrap_array().array_iterator() {
            res.push(elem.as_njv().unwrap_char_strict())
        }
        res
    }

    pub fn to_rust_string_better(&self, jvm: &'gc JVMState<'gc>) -> Option<String> {
        let string_class = assert_inited_or_initing_class(jvm, CClassName::string().into());
        let as_allocated_obj = &self.normal_object;
        let value_field = as_allocated_obj.get_var(jvm, &string_class, FieldName::field_value());
        value_field.as_njv().unwrap_object_alloc()?;
        let mut res = vec![];
        for elem in value_field.unwrap_object_nonnull().unwrap_array().array_iterator() {
            res.push(elem.as_njv().unwrap_char_strict())
        }
        String::from_utf16(res.as_slice()).ok()
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for JString<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
