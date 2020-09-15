pub mod method_type {
    use std::cell::RefCell;
    use std::sync::Arc;

    use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
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
        pub fn from_method_descriptor_string<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, str: crate::java::lang::string::JString, class_loader: Option<ClassLoader>) -> MethodType {
            int_state.push_current_operand_stack(str.java_value());
            int_state.push_current_operand_stack(class_loader.map(|x| x.java_value()).unwrap_or(JavaValue::Object(None)));
            let method_type = check_inited_class(jvm, int_state, &ClassName::method_type().into(), int_state.current_loader(jvm).clone());
            crate::instructions::invoke::native::mhn_temp::run_static_or_virtual(jvm, int_state, &method_type, "fromMethodDescriptorString".to_string(), "(Ljava/lang/String;Ljava/lang/ClassLoader;)Ljava/lang/invoke/MethodType;".to_string());
            int_state.pop_current_operand_stack().cast_method_type()
        }

        pub fn set_rtype(&self, rtype: JClass) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("rtype".to_string(), rtype.java_value());
        }

        pub fn set_ptypes(&self, ptypes: JavaValue) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("ptypes".to_string(), ptypes);
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
            erased_type.map(|x| {
                res.set_erased_type(x);
            });
            basic_type.map(|x| {
                res.set_basic_type(x);
            });
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

    use crate::{InterpreterStateGuard, JVMState};
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_util::check_inited_class;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};

    pub struct MethodHandle {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method_handle(&self) -> MethodHandle {
            MethodHandle { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl MethodHandle {
        pub fn lookup<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Lookup {
            let method_handles_class = check_inited_class(jvm, int_state, &ClassName::method_handles().into(), int_state.current_loader(jvm));
            run_static_or_virtual(jvm, int_state, &method_handles_class, "lookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string());
            int_state.pop_current_operand_stack().cast_lookup()
        }
        pub fn public_lookup<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Lookup {
            let method_handles_class = check_inited_class(jvm, int_state, &ClassName::method_handles().into(), int_state.current_loader(jvm));
            run_static_or_virtual(jvm, int_state, &method_handles_class, "publicLookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string());
            int_state.pop_current_operand_stack().cast_lookup()
        }

        as_object_or_java_value!();
    }

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
        pub fn find_virtual<'l>(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, obj: JClass, name: JString, mt: MethodType) -> MethodHandle {
            let lookup_class = check_inited_class(jvm, int_state, &ClassName::lookup().into(), int_state.current_loader(jvm).clone());
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(obj.java_value());
            int_state.push_current_operand_stack(name.java_value());
            int_state.push_current_operand_stack(mt.java_value());
            run_static_or_virtual(jvm, int_state, &lookup_class, "findVirtual".to_string(), "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;".to_string());
            int_state.pop_current_operand_stack().cast_method_handle()
        }

        as_object_or_java_value!();
    }
}
