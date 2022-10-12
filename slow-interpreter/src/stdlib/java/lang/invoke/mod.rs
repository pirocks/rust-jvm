pub mod method_type {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;
    use runtime_class_stuff::RuntimeClass;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;

    use crate::{AllocatedHandle, JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::better_java_stack::java_stack_guard::JavaStackGuard;
    use crate::better_java_stack::opaque_frame::OpaqueFrame;
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_util::new_object;
    use crate::java_values::JavaValue;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::stdlib::java::lang::class::JClass;
    use crate::stdlib::java::lang::class_loader::ClassLoader;
    use crate::stdlib::java::lang::invoke::method_type_form::MethodTypeForm;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::NewAsObjectOrJavaValue;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct MethodType<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> MethodType<'gc> {
        pub fn from_method_descriptor_string<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, str: JString<'gc>, class_loader: Option<ClassLoader<'gc>>) -> Result<MethodType<'gc>, WasException<'gc>> {
            let method_type: Arc<RuntimeClass<'gc>> = assert_inited_or_initing_class(jvm, CClassName::method_type().into());
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::string().into(), CClassName::classloader().into()],
                return_type: CClassName::method_type().into(),
            };
            let res = run_static_or_virtual(
                jvm,
                int_state,
                &method_type,
                MethodName::method_fromMethodDescriptorString(),
                &desc,
                vec![str.new_java_value(), class_loader.as_ref().map(|x| x.new_java_value()).unwrap_or(NewJavaValue::Null)],
            )?.unwrap();
            Ok(res.cast_method_type())
        }

        pub fn set_rtype(&self, jvm: &'gc JVMState<'gc>, rtype: JClass<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_rtype(), rtype.new_java_value());
        }

        pub fn get_rtype_or_null<'k>(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
            Some(self.normal_object.get_var_top_level(jvm, FieldName::field_rtype()).unwrap_object()?.cast_class())
            /*if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.to_new().cast_class().into()
                } else {
                    None
                }
            } else {
                maybe_null.to_new().cast_class().into()
            }*/
        }
        pub fn get_rtype<'k>(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
            self.get_rtype_or_null(jvm).unwrap()
        }

        pub fn get_rtype_as_type(&self, jvm: &'gc JVMState<'gc>) -> CPDType {
            self.get_rtype(jvm).as_type(jvm)
        }

        pub fn set_ptypes<'irrelevant>(&self, jvm: &'gc JVMState<'gc>, ptypes: NewJavaValue<'gc, 'irrelevant>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_ptypes(), ptypes.as_njv());
        }

        pub fn get_ptypes_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<NewJavaValueHandle<'gc>> {
            Some(self.normal_object.get_var_top_level(jvm, FieldName::field_ptypes()).unwrap_object()?.new_java_value_handle())
            /*if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.clone().into()
                } else {
                    None
                }
            } else {
                maybe_null.clone().into()
            }*/
        }
        pub fn get_ptypes(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.get_ptypes_or_null(jvm).unwrap()
        }

        pub fn get_ptypes_as_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<CPDType> {
            self.get_ptypes(jvm).unwrap_object_nonnull().unwrap_array().array_iterator().map(|x| x.cast_class().unwrap().as_type(jvm)).collect()
        }

        pub fn set_form(&self, jvm: &'gc JVMState<'gc>, form: MethodTypeForm<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_form(), form.new_java_value());
        }

        pub fn get_form(&self, jvm: &'gc JVMState<'gc>) -> MethodTypeForm<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_form()).cast_method_type_form()
        }

        pub fn set_wrap_alt(&self, jvm: &'gc JVMState<'gc>, val: JavaValue<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_ptypes(), val.to_new());
        }

        pub fn set_invokers(&self, jvm: &'gc JVMState<'gc>, invokers: JavaValue<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_invokers(), invokers.to_new());
        }

        pub fn set_method_descriptors(&self, jvm: &'gc JVMState<'gc>, method_descriptor: JavaValue<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_methodDescriptor(), method_descriptor.to_new());
        }

        pub fn parameter_type<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, int: jint) -> Result<JClass<'gc>, WasException<'gc>> {
            let method_type = assert_inited_or_initing_class(jvm, CClassName::method_type().into());
            let desc = CMethodDescriptor { arg_types: vec![CPDType::IntType], return_type: CClassName::class().into() };
            let args = vec![self.new_java_value(), NewJavaValue::Int(int)];
            let res = run_static_or_virtual(jvm, int_state, &method_type, MethodName::method_parameterType(), &desc, args)?;
            Ok(res.unwrap().cast_class().unwrap())
        }

        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaStackGuard<'gc>, rtype: JClass<'gc>, ptypes: Vec<JClass<'gc>>, form: MethodTypeForm<'gc>, wrap_alt: JavaValue<'gc>, invokers: JavaValue<'gc>, method_descriptor: JavaValue<'gc>) -> MethodType<'gc> {
            let method_type: Arc<RuntimeClass<'gc>> = assert_inited_or_initing_class(jvm, CClassName::method_type().into());
            let mut temp: OpaqueFrame<'gc, '_> = todo!();
            let res_handle: AllocatedNormalObjectHandle<'gc> = new_object(jvm, &mut temp/*int_state*/, &method_type, false);
            let res = AllocatedHandle::NormalObject(res_handle).cast_method_type();
            let ptypes_arr_handle = jvm.allocate_object(todo!()/*Object::Array(ArrayObject {
                // elems: UnsafeCell::new(ptypes.into_iter().map(|x| x.java_value().to_native()).collect::<Vec<_>>()),
                whole_array_runtime_class: todo!(),
                loader: todo!(),
                len: todo!(),
                elems_base: todo!(),
                phantom_data: Default::default(),
                elem_type: CClassName::class().into(),
                // monitor: jvm.thread_state.new_monitor("".to_string()),
            })*/);
            let ptypes_arr = ptypes_arr_handle.new_java_value();
            res.set_ptypes(jvm, ptypes_arr);
            res.set_rtype(jvm, rtype);
            res.set_form(jvm, form);
            res.set_wrap_alt(jvm, wrap_alt);
            res.set_invokers(jvm, invokers);
            res.set_method_descriptors(jvm, method_descriptor);
            res
        }

        // as_object_or_java_value!();
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for MethodType<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod method_type_form {
    use jvmti_jni_bindings::jlong;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::field_names::FieldName;

    use crate::{AllocatedHandle, NewAsObjectOrJavaValue, NewJavaValue, pushable_frame_todo};
    use crate::better_java_stack::java_stack_guard::JavaStackGuard;
    use crate::better_java_stack::opaque_frame::OpaqueFrame;
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_util::new_object;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::stdlib::java::lang::invoke::method_type::MethodType;

    #[derive(Clone)]
    pub struct MethodTypeForm<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> MethodTypeForm<'gc> {
        pub fn set_arg_to_slot_table(&self, jvm: &'gc JVMState<'gc>, int_arr: NewJavaValue<'gc, '_>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_argToSlotTable(), int_arr);
        }

        pub fn set_slot_to_arg_table(&self, jvm: &'gc JVMState<'gc>, int_arr: NewJavaValue<'gc, '_>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_slotToArgTable(), int_arr);
        }

        pub fn set_arg_counts(&self, jvm: &'gc JVMState<'gc>, counts: jlong) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_argCounts(), NewJavaValue::Long(counts));
        }

        pub fn set_prim_counts(&self, jvm: &'gc JVMState<'gc>, counts: jlong) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_primCounts(), NewJavaValue::Long(counts));
        }

        pub fn set_erased_type(&self, jvm: &'gc JVMState<'gc>, type_: MethodType<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_erasedType(), type_.new_java_value());
        }

        pub fn set_basic_type(&self, jvm: &'gc JVMState<'gc>, type_: MethodType<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_basicType(), type_.new_java_value());
        }

        pub fn set_method_handles(&self, jvm: &'gc JVMState<'gc>, method_handle: NewJavaValue<'gc, '_>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_methodHandles(), method_handle);
        }

        pub fn set_lambda_forms(&self, jvm: &'gc JVMState<'gc>, lambda_forms: NewJavaValue<'gc, '_>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_methodHandles(), lambda_forms);
        }

        pub fn new<'l>(
            jvm: &'gc JVMState<'gc>,
            int_state: &mut JavaStackGuard<'gc>,
            arg_to_slot_table: NewJavaValue<'gc, '_>,
            slot_to_arg_table: NewJavaValue<'gc, '_>,
            arg_counts: jlong,
            prim_counts: jlong,
            erased_type: Option<MethodType<'gc>>,
            basic_type: Option<MethodType<'gc>>,
            method_handles: NewJavaValue<'gc, '_>,
            lambda_forms: NewJavaValue<'gc, '_>,
        ) -> MethodTypeForm<'gc> {
            let mut temp: OpaqueFrame<'gc, '_> = todo!();
            let method_type_form = assert_inited_or_initing_class(jvm, CClassName::method_type_form().into());
            let res_handle = AllocatedHandle::NormalObject(new_object(jvm, pushable_frame_todo()/*int_state*/, &method_type_form, false));
            let res = res_handle.cast_method_type_form();
            res.set_arg_to_slot_table(jvm, arg_to_slot_table);
            res.set_slot_to_arg_table(jvm, slot_to_arg_table);
            res.set_arg_counts(jvm, arg_counts);
            res.set_prim_counts(jvm, prim_counts);
            if let Some(x) = erased_type {
                res.set_erased_type(jvm, x);
            }
            if let Some(x) = basic_type {
                res.set_basic_type(jvm, x);
            }
            res.set_method_handles(jvm, method_handles);
            res.set_lambda_forms(jvm, lambda_forms);
            res
        }

        // as_object_or_java_value!();
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for MethodTypeForm<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}

pub mod method_handle {
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;

    use crate::{JVMState, NewAsObjectOrJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::java_values::JavaValue;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::stdlib::java::lang::invoke::lambda_form::LambdaForm;
    use crate::stdlib::java::lang::invoke::method_handles::lookup::Lookup;
    use crate::stdlib::java::lang::invoke::method_type::MethodType;
    use crate::stdlib::java::lang::member_name::MemberName;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct MethodHandle<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_method_handle(&self) -> MethodHandle<'gc> {
            todo!()
        }
    }

    impl<'gc> MethodHandle<'gc> {
        pub fn lookup<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Lookup<'gc>, WasException<'gc>> {
            let method_handles_class = assert_inited_or_initing_class(jvm, CClassName::method_handles().into());
            run_static_or_virtual(jvm, int_state, &method_handles_class, MethodName::method_lookup(), &CMethodDescriptor::empty_args(CClassName::method_handles_lookup().into()), todo!())?;
            Ok(todo!()/*int_state.pop_current_operand_stack(Some(CClassName::method_handles().into())).cast_lookup()*/)
        }
        pub fn public_lookup<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Lookup<'gc>, WasException<'gc>> {
            let method_handles_class = assert_inited_or_initing_class(jvm, CClassName::method_handles().into());
            run_static_or_virtual(jvm, int_state, &method_handles_class, MethodName::method_publicLookup(), &CMethodDescriptor::empty_args(CClassName::method_handles_lookup().into()), todo!())?;
            Ok(todo!()/*int_state.pop_current_operand_stack(Some(CClassName::method_handles().into())).cast_lookup()*/)
        }

        pub fn internal_member_name<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<MemberName<'gc>, WasException<'gc>> {
            let method_handle_class = assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
            let desc = CMethodDescriptor::empty_args(CClassName::member_name().into());
            let args = vec![self.new_java_value()];
            let res = run_static_or_virtual(jvm, int_state, &method_handle_class, MethodName::method_internalMemberName(), &desc, args)?;
            Ok(res.unwrap().cast_member_name())
        }

        pub fn type__(&self, jvm: &'gc JVMState<'gc>) -> MethodType<'gc> {
            let method_handle_class = assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
            self.normal_object.get_var(jvm, &method_handle_class, FieldName::field_type()).cast_method_type()
        }

        pub fn type_<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<MethodType<'gc>, WasException<'gc>> {
            /*let method_handle_class = assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(jvm, int_state, &method_handle_class, MethodName::method_type(), &CMethodDescriptor::empty_args(CClassName::method_type().into()), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::method_type().into())).cast_method_type())*/
            todo!()
        }

        pub fn get_form_or_null(&self, jvm: &'gc JVMState<'gc>) -> Result<Option<LambdaForm<'gc>>, WasException<'gc>> {
            let method_handle_class = assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
            let maybe_null = self.normal_object.get_var(jvm, &method_handle_class, FieldName::field_form());
            match maybe_null.unwrap_object() {
                Some(maybe_null) => Ok(Some(maybe_null.cast_lambda_form())),
                None => return Err(WasException { exception_obj: todo!() }),
            }
        }
        pub fn get_form(&self, jvm: &'gc JVMState<'gc>) -> Result<LambdaForm<'gc>, WasException<'gc>> {
            Ok(self.get_form_or_null(jvm)?.unwrap())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for MethodHandle<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod method_handles {
    pub mod lookup {
        use std::ops::Deref;
        use rust_jvm_common::compressed_classfile::class_names::CClassName;

        use rust_jvm_common::compressed_classfile::CMethodDescriptor;
        use rust_jvm_common::compressed_classfile::field_names::FieldName;
        use rust_jvm_common::compressed_classfile::method_names::MethodName;

        use crate::better_java_stack::frames::PushableFrame;
        use crate::class_loading::assert_inited_or_initing_class;
        use crate::java_values::JavaValue;
        use crate::jvm_state::JVMState;
        use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
        use crate::new_java_values::owned_casts::OwnedCastAble;
        use crate::runtime_class::static_vars;
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
                static_vars.get(FieldName::field_IMPL_LOOKUP()).cast_lookup()
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

            // as_object_or_java_value!();
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
}

pub mod lambda_form {
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::stdlib::java::lang::invoke::lambda_form::name::Name;
    use crate::stdlib::java::lang::member_name::MemberName;

    pub mod named_function {
        use crate::{NewAsObjectOrJavaValue, WasException};
        use crate::better_java_stack::frames::PushableFrame;
        use crate::java_values::JavaValue;
        use crate::jvm_state::JVMState;
        use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
        use crate::stdlib::java::lang::invoke::method_type::MethodType;
        use crate::stdlib::java::lang::member_name::MemberName;

        #[derive(Clone)]
        pub struct NamedFunction<'gc> {
            pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
        }

        impl<'gc> JavaValue<'gc> {
            pub fn cast_lambda_form_named_function(&self) -> NamedFunction<'gc> {
                todo!()
            }
        }

        impl<'gc> NamedFunction<'gc> {
            //noinspection DuplicatedCode
            pub fn get_member_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<MemberName<'gc>> {
                // let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_member());
                /*if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        todo!()/*maybe_null.cast_member_name().into()*/
                    } else {
                        None
                    }
                } else {
                    todo!()/*maybe_null.cast_member_name().into()*/
                }*/
                todo!()
            }
            pub fn get_member(&self, jvm: &'gc JVMState<'gc>) -> MemberName<'gc> {
                self.get_member_or_null(jvm).unwrap()
            }

            pub fn method_type<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<MethodType<'gc>, WasException<'gc>> {
                // java.lang.invoke.LambdaForm.NamedFunction
                /*let named_function_type = assert_inited_or_initing_class(jvm, CClassName::lambda_from_named_function().into());
                int_state.push_current_operand_stack(self.clone().java_value());
                run_static_or_virtual(jvm, int_state, &named_function_type, MethodName::method_methodType(), &CMethodDescriptor::empty_args(CClassName::method_type().into()), todo!())?;
                Ok(int_state.pop_current_operand_stack(Some(CClassName::method_type().into())).cast_method_type())*/
                todo!()
            }
        }

        impl<'gc> NewAsObjectOrJavaValue<'gc> for NamedFunction<'gc> {
            fn object(self) -> AllocatedNormalObjectHandle<'gc> {
                self.normal_object
            }

            fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
                &self.normal_object
            }
        }
    }

    pub mod name {
        use itertools::Itertools;

        use jvmti_jni_bindings::jint;
        use rust_jvm_common::compressed_classfile::field_names::FieldName;

        use crate::java_values::JavaValue;
        use crate::jvm_state::JVMState;
        use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
        use crate::NewJavaValueHandle;
        use crate::stdlib::java::lang::invoke::lambda_form::basic_type::BasicType;
        use crate::stdlib::java::lang::invoke::lambda_form::named_function::NamedFunction;

        #[derive(Clone)]
        pub struct Name<'gc> {
            pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
        }

        impl<'gc> JavaValue<'gc> {
            pub fn cast_lambda_form_name(&self) -> Name<'gc> {
                todo!()
            }
        }

        impl<'gc> Name<'gc> {
            pub fn arguments(&self, jvm: &'gc JVMState<'gc>) -> Vec<NewJavaValueHandle<'gc>> {
                self.normal_object.get_var_top_level(jvm, FieldName::field_arguments()).unwrap_object_nonnull().unwrap_array().array_iterator().collect_vec()
            }

            //noinspection DuplicatedCode
            pub fn get_index_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jint> {
                todo!()
                /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_index());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.unwrap_int().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.unwrap_int().into()
                }*/
            }
            pub fn get_index(&self, jvm: &'gc JVMState<'gc>) -> jint {
                self.get_index_or_null(jvm).unwrap()
            }
            pub fn get_type_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<BasicType<'gc>> {
                todo!()
                /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_type());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.cast_lambda_form_basic_type().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.cast_lambda_form_basic_type().into()
                }*/
            }
            pub fn get_type(&self, jvm: &'gc JVMState<'gc>) -> BasicType<'gc> {
                self.get_type_or_null(jvm).unwrap()
            }
            pub fn get_function_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<NamedFunction<'gc>> {
                todo!()
                /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_function());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.cast_lambda_form_named_function().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.cast_lambda_form_named_function().into()
                }*/
            }
            pub fn get_function(&self, jvm: &'gc JVMState<'gc>) -> NamedFunction<'gc> {
                self.get_function_or_null(jvm).unwrap()
            }
        }
    }

    pub mod basic_type {
        use jvmti_jni_bindings::jchar;
        use jvmti_jni_bindings::jint;

        use crate::{JString, NewAsObjectOrJavaValue};
        use crate::jvm_state::JVMState;
        use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
        use crate::stdlib::java::lang::class::JClass;

        #[derive(Clone)]
        pub struct BasicType<'gc> {
            pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
        }

        impl<'gc> BasicType<'gc> {
            //noinspection DuplicatedCode
            pub fn get_ordinal_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jint> {
                todo!()
                /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_ordinal());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.unwrap_int().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.unwrap_int().into()
                }*/
            }
            pub fn get_ordinal(&self, jvm: &'gc JVMState<'gc>) -> jint {
                self.get_ordinal_or_null(jvm).unwrap()
            }
            pub fn get_bt_char_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jchar> {
                todo!()
                /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_btChar());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.unwrap_char().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.unwrap_char().into()
                }*/
            }
            pub fn get_bt_char(&self, jvm: &'gc JVMState<'gc>) -> jchar {
                self.get_bt_char_or_null(jvm).unwrap()
            }

            //noinspection DuplicatedCode
            pub fn get_bt_class_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
                // let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_btClass());
                todo!()
                /*if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.to_new().cast_class().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.to_new().cast_class().into()
                }*/
            }
            pub fn get_bt_class(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
                self.get_bt_class_or_null(jvm).unwrap()
            }
            pub fn get_name_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JString<'gc>> {
                // let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_name());
                todo!()
                /*if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.cast_string().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.cast_string().into()
                }*/
            }
            pub fn get_name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
                self.get_name_or_null(jvm).unwrap()
            }
        }

        impl<'gc> NewAsObjectOrJavaValue<'gc> for BasicType<'gc> {
            fn object(self) -> AllocatedNormalObjectHandle<'gc> {
                self.normal_object
            }

            fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
                &self.normal_object
            }
        }
    }

    #[derive(Clone)]
    pub struct LambdaForm<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> LambdaForm<'gc> {
        pub fn names(&self, jvm: &'gc JVMState<'gc>) -> Vec<Name<'gc>> {
            todo!()
            // self.normal_object.get_var_top_level(jvm, FieldName::field_names()).unwrap_object_nonnull().unwrap_array().unwrap_object_array(jvm).iter().map(|name| JavaValue::Object(todo!() /*name.clone()*/).cast_lambda_form_name()).collect()
        }

        //noinspection DuplicatedCode
        pub fn get_vmentry_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<MemberName<'gc>> {
            Some(self.normal_object.get_var_top_level(jvm, FieldName::field_vmentry()).unwrap_object()?.cast_member_name())
            /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_vmentry());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    todo!()/*maybe_null.cast_member_name().into()*/
                } else {
                    None
                }
            } else {
                todo!()/*maybe_null.cast_member_name().into()*/
            }*/
        }
        pub fn get_vmentry(&self, jvm: &'gc JVMState<'gc>) -> MemberName<'gc> {
            self.get_vmentry_or_null(jvm).unwrap()
        }
    }
}

pub mod call_site {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;

    use crate::{NewAsObjectOrJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::common::invoke::virtual_::invoke_virtual;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::stdlib::java::lang::invoke::method_handle::MethodHandle;

    #[derive(Clone)]
    pub struct CallSite<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> CallSite<'gc> {
        pub fn get_target<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<MethodHandle<'gc>, WasException<'gc>> {
            let call_site_class = assert_inited_or_initing_class(jvm, CClassName::call_site().into());
            let args = vec![self.new_java_value()];
            let desc = CMethodDescriptor { arg_types: vec![], return_type: CPDType::Class(CClassName::method_handle()) };
            let res = invoke_virtual(jvm, int_state, MethodName::method_getTarget(), &desc, args)?;
            Ok(res.unwrap().cast_method_handle())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for CallSite<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}