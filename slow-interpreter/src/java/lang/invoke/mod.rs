pub mod method_type {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;
    use rust_jvm_common::ptype::PType;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_util::push_new_object;
    use crate::java::lang::class::JClass;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::invoke::method_type_form::MethodTypeForm;
    use crate::java::lang::string::JString;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::runtime_class::RuntimeClass;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct MethodType<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_method_type(&self) -> MethodType<'gc_life> {
            MethodType { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> MethodType<'gc_life> {
        pub fn from_method_descriptor_string(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, str: JString<'gc_life>, class_loader: Option<ClassLoader<'gc_life>>) -> Result<MethodType<'gc_life>, WasException> {
            int_state.push_current_operand_stack(str.java_value());
            int_state.push_current_operand_stack(class_loader.map(|x| x.java_value()).unwrap_or(JavaValue::Object(todo!()/*None*/)));
            let method_type: Arc<RuntimeClass<'gc_life>> = assert_inited_or_initing_class(jvm, ClassName::method_type().into());
            run_static_or_virtual(jvm, int_state, &method_type, "fromMethodDescriptorString".to_string(), "(Ljava/lang/String;Ljava/lang/ClassLoader;)Ljava/lang/invoke/MethodType;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::method_type().into()).cast_method_type())
        }

        pub fn set_rtype(&self, rtype: JClass<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("rtype".to_string(), rtype.java_value());
        }

        pub fn get_rtype_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<JClass<'gc_life>> {
            let maybe_null = self.normal_object.lookup_field(jvm, "rtype");
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
        pub fn get_rtype(&self, jvm: &JVMState<'gc_life>) -> JClass<'gc_life> {
            self.get_rtype_or_null(jvm).unwrap()
        }

        pub fn get_rtype_as_type(&self, jvm: &'_ JVMState<'gc_life>) -> PType {
            self.get_rtype(jvm).as_type(jvm).to_ptype()
        }

        pub fn set_ptypes(&self, ptypes: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("ptypes".to_string(), ptypes);
        }

        pub fn get_ptypes_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<JavaValue<'gc_life>> {
            let maybe_null = self.normal_object.lookup_field(jvm, "ptypes");
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
        pub fn get_ptypes(&self, jvm: &JVMState<'gc_life>) -> JavaValue<'gc_life> {
            self.get_ptypes_or_null(jvm).unwrap()
        }

        pub fn get_ptypes_as_types(&self, jvm: &'_ JVMState<'gc_life>) -> Vec<PType> {
            self.get_ptypes(jvm).unwrap_array().unwrap_object_array(jvm).iter()
                .map(|x| JavaValue::Object(todo!()/*x.clone()*/).cast_class().unwrap().as_type(jvm).to_ptype()).collect()
        }

        pub fn set_form(&self, jvm: &JVMState<'gc_life>, form: MethodTypeForm<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("form".to_string(), form.java_value());
        }

        pub fn get_form(&self, jvm: &JVMState<'gc_life>) -> MethodTypeForm<'gc_life> {
            self.normal_object.unwrap_normal_object().get_var_top_level(jvm, "form").cast_method_type_form()
        }

        pub fn set_wrap_alt(&self, jvm: &JVMState<'gc_life>, val: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("ptypes".to_string(), val);
        }

        pub fn set_invokers(&self, jvm: &JVMState<'gc_life>, invokers: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("invokers".to_string(), invokers);
        }

        pub fn set_method_descriptors(&self, jvm: &JVMState<'gc_life>, method_descriptor: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("methodDescriptor".to_string(), method_descriptor);
        }

        pub fn parameter_type(&self, jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, int: jint) -> Result<JClass<'gc_life>, WasException> {
            let method_type = assert_inited_or_initing_class(jvm, ClassName::method_type().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(JavaValue::Int(int));
            run_static_or_virtual(jvm, int_state, &method_type, "parameterType".to_string(), "(I)Ljava/lang/Class;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::class().into()).cast_class().unwrap())
        }

        pub fn new(
            jvm: &'_ JVMState<'gc_life>,
            int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
            rtype: JClass<'gc_life>,
            ptypes: Vec<JClass>,
            form: MethodTypeForm<'gc_life>,
            wrap_alt: JavaValue<'gc_life>,
            invokers: JavaValue<'gc_life>,
            method_descriptor: JavaValue<'gc_life>,
        ) -> MethodType<'gc_life> {
            let method_type = assert_inited_or_initing_class(jvm, ClassName::method_type().into());
            push_new_object(jvm, int_state, &method_type);
            let res = int_state.pop_current_operand_stack(ClassName::method_type().into()).cast_method_type();
            let ptypes_arr = JavaValue::Object(todo!()/*Some(Arc::new(
                Object::Array(ArrayObject {
                    elems: UnsafeCell::new(ptypes.into_iter().map(|x| x.java_value()).collect::<Vec<_>>()),
                    elem_type: PTypeView::Ref(ReferenceTypeView::Class(ClassName::class())),
                    monitor: jvm.thread_state.new_monitor("".to_string()),
                })))*/);
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
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::push_new_object;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    #[derive(Clone)]
    pub struct MethodTypeForm<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_method_type_form(&self) -> MethodTypeForm<'gc_life> {
            MethodTypeForm { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> MethodTypeForm<'gc_life> {
        pub fn set_arg_to_slot_table(&self, int_arr: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("argToSlotTable".to_string(), int_arr);
        }

        pub fn set_slot_to_arg_table(&self, int_arr: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("slotToArgTable".to_string(), int_arr);
        }

        pub fn set_arg_counts(&self, counts: jlong) {
            self.normal_object.unwrap_normal_object().set_var_top_level("argCounts".to_string(), JavaValue::Long(counts));
        }

        pub fn set_prim_counts(&self, counts: jlong) {
            self.normal_object.unwrap_normal_object().set_var_top_level("primCounts".to_string(), JavaValue::Long(counts));
        }

        pub fn set_erased_type(&self, type_: MethodType<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("erasedType".to_string(), type_.java_value());
        }

        pub fn set_basic_type(&self, type_: MethodType<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("basicType".to_string(), type_.java_value());
        }

        pub fn set_method_handles(&self, method_handle: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("methodHandles".to_string(), method_handle);
        }

        pub fn set_lambda_forms(&self, lambda_forms: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("methodHandles".to_string(), lambda_forms);
        }

        pub fn new(jvm: &'_ JVMState<'gc_life>,
                   int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
                   arg_to_slot_table: JavaValue<'gc_life>,
                   slot_to_arg_table: JavaValue<'gc_life>,
                   arg_counts: jlong,
                   prim_counts: jlong,
                   erased_type: Option<MethodType<'gc_life>>,
                   basic_type: Option<MethodType<'gc_life>>,
                   method_handles: JavaValue<'gc_life>,
                   lambda_forms: JavaValue<'gc_life>) -> MethodTypeForm<'gc_life> {
            let method_type_form = assert_inited_or_initing_class(jvm, ClassName::method_type_form().into());
            push_new_object(jvm, int_state, &method_type_form);
            let res = int_state.pop_current_operand_stack(ClassName::method_type_form().into()).cast_method_type_form();
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
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::java::lang::invoke::lambda_form::LambdaForm;
    use crate::java::lang::invoke::method_handles::lookup::Lookup;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::member_name::MemberName;
    use crate::java_values::{GcManagedObject, JavaValue, Object};
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct MethodHandle<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_method_handle(&self) -> MethodHandle<'gc_life> {
            MethodHandle { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> MethodHandle<'gc_life> {
        pub fn lookup(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<Lookup<'gc_life>, WasException> {
            let method_handles_class = assert_inited_or_initing_class(jvm, ClassName::method_handles().into());
            run_static_or_virtual(jvm, int_state, &method_handles_class, "lookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::method_handles().into()).cast_lookup())
        }
        pub fn public_lookup(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<Lookup<'gc_life>, WasException> {
            let method_handles_class = assert_inited_or_initing_class(jvm, ClassName::method_handles().into());
            run_static_or_virtual(jvm, int_state, &method_handles_class, "publicLookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::method_handles().into()).cast_lookup())
        }

        pub fn internal_member_name(&self, jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<MemberName<'gc_life>, WasException> {
            let method_handle_class = assert_inited_or_initing_class(jvm, ClassName::method_handle().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(jvm, int_state, &method_handle_class, "internalMemberName".to_string(), "()Ljava/lang/invoke/MemberName;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::method_handle().into()).cast_member_name())
        }

        pub fn type__(&self, jvm: &JVMState<'gc_life>) -> MethodType<'gc_life> {
            self.normal_object.lookup_field(jvm, "type").cast_method_type()
        }

        pub fn type_(&self, jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<MethodType<'gc_life>, WasException> {
            let method_handle_class = assert_inited_or_initing_class(jvm, ClassName::method_type().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(jvm, int_state, &method_handle_class, "type".to_string(), "()Ljava/lang/invoke/MethodType;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::method_type().into()).cast_method_type())
        }


        pub fn get_form_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<LambdaForm<'gc_life>> {
            let maybe_null = self.normal_object.lookup_field(jvm, "form");
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.cast_lambda_form().into()
                } else {
                    None
                }
            } else {
                maybe_null.cast_lambda_form().into()
            }
        }
        pub fn get_form(&self, jvm: &JVMState<'gc_life>) -> LambdaForm<'gc_life> {
            self.get_form_or_null(jvm).unwrap()
        }


        as_object_or_java_value!();
    }
}

pub mod method_handles {
    pub mod lookup {
        use rust_jvm_common::classnames::ClassName;

        use crate::class_loading::assert_inited_or_initing_class;
        use crate::interpreter::WasException;
        use crate::interpreter_state::InterpreterStateGuard;
        use crate::java::lang::class::JClass;
        use crate::java::lang::invoke::method_handle::MethodHandle;
        use crate::java::lang::invoke::method_type::MethodType;
        use crate::java::lang::string::JString;
        use crate::java_values::{GcManagedObject, JavaValue, Object};
        use crate::jvm_state::JVMState;
        use crate::utils::run_static_or_virtual;

        #[derive(Clone)]
        pub struct Lookup<'gc_life> {
            normal_object: GcManagedObject<'gc_life>,
        }

        impl<'gc_life> JavaValue<'gc_life> {
            pub fn cast_lookup(&self) -> Lookup<'gc_life> {
                Lookup { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl<'gc_life> Lookup<'gc_life> {
            pub fn trusted_lookup(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Self {
                let lookup = assert_inited_or_initing_class(jvm, ClassName::lookup().into());
                let static_vars = lookup.static_vars();
                static_vars.get("IMPL_LOOKUP").unwrap().cast_lookup()
            }

            pub fn find_virtual(&self, jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, obj: JClass<'gc_life>, name: JString<'gc_life>, mt: MethodType<'gc_life>) -> Result<MethodHandle<'gc_life>, WasException> {
                let lookup_class = assert_inited_or_initing_class(jvm, ClassName::lookup().into());
                int_state.push_current_operand_stack(self.clone().java_value());
                int_state.push_current_operand_stack(obj.java_value());
                int_state.push_current_operand_stack(name.java_value());
                int_state.push_current_operand_stack(mt.java_value());
                run_static_or_virtual(jvm, int_state, &lookup_class, "findVirtual".to_string(), "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;".to_string())?;
                Ok(int_state.pop_current_operand_stack(ClassName::lookup().into()).cast_method_handle())
            }


            pub fn find_static(&self, jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, obj: JClass<'gc_life>, name: JString<'gc_life>, mt: MethodType<'gc_life>) -> Result<MethodHandle<'gc_life>, WasException> {
                let lookup_class = assert_inited_or_initing_class(jvm, ClassName::lookup().into());
                int_state.push_current_operand_stack(self.clone().java_value());
                int_state.push_current_operand_stack(obj.java_value());
                int_state.push_current_operand_stack(name.java_value());
                int_state.push_current_operand_stack(mt.java_value());
                run_static_or_virtual(jvm, int_state, &lookup_class, "findStatic".to_string(), "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;".to_string())?;
                Ok(int_state.pop_current_operand_stack(ClassName::lookup().into()).cast_method_handle())
            }

            pub fn find_special(&self, jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, obj: JClass<'gc_life>, name: JString<'gc_life>, mt: MethodType<'gc_life>, special_caller: JClass<'gc_life>) -> Result<MethodHandle<'gc_life>, WasException> {
                let lookup_class = assert_inited_or_initing_class(jvm, ClassName::lookup().into());
                int_state.push_current_operand_stack(self.clone().java_value());
                int_state.push_current_operand_stack(obj.java_value());
                int_state.push_current_operand_stack(name.java_value());
                int_state.push_current_operand_stack(mt.java_value());
                int_state.push_current_operand_stack(special_caller.java_value());
                run_static_or_virtual(jvm, int_state, &lookup_class, "findSpecial".to_string(), "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;".to_string())?;
                Ok(int_state.pop_current_operand_stack(ClassName::lookup().into()).cast_method_handle())
            }

            as_object_or_java_value!();
        }
    }
}

pub mod lambda_form {
    use crate::java::lang::invoke::lambda_form::name::Name;
    use crate::java::lang::member_name::MemberName;
    use crate::java_values::{GcManagedObject, JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub mod named_function {
        use rust_jvm_common::classnames::ClassName;

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
                NamedFunction { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl<'gc_life> NamedFunction<'gc_life> {
            as_object_or_java_value!();

            pub fn get_member_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<MemberName<'gc_life>> {
                let maybe_null = self.normal_object.lookup_field(jvm, "member");
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
            pub fn get_member(&self, jvm: &JVMState<'gc_life>) -> MemberName<'gc_life> {
                self.get_member_or_null(jvm).unwrap()
            }

            pub fn method_type(&self, jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<MethodType<'gc_life>, WasException> { // java.lang.invoke.LambdaForm.NamedFunction
                let named_function_type = assert_inited_or_initing_class(jvm, ClassName::Str("java/lang/invoke/LambdaForm$NamedFunction".to_string()).into());
                int_state.push_current_operand_stack(self.clone().java_value());
                run_static_or_virtual(jvm, int_state, &named_function_type, "methodType".to_string(), "()Ljava/lang/invoke/MethodType;".to_string())?;
                Ok(int_state.pop_current_operand_stack(ClassName::method_type().into()).cast_method_type())
            }
        }
    }

    pub mod name {
        use itertools::Itertools;

        use jvmti_jni_bindings::jint;

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
                Name { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl<'gc_life> Name<'gc_life> {
            as_object_or_java_value!();
            pub fn arguments(&self, jvm: &JVMState<'gc_life>) -> Vec<JavaValue<'gc_life>> {
                self.normal_object.unwrap_normal_object().get_var_top_level(jvm, "arguments")
                    .unwrap_array().array_iterator(jvm).collect_vec()
            }


            pub fn get_index_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<jint> {
                let maybe_null = self.normal_object.lookup_field(jvm, "index");
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
            pub fn get_index(&self, jvm: &JVMState<'gc_life>) -> jint {
                self.get_index_or_null(jvm).unwrap()
            }
            pub fn get_type_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<BasicType<'gc_life>> {
                let maybe_null = self.normal_object.lookup_field(jvm, "type");
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
            pub fn get_type(&self, jvm: &JVMState<'gc_life>) -> BasicType<'gc_life> {
                self.get_type_or_null(jvm).unwrap()
            }
            pub fn get_function_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<NamedFunction<'gc_life>> {
                let maybe_null = self.normal_object.lookup_field(jvm, "function");
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
            pub fn get_function(&self, jvm: &JVMState<'gc_life>) -> NamedFunction<'gc_life> {
                self.get_function_or_null(jvm).unwrap()
            }
        }
    }

    pub mod basic_type {
        use jvmti_jni_bindings::jchar;
        use jvmti_jni_bindings::jint;

        use crate::java::lang::class::JClass;
        use crate::java_values::{GcManagedObject, JavaValue, Object};
        use crate::JString;
        use crate::jvm_state::JVMState;

        #[derive(Clone)]
        pub struct BasicType<'gc_life> {
            normal_object: GcManagedObject<'gc_life>,
        }

        impl<'gc_life> JavaValue<'gc_life> {
            pub fn cast_lambda_form_basic_type(&self) -> BasicType<'gc_life> {
                BasicType { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl<'gc_life> BasicType<'gc_life> {
            as_object_or_java_value!();

            pub fn get_ordinal_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<jint> {
                let maybe_null = self.normal_object.lookup_field(jvm, "ordinal");
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
            pub fn get_ordinal(&self, jvm: &JVMState<'gc_life>) -> jint {
                self.get_ordinal_or_null(jvm).unwrap()
            }
            pub fn get_bt_char_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<jchar> {
                let maybe_null = self.normal_object.lookup_field(jvm, "btChar");
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
            pub fn get_bt_char(&self, jvm: &JVMState<'gc_life>) -> jchar {
                self.get_bt_char_or_null(jvm).unwrap()
            }
            pub fn get_bt_class_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<JClass<'gc_life>> {
                let maybe_null = self.normal_object.lookup_field(jvm, "btClass");
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
            pub fn get_bt_class(&self, jvm: &JVMState<'gc_life>) -> JClass<'gc_life> {
                self.get_bt_class_or_null(jvm).unwrap()
            }
            pub fn get_name_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<JString<'gc_life>> {
                let maybe_null = self.normal_object.lookup_field(jvm, "name");
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
            pub fn get_name(&self, jvm: &JVMState<'gc_life>) -> JString<'gc_life> {
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
            LambdaForm { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> LambdaForm<'gc_life> {
        pub fn names(&self, jvm: &JVMState<'gc_life>) -> Vec<Name<'gc_life>> {
            self.normal_object.unwrap_normal_object().get_var_top_level(jvm, "names")
                .unwrap_array()
                .unwrap_object_array(jvm)
                .iter().map(|name| JavaValue::Object(todo!()/*name.clone()*/).cast_lambda_form_name()).collect()
        }

        pub fn get_vmentry_or_null(&self, jvm: &JVMState<'gc_life>) -> Option<MemberName<'gc_life>> {
            let maybe_null = self.normal_object.lookup_field(jvm, "vmentry");
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
        pub fn get_vmentry(&self, jvm: &JVMState<'gc_life>) -> MemberName<'gc_life> {
            self.get_vmentry_or_null(jvm).unwrap()
        }

        as_object_or_java_value!();
    }
}

pub mod call_site {
    use rust_jvm_common::classnames::ClassName;
    use rust_jvm_common::descriptor_parser::MethodDescriptor;
    use rust_jvm_common::ptype::{PType, ReferenceType};

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
            CallSite { normal_object: self.unwrap_object_nonnull() }//todo every cast is an implicit npe
        }
    }

    impl<'gc_life> CallSite<'gc_life> {
        pub fn get_target(&self, jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<MethodHandle<'gc_life>, WasException> {
            let _call_site_class = assert_inited_or_initing_class(jvm, ClassName::Str("java/lang/invoke/CallSite".to_string()).into());
            int_state.push_current_operand_stack(self.clone().java_value());
            invoke_virtual(jvm, int_state, "getTarget", &MethodDescriptor { parameter_types: vec![], return_type: PType::Ref(ReferenceType::Class(ClassName::method_handle())) })?;
            Ok(int_state.pop_current_operand_stack(ClassName::object().into()).cast_method_handle())
        }

        as_object_or_java_value!();
    }
}