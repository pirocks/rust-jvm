pub mod lookup {
    use std::ops::Deref;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;


    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::java_values::JavaValue;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::static_vars::static_vars;
    use crate::stdlib::java::lang::class::JClass;
    use crate::stdlib::java::lang::invoke::method_handle::MethodHandle;
    use crate::stdlib::java::lang::invoke::method_type::MethodType;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::NewAsObjectOrJavaValue;
    use crate::utils::run_static_or_virtual;
    use crate::WasException;

    #[derive(Clone)]
    pub struct Lookup<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_lookup(&self) -> Lookup<'gc> {
            todo!()
        }
    }

    impl<'gc> Lookup<'gc> {
        pub fn trusted_lookup<'l>(jvm: &'gc JVMState<'gc>, _int_state: &mut impl PushableFrame<'gc>) -> Self {
            let lookup = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
            let static_vars = static_vars(lookup.deref(), jvm);
            static_vars.get(FieldName::field_IMPL_LOOKUP(), CPDType::object()).cast_lookup()
        }

        //noinspection DuplicatedCode
        pub fn find_virtual<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, obj: JClass<'gc>, name: JString<'gc>, mt: MethodType<'gc>) -> Result<MethodHandle<'gc>, WasException<'gc>> {
            let lookup_class = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
            let args = vec![self.new_java_value(), obj.new_java_value(), name.new_java_value(), mt.new_java_value()];
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::class().into(), CClassName::string().into(), CClassName::method_type().into()],
                return_type: CClassName::method_handle().into(),
            };
            let res = run_static_or_virtual(jvm, int_state, &lookup_class, MethodName::method_findVirtual(), &desc, args)?.unwrap();
            Ok(res.cast_method_handle())
        }

        //noinspection DuplicatedCode
        pub fn find_static<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, obj: JClass<'gc>, name: JString<'gc>, mt: MethodType<'gc>) -> Result<MethodHandle<'gc>, WasException<'gc>> {
            let lookup_class = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::class().into(), CClassName::string().into(), CClassName::method_type().into()],
                return_type: CClassName::method_handle().into(),
            };
            let args = vec![self.new_java_value(), obj.new_java_value(), name.new_java_value(), mt.new_java_value()];
            let res = run_static_or_virtual(jvm, int_state, &lookup_class, MethodName::method_findStatic(), &desc, args)?;
            Ok(res.unwrap().cast_method_handle())
        }

        pub fn find_special<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, obj: JClass<'gc>, name: JString<'gc>, mt: MethodType<'gc>, special_caller: JClass<'gc>) -> Result<MethodHandle<'gc>, WasException<'gc>> {
            let lookup_class = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::class().into(), CClassName::string().into(), CClassName::method_type().into(), CClassName::class().into()],
                return_type: CClassName::method_handle().into(),
            };
            let args = vec![self.new_java_value(), obj.new_java_value(), name.new_java_value(), mt.new_java_value(), special_caller.new_java_value()];
            let res = run_static_or_virtual(jvm, int_state, &lookup_class, MethodName::method_findSpecial(), &desc, args)?;
            Ok(res.unwrap().cast_method_handle())
        }

        pub fn find_constructor<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, obj: JClass<'gc>, mt: MethodType<'gc>) -> Result<MethodHandle<'gc>, WasException<'gc>> {
            let lookup_class = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::class().into(), CClassName::method_type().into()],
                return_type: CClassName::method_handle().into(),
            };
            let args = vec![self.new_java_value(), obj.new_java_value(), mt.new_java_value()];
            let res = run_static_or_virtual(jvm, int_state, &lookup_class, MethodName::method_findConstructor(), &desc, args)?;
            Ok(res.unwrap().cast_method_handle())
        }


    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Lookup<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}
