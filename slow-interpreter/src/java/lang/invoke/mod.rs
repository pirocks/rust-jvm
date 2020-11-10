pub mod method_type {
    use std::cell::RefCell;
    use std::sync::Arc;

    use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;
    use rust_jvm_common::ptype::PType;
    use type_safe_proc_macro_utils::getter_gen;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_util::{check_inited_class, push_new_object};
    use crate::java::lang::class::JClass;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::invoke::method_type_form::MethodTypeForm;
    use crate::java_values::{ArrayObject, JavaValue, Object};

    #[derive(Clone)]
    pub struct MethodType {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method_type(&self) -> MethodType {
            MethodType { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl MethodType {
        pub fn from_method_descriptor_string(jvm: &JVMState, int_state: &mut InterpreterStateGuard, str: crate::java::lang::string::JString, class_loader: Option<ClassLoader>) -> MethodType {
            int_state.push_current_operand_stack(str.java_value());
            int_state.push_current_operand_stack(class_loader.map(|x| x.java_value()).unwrap_or(JavaValue::Object(None)));
            let method_type = check_inited_class(jvm, int_state, &ClassName::method_type().into(), int_state.current_loader(jvm).clone());
            crate::instructions::invoke::native::mhn_temp::run_static_or_virtual(jvm, int_state, &method_type, "fromMethodDescriptorString".to_string(), "(Ljava/lang/String;Ljava/lang/ClassLoader;)Ljava/lang/invoke/MethodType;".to_string());
            int_state.pop_current_operand_stack().cast_method_type()
        }

        pub fn set_rtype(&self, rtype: JClass) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("rtype".to_string(), rtype.java_value());
        }

        getter_gen!(rtype,JClass,cast_class);

        pub fn get_rtype_as_type(&self) -> PType {
            self.get_rtype().as_type().to_ptype()
        }

        pub fn set_ptypes(&self, ptypes: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("ptypes".to_string(), ptypes);
        }

        getter_gen!(ptypes,JavaValue,clone);

        pub fn get_ptypes_as_types(&self) -> Vec<PType> {
            self.get_ptypes().unwrap_array().unwrap_object_array().iter()
                .map(|x| JavaValue::Object(x.clone()).cast_class().as_type().to_ptype()).collect()
        }

        pub fn set_form(&self, form: MethodTypeForm) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("form".to_string(), form.java_value());
        }

        pub fn get_form(&self) -> MethodTypeForm {
            self.normal_object.unwrap_normal_object().fields.borrow().get("form").unwrap().cast_method_type_form()
        }

        pub fn set_wrap_alt(&self, val: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("ptypes".to_string(), val);
        }

        pub fn set_invokers(&self, invokers: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("invokers".to_string(), invokers);
        }

        pub fn set_method_descriptors(&self, method_descriptor: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("methodDescriptor".to_string(), method_descriptor);
        }

        pub fn parameter_type(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, int: jint) -> JClass {
            let method_type = check_inited_class(jvm, int_state, &ClassName::method_type().into(), int_state.current_loader(jvm));
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(JavaValue::Int(int));
            run_static_or_virtual(jvm, int_state, &method_type, "parameterType".to_string(), "(I)Ljava/lang/Class;".to_string());
            int_state.pop_current_operand_stack().cast_class()
        }

        pub fn new(
            jvm: &JVMState,
            int_state: &mut InterpreterStateGuard,
            rtype: JClass,
            ptypes: Vec<JClass>,
            form: MethodTypeForm,
            wrap_alt: JavaValue,
            invokers: JavaValue,
            method_descriptor: JavaValue,
        ) -> MethodType {
            let method_type = check_inited_class(jvm, int_state, &ClassName::method_type().into(), int_state.current_loader(jvm).clone());
            push_new_object(jvm, int_state, &method_type, None);
            let res = int_state.pop_current_operand_stack().cast_method_type();
            let ptypes_arr = JavaValue::Object(Some(Arc::new(
                Object::Array(ArrayObject {
                    elems: RefCell::new(ptypes.into_iter().map(|x| x.java_value()).collect::<Vec<_>>()),
                    elem_type: PTypeView::Ref(ReferenceTypeView::Class(ClassName::class())),
                    monitor: jvm.thread_state.new_monitor("".to_string()),
                }))));
            res.set_ptypes(ptypes_arr);
            res.set_rtype(rtype);
            res.set_form(form);
            res.set_wrap_alt(wrap_alt);
            res.set_invokers(invokers);
            res.set_method_descriptors(method_descriptor);
            res
        }

        as_object_or_java_value!();
    }
}


pub mod method_type_form {
    use std::sync::Arc;

    use jvmti_jni_bindings::jlong;
    use rust_jvm_common::classnames::ClassName;

    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{check_inited_class, push_new_object};
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    #[derive(Clone)]
    pub struct MethodTypeForm {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method_type_form(&self) -> MethodTypeForm {
            MethodTypeForm { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl MethodTypeForm {
        pub fn set_arg_to_slot_table(&self, int_arr: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("argToSlotTable".to_string(), int_arr);
        }

        pub fn set_slot_to_arg_table(&self, int_arr: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("slotToArgTable".to_string(), int_arr);
        }

        pub fn set_arg_counts(&self, counts: jlong) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("argCounts".to_string(), JavaValue::Long(counts));
        }

        pub fn set_prim_counts(&self, counts: jlong) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("primCounts".to_string(), JavaValue::Long(counts));
        }

        pub fn set_erased_type(&self, type_: MethodType) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("erasedType".to_string(), type_.java_value());
        }

        pub fn set_basic_type(&self, type_: MethodType) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("basicType".to_string(), type_.java_value());
        }

        pub fn set_method_handles(&self, method_handle: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("methodHandles".to_string(), method_handle);
        }

        pub fn set_lambda_forms(&self, lambda_forms: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("methodHandles".to_string(), lambda_forms);
        }

        pub fn new(jvm: &JVMState,
                   int_state: &mut InterpreterStateGuard,
                   arg_to_slot_table: JavaValue,
                   slot_to_arg_table: JavaValue,
                   arg_counts: jlong,
                   prim_counts: jlong,
                   erased_type: Option<MethodType>,
                   basic_type: Option<MethodType>,
                   method_handles: JavaValue,
                   lambda_forms: JavaValue) -> MethodTypeForm {
            let method_type_form = check_inited_class(jvm, int_state, &ClassName::method_type_form().into(), int_state.current_loader(jvm).clone());
            push_new_object(jvm, int_state, &method_type_form, None);
            let res = int_state.pop_current_operand_stack().cast_method_type_form();
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
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;
    use type_safe_proc_macro_utils::getter_gen;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_util::check_inited_class;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::lambda_form::LambdaForm;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::member_name::MemberName;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};

    #[derive(Clone, Debug)]
    pub struct MethodHandle {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method_handle(&self) -> MethodHandle {
            MethodHandle { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl MethodHandle {
        //todo put this in MethodHandle
        pub fn lookup(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Lookup {
            let method_handles_class = check_inited_class(jvm, int_state, &ClassName::method_handles().into(), int_state.current_loader(jvm));
            run_static_or_virtual(jvm, int_state, &method_handles_class, "lookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string());
            int_state.pop_current_operand_stack().cast_lookup()
        }
        pub fn public_lookup(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Lookup {
            let method_handles_class = check_inited_class(jvm, int_state, &ClassName::method_handles().into(), int_state.current_loader(jvm));
            run_static_or_virtual(jvm, int_state, &method_handles_class, "publicLookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string());
            int_state.pop_current_operand_stack().cast_lookup()
        }

        pub fn internal_member_name(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> MemberName {
            let method_handle_class = check_inited_class(jvm, int_state, &ClassName::method_handle().into(), int_state.current_loader(jvm));
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(jvm, int_state, &method_handle_class, "internalMemberName".to_string(), "()Ljava/lang/invoke/MethodHandle;".to_string());
            int_state.pop_current_operand_stack().cast_member_name()
        }

        getter_gen!(form,LambdaForm,cast_lambda_form);

        as_object_or_java_value!();
    }

    //todo this is in the wrong place
    #[derive(Clone)]
    pub struct Lookup {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_lookup(&self) -> Lookup {
            Lookup { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Lookup {
        pub fn trusted_lookup(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Self {
            let lookup = check_inited_class(jvm, int_state, &ClassName::lookup().into(), int_state.current_loader(jvm).clone());
            let static_vars = lookup.static_vars();
            static_vars.get("IMPL_LOOKUP").unwrap().cast_lookup()
        }

        pub fn find_virtual(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, obj: JClass, name: JString, mt: MethodType) -> MethodHandle {
            let lookup_class = check_inited_class(jvm, int_state, &ClassName::lookup().into(), int_state.current_loader(jvm).clone());
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(obj.java_value());
            int_state.push_current_operand_stack(name.java_value());
            int_state.push_current_operand_stack(mt.java_value());
            run_static_or_virtual(jvm, int_state, &lookup_class, "findVirtual".to_string(), "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;".to_string());
            int_state.pop_current_operand_stack().cast_method_handle()
        }


        pub fn find_static(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, obj: JClass, name: JString, mt: MethodType) -> MethodHandle {
            let lookup_class = check_inited_class(jvm, int_state, &ClassName::lookup().into(), int_state.current_loader(jvm).clone());
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(obj.java_value());
            int_state.push_current_operand_stack(name.java_value());
            int_state.push_current_operand_stack(mt.java_value());
            run_static_or_virtual(jvm, int_state, &lookup_class, "findStatic".to_string(), "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;".to_string());
            int_state.pop_current_operand_stack().cast_method_handle()
        }

        as_object_or_java_value!();
    }
}


pub mod lambda_form {
    use std::sync::Arc;

    use type_safe_proc_macro_utils::getter_gen;

    use crate::java::lang::invoke::lambda_form::name::Name;
    use crate::java::lang::member_name::MemberName;
    use crate::java_values::{JavaValue, Object};

    pub mod named_function {
        use std::sync::Arc;

        use rust_jvm_common::classnames::ClassName;
        use type_safe_proc_macro_utils::getter_gen;

        use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
        use crate::interpreter_state::InterpreterStateGuard;
        use crate::interpreter_util::check_inited_class;
        use crate::java::lang::invoke::method_type::MethodType;
        use crate::java::lang::member_name::MemberName;
        use crate::java_values::{JavaValue, Object};
        use crate::jvm_state::JVMState;

        #[derive(Clone, Debug)]
        pub struct NamedFunction {
            normal_object: Arc<Object>
        }

        impl JavaValue {
            pub fn cast_lambda_form_named_function(&self) -> NamedFunction {
                NamedFunction { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl NamedFunction {
            as_object_or_java_value!();
            getter_gen!(member,MemberName,cast_member_name);

            // error appears to be in name.function.methodType().parameterType(paramIndex)
            pub fn method_type(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> MethodType { // java.lang.invoke.LambdaForm.NamedFunction
                let named_function_type = check_inited_class(jvm, int_state, &ClassName::Str("java/lang/invoke/LambdaForm$NamedFunction".to_string()).into(), int_state.current_loader(jvm));
                int_state.push_current_operand_stack(self.clone().java_value());
                run_static_or_virtual(jvm, int_state, &named_function_type, "methodType".to_string(), "()Ljava/lang/invoke/MethodType;".to_string());
                int_state.pop_current_operand_stack().cast_method_type()
            }
        }
    }

    pub mod name {
        use std::sync::Arc;

        use jvmti_jni_bindings::jint;
        use type_safe_proc_macro_utils::getter_gen;

        use crate::java::lang::invoke::lambda_form::basic_type::BasicType;
        use crate::java::lang::invoke::lambda_form::named_function::NamedFunction;
        use crate::java_values::{JavaValue, Object};

        #[derive(Clone, Debug)]
        pub struct Name {
            normal_object: Arc<Object>
        }

        impl JavaValue {
            pub fn cast_lambda_form_name(&self) -> Name {
                Name { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl Name {
            as_object_or_java_value!();
            pub fn arguments(&self) -> Vec<JavaValue> {
                self.normal_object.unwrap_normal_object().fields.borrow().get("arguments")
                    .unwrap()
                    .unwrap_array().elems.borrow().clone()
            }



            getter_gen!(index,jint,unwrap_int);

            getter_gen!(type,BasicType,cast_lambda_form_basic_type);

            getter_gen!(function,NamedFunction,cast_lambda_form_named_function);
        }
    }

    pub mod basic_type {
        use std::sync::Arc;

        use jvmti_jni_bindings::jchar;
        use jvmti_jni_bindings::jint;
        use type_safe_proc_macro_utils::getter_gen;

        use crate::java::lang::class::JClass;
        use crate::java_values::{JavaValue, Object};
        use crate::JString;

        #[derive(Clone, Debug)]
        pub struct BasicType {
            normal_object: Arc<Object>
        }

        impl JavaValue {
            pub fn cast_lambda_form_basic_type(&self) -> BasicType {
                BasicType { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl BasicType {
            as_object_or_java_value!();

            getter_gen!(ordinal,jint,unwrap_int);
            getter_gen!(btChar,jchar,unwrap_char);
            getter_gen!(btClass,JClass,cast_class);
            getter_gen!(name,JString,cast_string);
        }
    }


    #[derive(Clone, Debug)]
    pub struct LambdaForm {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_lambda_form(&self) -> LambdaForm {
            LambdaForm { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl LambdaForm {
        pub fn names(&self) -> Vec<Name> {
            self.normal_object.unwrap_normal_object().fields.borrow().get("names")
                .unwrap()
                .unwrap_array()
                .unwrap_object_array()
                .iter().map(|name| JavaValue::Object(name.clone()).cast_lambda_form_name()).collect()
        }

        getter_gen!(vmentry,MemberName,cast_member_name);
    }
}