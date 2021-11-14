pub mod method_type {
    use std::cell::UnsafeCell;
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_util::new_object;
    use crate::java::lang::class::JClass;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::invoke::method_type_form::MethodTypeForm;
    use crate::java::lang::string::JString;
    use crate::java_values::{ArrayObject, GcManagedObject, JavaValue, Object};
    use crate::runtime_class::RuntimeClass;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct MethodType<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_method_type(&self) -> MethodType<'gc_life> {
            MethodType {
                normal_object: self.unwrap_object_nonnull(),
            }
        }
    }

    impl<'gc_life> MethodType<'gc_life> {
        pub fn from_method_descriptor_string(
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
            str: JString<'gc_life>,
            class_loader: Option<ClassLoader<'gc_life>>,
        ) -> Result<MethodType<'gc_life>, WasException> {
            int_state.push_current_operand_stack(str.java_value());
            int_state.push_current_operand_stack(
                class_loader
                    .map(|x| x.java_value())
                    .unwrap_or(JavaValue::Object(None)),
            );
            let method_type: Arc<RuntimeClass<'gc_life>> =
                assert_inited_or_initing_class(jvm, CClassName::method_type().into());
            run_static_or_virtual(
                jvm,
                int_state,
                &method_type,
                MethodName::method_fromMethodDescriptorString(),
                &CMethodDescriptor {
                    arg_types: vec![
                        CClassName::string().into(),
                        CClassName::classloader().into(),
                    ],
                    return_type: CClassName::method_type().into(),
                },
                todo!(),
            )?;
            Ok(int_state
                .pop_current_operand_stack(Some(CClassName::method_type().into()))
                .cast_method_type())
        }

        pub fn set_rtype(&self, rtype: JClass<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_rtype(), rtype.java_value());
        }

        pub fn get_rtype_or_null(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
        ) -> Option<JClass<'gc_life>> {
            let maybe_null = self
                .normal_object
                .lookup_field(jvm, FieldName::field_rtype());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.cast_class().into()
                } else {
                    None
                }
            } else {
                maybe_null.cast_class().into()
            }
        }
        pub fn get_rtype(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.get_rtype_or_null(jvm).unwrap()
        }

        pub fn get_rtype_as_type(&self, jvm: &'gc_life JVMState<'gc_life>) -> CPDType {
            self.get_rtype(jvm).as_type(jvm)
        }

        pub fn set_ptypes(&self, ptypes: JavaValue<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_ptypes(), ptypes);
        }

        pub fn get_ptypes_or_null(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
        ) -> Option<JavaValue<'gc_life>> {
            let maybe_null = self
                .normal_object
                .lookup_field(jvm, FieldName::field_ptypes());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.clone().into()
                } else {
                    None
                }
            } else {
                maybe_null.clone().into()
            }
        }
        pub fn get_ptypes(&self, jvm: &'gc_life JVMState<'gc_life>) -> JavaValue<'gc_life> {
            self.get_ptypes_or_null(jvm).unwrap()
        }

        pub fn get_ptypes_as_types(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<CPDType> {
            self.get_ptypes(jvm)
                .unwrap_array()
                .unwrap_object_array(jvm)
                .iter()
                .map(|x| {
                    JavaValue::Object(x.clone())
                        .cast_class()
                        .unwrap()
                        .as_type(jvm)
                })
                .collect()
        }

        pub fn set_form(&self, jvm: &'gc_life JVMState<'gc_life>, form: MethodTypeForm<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_form(), form.java_value());
        }

        pub fn get_form(&self, jvm: &'gc_life JVMState<'gc_life>) -> MethodTypeForm<'gc_life> {
            self.normal_object
                .unwrap_normal_object()
                .get_var_top_level(jvm, FieldName::field_form())
                .cast_method_type_form()
        }

        pub fn set_wrap_alt(&self, jvm: &'gc_life JVMState<'gc_life>, val: JavaValue<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_ptypes(), val);
        }

        pub fn set_invokers(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
            invokers: JavaValue<'gc_life>,
        ) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_invokers(), invokers);
        }

        pub fn set_method_descriptors(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
            method_descriptor: JavaValue<'gc_life>,
        ) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_methodDescriptor(), method_descriptor);
        }

        pub fn parameter_type(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
            int: jint,
        ) -> Result<JClass<'gc_life>, WasException> {
            let method_type = assert_inited_or_initing_class(jvm, CClassName::method_type().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(JavaValue::Int(int));
            run_static_or_virtual(
                jvm,
                int_state,
                &method_type,
                MethodName::method_parameterType(),
                &CMethodDescriptor {
                    arg_types: vec![CPDType::IntType],
                    return_type: CClassName::class().into(),
                },
                todo!(),
            )?;
            Ok(int_state
                .pop_current_operand_stack(Some(CClassName::class().into()))
                .cast_class()
                .unwrap())
        }

        pub fn new(
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
            rtype: JClass<'gc_life>,
            ptypes: Vec<JClass<'gc_life>>,
            form: MethodTypeForm<'gc_life>,
            wrap_alt: JavaValue<'gc_life>,
            invokers: JavaValue<'gc_life>,
            method_descriptor: JavaValue<'gc_life>,
        ) -> MethodType<'gc_life> {
            let method_type = assert_inited_or_initing_class(jvm, CClassName::method_type().into());
            let res = new_object(jvm, int_state, &method_type).cast_method_type();
            let ptypes_arr =
                JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject {
                    // elems: UnsafeCell::new(ptypes.into_iter().map(|x| x.java_value().to_native()).collect::<Vec<_>>()),
                    whole_array_runtime_class: todo!(),
                    loader: todo!(),
                    len: todo!(),
                    elems: todo!(),
                    phantom_data: Default::default(),
                    elem_type: CClassName::class().into(),
                    // monitor: jvm.thread_state.new_monitor("".to_string()),
                }))));
            res.set_ptypes(ptypes_arr);
            res.set_rtype(rtype);
            res.set_form(jvm, form);
            res.set_wrap_alt(jvm, wrap_alt);
            res.set_invokers(jvm, invokers);
            res.set_method_descriptors(jvm, method_descriptor);
            res
        }

        as_object_or_java_value!();
    }
}

pub mod method_type_form {
    use jvmti_jni_bindings::jlong;
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::new_object;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    #[derive(Clone)]
    pub struct MethodTypeForm<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_method_type_form(&self) -> MethodTypeForm<'gc_life> {
            MethodTypeForm {
                normal_object: self.unwrap_object_nonnull(),
            }
        }
    }

    impl<'gc_life> MethodTypeForm<'gc_life> {
        pub fn set_arg_to_slot_table(&self, int_arr: JavaValue<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_argToSlotTable(), int_arr);
        }

        pub fn set_slot_to_arg_table(&self, int_arr: JavaValue<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_slotToArgTable(), int_arr);
        }

        pub fn set_arg_counts(&self, counts: jlong) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_argCounts(), JavaValue::Long(counts));
        }

        pub fn set_prim_counts(&self, counts: jlong) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_primCounts(), JavaValue::Long(counts));
        }

        pub fn set_erased_type(&self, type_: MethodType<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_erasedType(), type_.java_value());
        }

        pub fn set_basic_type(&self, type_: MethodType<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_basicType(), type_.java_value());
        }

        pub fn set_method_handles(&self, method_handle: JavaValue<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_methodHandles(), method_handle);
        }

        pub fn set_lambda_forms(&self, lambda_forms: JavaValue<'gc_life>) {
            self.normal_object
                .unwrap_normal_object()
                .set_var_top_level(FieldName::field_methodHandles(), lambda_forms);
        }

        pub fn new(
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
            arg_to_slot_table: JavaValue<'gc_life>,
            slot_to_arg_table: JavaValue<'gc_life>,
            arg_counts: jlong,
            prim_counts: jlong,
            erased_type: Option<MethodType<'gc_life>>,
            basic_type: Option<MethodType<'gc_life>>,
            method_handles: JavaValue<'gc_life>,
            lambda_forms: JavaValue<'gc_life>,
        ) -> MethodTypeForm<'gc_life> {
            let method_type_form =
                assert_inited_or_initing_class(jvm, CClassName::method_type_form().into());
            let res = new_object(jvm, int_state, &method_type_form).cast_method_type_form();
            res.set_arg_to_slot_table(arg_to_slot_table);
            res.set_slot_to_arg_table(slot_to_arg_table);
            res.set_arg_counts(arg_counts);
            res.set_prim_counts(prim_counts);
            if let Some(x) = erased_type {
                res.set_erased_type(x);
            }
            if let Some(x) = basic_type {
                res.set_basic_type(x);
            }
            res.set_method_handles(method_handles);
            res.set_lambda_forms(lambda_forms);
            res
        }

        as_object_or_java_value!();
    }
}

pub mod method_handle {
    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::java::lang::invoke::lambda_form::LambdaForm;
    use crate::java::lang::invoke::method_handles::lookup::Lookup;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::member_name::MemberName;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct MethodHandle<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_method_handle(&self) -> MethodHandle<'gc_life> {
            MethodHandle {
                normal_object: self.unwrap_object_nonnull(),
            }
        }
    }

    impl<'gc_life> MethodHandle<'gc_life> {
        pub fn lookup(
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
        ) -> Result<Lookup<'gc_life>, WasException> {
            let method_handles_class =
                assert_inited_or_initing_class(jvm, CClassName::method_handles().into());
            run_static_or_virtual(
                jvm,
                int_state,
                &method_handles_class,
                MethodName::method_lookup(),
                &CMethodDescriptor::empty_args(CClassName::method_handles_lookup().into()),
                todo!(),
            )?;
            Ok(int_state
                .pop_current_operand_stack(Some(CClassName::method_handles().into()))
                .cast_lookup())
        }
        pub fn public_lookup(
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
        ) -> Result<Lookup<'gc_life>, WasException> {
            let method_handles_class =
                assert_inited_or_initing_class(jvm, CClassName::method_handles().into());
            run_static_or_virtual(
                jvm,
                int_state,
                &method_handles_class,
                MethodName::method_publicLookup(),
                &CMethodDescriptor::empty_args(CClassName::method_handles_lookup().into()),
                todo!(),
            )?;
            Ok(int_state
                .pop_current_operand_stack(Some(CClassName::method_handles().into()))
                .cast_lookup())
        }

        pub fn internal_member_name(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
        ) -> Result<MemberName<'gc_life>, WasException> {
            let method_handle_class =
                assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(
                jvm,
                int_state,
                &method_handle_class,
                MethodName::method_internalMemberName(),
                &CMethodDescriptor::empty_args(CClassName::member_name().into()),
                todo!(),
            )?;
            Ok(int_state
                .pop_current_operand_stack(Some(CClassName::method_handle().into()))
                .cast_member_name())
        }

        pub fn type__(&self, jvm: &'gc_life JVMState<'gc_life>) -> MethodType<'gc_life> {
            let method_handle_class =
                assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
            self.normal_object
                .unwrap_normal_object()
                .get_var(jvm, method_handle_class, FieldName::field_type())
                .cast_method_type()
        }

        pub fn type_(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
        ) -> Result<MethodType<'gc_life>, WasException> {
            let method_handle_class =
                assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(
                jvm,
                int_state,
                &method_handle_class,
                MethodName::method_type(),
                &CMethodDescriptor::empty_args(CClassName::method_type().into()),
                todo!(),
            )?;
            Ok(int_state
                .pop_current_operand_stack(Some(CClassName::method_type().into()))
                .cast_method_type())
        }

        pub fn get_form_or_null(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
        ) -> Result<Option<LambdaForm<'gc_life>>, WasException> {
            let method_handle_class =
                assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
            dbg!(self
                .normal_object
                .unwrap_normal_object()
                .objinfo
                .class_pointer
                .view()
                .name()
                .unwrap_object_name()
                .0
                .to_str(&jvm.string_pool));
            let maybe_null = self.normal_object.unwrap_normal_object().get_var(
                jvm,
                method_handle_class,
                FieldName::field_form(),
            ); //.lookup_field(jvm, FieldName::field_form());
            Ok(if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.cast_lambda_form().into()
                } else {
                    None
                }
            } else {
                maybe_null.cast_lambda_form().into()
            })
        }
        pub fn get_form(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
        ) -> Result<LambdaForm<'gc_life>, WasException> {
            Ok(self.get_form_or_null(jvm)?.unwrap())
        }

        as_object_or_java_value!();
    }
}

pub mod method_handles {
    pub mod lookup {
        use rust_jvm_common::compressed_classfile::CMethodDescriptor;
        use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

        use crate::class_loading::assert_inited_or_initing_class;
        use crate::interpreter::WasException;
        use crate::interpreter_state::InterpreterStateGuard;
        use crate::java::lang::class::JClass;
        use crate::java::lang::invoke::method_handle::MethodHandle;
        use crate::java::lang::invoke::method_type::MethodType;
        use crate::java::lang::string::JString;
        use crate::java_values::{GcManagedObject, JavaValue};
        use crate::jvm_state::JVMState;
        use crate::utils::run_static_or_virtual;

        #[derive(Clone)]
        pub struct Lookup<'gc_life> {
            normal_object: GcManagedObject<'gc_life>,
        }

        impl<'gc_life> JavaValue<'gc_life> {
            pub fn cast_lookup(&self) -> Lookup<'gc_life> {
                Lookup {
                    normal_object: self.unwrap_object_nonnull(),
                }
            }
        }

        impl<'gc_life> Lookup<'gc_life> {
            pub fn trusted_lookup(
                jvm: &'gc_life JVMState<'gc_life>,
                _int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
            ) -> Self {
                let lookup = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
                let static_vars = lookup.static_vars();
                static_vars
                    .get(&FieldName::field_IMPL_LOOKUP())
                    .unwrap()
                    .cast_lookup()
            }

            pub fn find_virtual(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
                int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
                obj: JClass<'gc_life>,
                name: JString<'gc_life>,
                mt: MethodType<'gc_life>,
            ) -> Result<MethodHandle<'gc_life>, WasException> {
                let lookup_class = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
                int_state.push_current_operand_stack(self.clone().java_value());
                int_state.push_current_operand_stack(obj.java_value());
                int_state.push_current_operand_stack(name.java_value());
                int_state.push_current_operand_stack(mt.java_value());
                let desc = CMethodDescriptor {
                    arg_types: vec![
                        CClassName::class().into(),
                        CClassName::string().into(),
                        CClassName::method_type().into(),
                    ],
                    return_type: CClassName::method_handle().into(),
                };
                run_static_or_virtual(
                    jvm,
                    int_state,
                    &lookup_class,
                    MethodName::method_findVirtual(),
                    &desc,
                    todo!(),
                )?;
                Ok(int_state
                    .pop_current_operand_stack(Some(CClassName::lookup().into()))
                    .cast_method_handle())
            }

            pub fn find_static(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
                int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
                obj: JClass<'gc_life>,
                name: JString<'gc_life>,
                mt: MethodType<'gc_life>,
            ) -> Result<MethodHandle<'gc_life>, WasException> {
                let lookup_class = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
                int_state.push_current_operand_stack(self.clone().java_value());
                int_state.push_current_operand_stack(obj.java_value());
                int_state.push_current_operand_stack(name.java_value());
                int_state.push_current_operand_stack(mt.java_value());
                let desc = CMethodDescriptor {
                    arg_types: vec![
                        CClassName::class().into(),
                        CClassName::string().into(),
                        CClassName::method_type().into(),
                    ],
                    return_type: CClassName::method_handle().into(),
                };
                run_static_or_virtual(
                    jvm,
                    int_state,
                    &lookup_class,
                    MethodName::method_findStatic(),
                    &desc,
                    todo!(),
                )?;
                Ok(int_state
                    .pop_current_operand_stack(Some(CClassName::lookup().into()))
                    .cast_method_handle())
            }

            pub fn find_special(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
                int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
                obj: JClass<'gc_life>,
                name: JString<'gc_life>,
                mt: MethodType<'gc_life>,
                special_caller: JClass<'gc_life>,
            ) -> Result<MethodHandle<'gc_life>, WasException> {
                let lookup_class = assert_inited_or_initing_class(jvm, CClassName::lookup().into());
                int_state.push_current_operand_stack(self.clone().java_value());
                int_state.push_current_operand_stack(obj.java_value());
                int_state.push_current_operand_stack(name.java_value());
                int_state.push_current_operand_stack(mt.java_value());
                int_state.push_current_operand_stack(special_caller.java_value());
                let desc = CMethodDescriptor {
                    arg_types: vec![
                        CClassName::class().into(),
                        CClassName::string().into(),
                        CClassName::method_type().into(),
                        CClassName::class().into(),
                    ],
                    return_type: CClassName::method_handle().into(),
                };
                run_static_or_virtual(
                    jvm,
                    int_state,
                    &lookup_class,
                    MethodName::method_findSpecial(),
                    &desc,
                    todo!(),
                )?;
                Ok(int_state
                    .pop_current_operand_stack(Some(CClassName::lookup().into()))
                    .cast_method_handle())
            }

            as_object_or_java_value!();
        }
    }
}

pub mod lambda_form {
    use rust_jvm_common::compressed_classfile::names::FieldName;

    use crate::java::lang::invoke::lambda_form::name::Name;
    use crate::java::lang::member_name::MemberName;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub mod named_function {
        use rust_jvm_common::compressed_classfile::CMethodDescriptor;
        use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

        use crate::class_loading::assert_inited_or_initing_class;
        use crate::interpreter::WasException;
        use crate::interpreter_state::InterpreterStateGuard;
        use crate::java::lang::invoke::method_type::MethodType;
        use crate::java::lang::member_name::MemberName;
        use crate::java_values::{GcManagedObject, JavaValue};
        use crate::jvm_state::JVMState;
        use crate::utils::run_static_or_virtual;

        #[derive(Clone)]
        pub struct NamedFunction<'gc_life> {
            normal_object: GcManagedObject<'gc_life>,
        }

        impl<'gc_life> JavaValue<'gc_life> {
            pub fn cast_lambda_form_named_function(&self) -> NamedFunction<'gc_life> {
                NamedFunction {
                    normal_object: self.unwrap_object_nonnull(),
                }
            }
        }

        impl<'gc_life> NamedFunction<'gc_life> {
            as_object_or_java_value!();

            pub fn get_member_or_null(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
            ) -> Option<MemberName<'gc_life>> {
                let maybe_null = self
                    .normal_object
                    .lookup_field(jvm, FieldName::field_member());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.cast_member_name().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.cast_member_name().into()
                }
            }
            pub fn get_member(&self, jvm: &'gc_life JVMState<'gc_life>) -> MemberName<'gc_life> {
                self.get_member_or_null(jvm).unwrap()
            }

            pub fn method_type(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
                int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
            ) -> Result<MethodType<'gc_life>, WasException> {
                // java.lang.invoke.LambdaForm.NamedFunction
                let named_function_type = assert_inited_or_initing_class(
                    jvm,
                    CClassName::lambda_from_named_function().into(),
                );
                int_state.push_current_operand_stack(self.clone().java_value());
                run_static_or_virtual(
                    jvm,
                    int_state,
                    &named_function_type,
                    MethodName::method_methodType(),
                    &CMethodDescriptor::empty_args(CClassName::method_type().into()),
                    todo!(),
                )?;
                Ok(int_state
                    .pop_current_operand_stack(Some(CClassName::method_type().into()))
                    .cast_method_type())
            }
        }
    }

    pub mod name {
        use itertools::Itertools;

        use jvmti_jni_bindings::jint;
        use rust_jvm_common::compressed_classfile::names::FieldName;

        use crate::java::lang::invoke::lambda_form::basic_type::BasicType;
        use crate::java::lang::invoke::lambda_form::named_function::NamedFunction;
        use crate::java_values::{GcManagedObject, JavaValue};
        use crate::jvm_state::JVMState;

        #[derive(Clone)]
        pub struct Name<'gc_life> {
            normal_object: GcManagedObject<'gc_life>,
        }

        impl<'gc_life> JavaValue<'gc_life> {
            pub fn cast_lambda_form_name(&self) -> Name<'gc_life> {
                Name {
                    normal_object: self.unwrap_object_nonnull(),
                }
            }
        }

        impl<'gc_life> Name<'gc_life> {
            as_object_or_java_value!();
            pub fn arguments(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<JavaValue<'gc_life>> {
                self.normal_object
                    .unwrap_normal_object()
                    .get_var_top_level(jvm, FieldName::field_arguments())
                    .unwrap_array()
                    .array_iterator(jvm)
                    .collect_vec()
            }

            pub fn get_index_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<jint> {
                let maybe_null = self
                    .normal_object
                    .lookup_field(jvm, FieldName::field_index());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.unwrap_int().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.unwrap_int().into()
                }
            }
            pub fn get_index(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
                self.get_index_or_null(jvm).unwrap()
            }
            pub fn get_type_or_null(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
            ) -> Option<BasicType<'gc_life>> {
                let maybe_null = self
                    .normal_object
                    .lookup_field(jvm, FieldName::field_type());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.cast_lambda_form_basic_type().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.cast_lambda_form_basic_type().into()
                }
            }
            pub fn get_type(&self, jvm: &'gc_life JVMState<'gc_life>) -> BasicType<'gc_life> {
                self.get_type_or_null(jvm).unwrap()
            }
            pub fn get_function_or_null(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
            ) -> Option<NamedFunction<'gc_life>> {
                let maybe_null = self
                    .normal_object
                    .lookup_field(jvm, FieldName::field_function());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.cast_lambda_form_named_function().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.cast_lambda_form_named_function().into()
                }
            }
            pub fn get_function(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
            ) -> NamedFunction<'gc_life> {
                self.get_function_or_null(jvm).unwrap()
            }
        }
    }

    pub mod basic_type {
        use jvmti_jni_bindings::jchar;
        use jvmti_jni_bindings::jint;
        use rust_jvm_common::compressed_classfile::names::FieldName;

        use crate::java::lang::class::JClass;
        use crate::java_values::{GcManagedObject, JavaValue};
        use crate::JString;
        use crate::jvm_state::JVMState;

        #[derive(Clone)]
        pub struct BasicType<'gc_life> {
            normal_object: GcManagedObject<'gc_life>,
        }

        impl<'gc_life> JavaValue<'gc_life> {
            pub fn cast_lambda_form_basic_type(&self) -> BasicType<'gc_life> {
                BasicType {
                    normal_object: self.unwrap_object_nonnull(),
                }
            }
        }

        impl<'gc_life> BasicType<'gc_life> {
            as_object_or_java_value!();

            pub fn get_ordinal_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<jint> {
                let maybe_null = self
                    .normal_object
                    .lookup_field(jvm, FieldName::field_ordinal());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.unwrap_int().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.unwrap_int().into()
                }
            }
            pub fn get_ordinal(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
                self.get_ordinal_or_null(jvm).unwrap()
            }
            pub fn get_bt_char_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<jchar> {
                let maybe_null = self
                    .normal_object
                    .lookup_field(jvm, FieldName::field_btChar());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.unwrap_char().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.unwrap_char().into()
                }
            }
            pub fn get_bt_char(&self, jvm: &'gc_life JVMState<'gc_life>) -> jchar {
                self.get_bt_char_or_null(jvm).unwrap()
            }
            pub fn get_bt_class_or_null(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
            ) -> Option<JClass<'gc_life>> {
                let maybe_null = self
                    .normal_object
                    .lookup_field(jvm, FieldName::field_btClass());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.cast_class().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.cast_class().into()
                }
            }
            pub fn get_bt_class(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
                self.get_bt_class_or_null(jvm).unwrap()
            }
            pub fn get_name_or_null(
                &self,
                jvm: &'gc_life JVMState<'gc_life>,
            ) -> Option<JString<'gc_life>> {
                let maybe_null = self
                    .normal_object
                    .lookup_field(jvm, FieldName::field_name());
                if maybe_null.try_unwrap_object().is_some() {
                    if maybe_null.unwrap_object().is_some() {
                        maybe_null.cast_string().into()
                    } else {
                        None
                    }
                } else {
                    maybe_null.cast_string().into()
                }
            }
            pub fn get_name(&self, jvm: &'gc_life JVMState<'gc_life>) -> JString<'gc_life> {
                self.get_name_or_null(jvm).unwrap()
            }
        }
    }

    #[derive(Clone)]
    pub struct LambdaForm<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_lambda_form(&self) -> LambdaForm<'gc_life> {
            LambdaForm {
                normal_object: self.unwrap_object_nonnull(),
            }
        }
    }

    impl<'gc_life> LambdaForm<'gc_life> {
        pub fn names(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<Name<'gc_life>> {
            self.normal_object
                .unwrap_normal_object()
                .get_var_top_level(jvm, FieldName::field_names())
                .unwrap_array()
                .unwrap_object_array(jvm)
                .iter()
                .map(|name| JavaValue::Object(todo!() /*name.clone()*/).cast_lambda_form_name())
                .collect()
        }

        pub fn get_vmentry_or_null(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
        ) -> Option<MemberName<'gc_life>> {
            let maybe_null = self
                .normal_object
                .lookup_field(jvm, FieldName::field_vmentry());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.cast_member_name().into()
                } else {
                    None
                }
            } else {
                maybe_null.cast_member_name().into()
            }
        }
        pub fn get_vmentry(&self, jvm: &'gc_life JVMState<'gc_life>) -> MemberName<'gc_life> {
            self.get_vmentry_or_null(jvm).unwrap()
        }

        as_object_or_java_value!();
    }
}

pub mod call_site {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::instructions::invoke::virtual_::invoke_virtual;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::invoke::method_handle::MethodHandle;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    #[derive(Clone)]
    pub struct CallSite<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_call_site(&self) -> CallSite<'gc_life> {
            CallSite {
                normal_object: self.unwrap_object_nonnull(),
            } //todo every cast is an implicit npe
        }
    }

    impl<'gc_life> CallSite<'gc_life> {
        pub fn get_target(
            &self,
            jvm: &'gc_life JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
        ) -> Result<MethodHandle<'gc_life>, WasException> {
            let _call_site_class =
                assert_inited_or_initing_class(jvm, CClassName::call_site().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            let desc = CMethodDescriptor {
                arg_types: vec![],
                return_type: CPDType::Ref(CPRefType::Class(CClassName::method_handle())),
            };
            invoke_virtual(jvm, int_state, MethodName::method_getTarget(), &desc)?;
            Ok(int_state
                .pop_current_operand_stack(Some(CClassName::object().into()))
                .cast_method_handle())
        }

        as_object_or_java_value!();
    }
}