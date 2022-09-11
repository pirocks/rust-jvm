use itertools::Either;


use classfile_view::view::field_view::FieldView;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::classfile::{ACC_FINAL, ACC_NATIVE, ACC_STATIC, ACC_SYNTHETIC, ACC_VARARGS, REF_INVOKE_INTERFACE, REF_INVOKE_SPECIAL, REF_INVOKE_STATIC, REF_INVOKE_VIRTUAL};
use rust_jvm_common::compressed_classfile::names::CClassName;

use crate::{JVMState, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter::common::invoke::native::mhn_temp::{IS_CONSTRUCTOR, IS_METHOD, REFERENCE_KIND_SHIFT};
use crate::stdlib::java::lang::member_name::MemberName;
use crate::stdlib::java::lang::reflect::constructor::Constructor;
use crate::stdlib::java::lang::reflect::method::Method;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::new_java_values::owned_casts::OwnedCastAble;

pub fn MHN_init<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, args: Vec<NewJavaValue<'gc, '_>>) -> Result<(), WasException<'gc>> {
    //two params, is a static function.
    let mname = args[0].to_handle_discouraged().cast_member_name();
    let target = args[1].to_handle_discouraged();
    let target_object = target.cast_object();
    let to_string = target_object.to_string(jvm, int_state)?.unwrap().to_rust_string(jvm);
    let assertion_case = match to_string.as_str() {
        "static void java.lang.invoke.Invokers.checkExactType(java.lang.Object,java.lang.Object)" => InitAssertionCase::CHECK_EXACT_TYPE.into(),
        _ => None,
    };
    let res = init(jvm, int_state, mname.clone(), target_object.new_java_value(), Either::Left(None), false);
    if let Some(case) = assertion_case {
        match case {
            InitAssertionCase::CHECK_EXACT_TYPE => {
                assert_eq!(mname.get_flags(jvm), 100728840);
                assert_eq!(mname.get_clazz(jvm).gc_lifeify().as_type(jvm).unwrap_class_type(), CClassName::invokers());
            }
        }
    }
    res
}

pub enum InitAssertionCase {
    CHECK_EXACT_TYPE,
}

pub fn init<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, mname: MemberName<'gc>, target: NewJavaValue<'gc, '_>, view: Either<Option<&MethodView>, Option<&FieldView>>, synthetic: bool) -> Result<(), WasException<'gc>> {
    if target.unwrap_normal_object().unwrap().runtime_class(jvm).view().name() == CClassName::method().into() {
        let target = target.to_handle_discouraged().unwrap_object().unwrap().cast_method();
        method_init(jvm, int_state, mname.clone(), target, view.left().unwrap(), synthetic)?;
    } else if target.unwrap_normal_object().unwrap().runtime_class(jvm).view().name() == CClassName::constructor().into() {
        let target = target.to_handle_discouraged().unwrap_object().unwrap().cast_constructor();
        constructor_init(jvm, mname.clone(), target, view.left().unwrap(), synthetic)?;
    } else if target.unwrap_normal_object().unwrap().runtime_class(jvm).view().name() == CClassName::field().into() {
        todo!()
    } else {
        todo!()
    }
    Ok(())
    //this is a void method.
}

/*
// unofficial modifier flags, used by HotSpot:
    static final int BRIDGE    = 0x00000040;//todo tf is a bridge method
    static final int VARARGS   = 0x00000080;
    static final int SYNTHETIC = 0x00001000;
    static final int ANNOTATION= 0x00002000;
    static final int ENUM      = 0x00004000;

    static final int
                MN_IS_METHOD           = 0x00010000, // method (not constructor)
                MN_IS_CONSTRUCTOR      = 0x00020000, // constructor
                MN_IS_FIELD            = 0x00040000, // field
                MN_IS_TYPE             = 0x00080000, // nested type
                MN_CALLER_SENSITIVE    = 0x00100000, // @CallerSensitive annotation detected
                MN_REFERENCE_KIND_SHIFT = 24, // refKind
                MN_REFERENCE_KIND_MASK = 0x0F000000 >> MN_REFERENCE_KIND_SHIFT,
                // The SEARCH_* bits are not for MN.flags but for the matchFlags argument of MHN.getMembers:
                MN_SEARCH_SUPERCLASSES = 0x00100000,
                MN_SEARCH_INTERFACES   = 0x00200000;

         /**
         * Access modifier flags.
         */
        static final char
            ACC_PUBLIC                 = 0x0001,
            ACC_PRIVATE                = 0x0002,
            ACC_PROTECTED              = 0x0004,
            ACC_STATIC                 = 0x0008,
            ACC_FINAL                  = 0x0010,
            ACC_SYNCHRONIZED           = 0x0020,
            ACC_VOLATILE               = 0x0040,
            ACC_TRANSIENT              = 0x0080,
            ACC_NATIVE                 = 0x0100,
            ACC_INTERFACE              = 0x0200,
            ACC_ABSTRACT               = 0x0400,
            ACC_STRICT                 = 0x0800,
            ACC_SYNTHETIC              = 0x1000,
            ACC_ANNOTATION             = 0x2000,
            ACC_ENUM                   = 0x4000,
            // aliases:
            ACC_SUPER                  = ACC_SYNCHRONIZED,
            ACC_BRIDGE                 = ACC_VOLATILE,
            ACC_VARARGS                = ACC_TRANSIENT;
*/

/// the method view param here and elsewhere is only passed when resolving
fn method_init<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, mname: MemberName<'gc>, method: Method<'gc>, method_view: Option<&MethodView>, synthetic: bool) -> Result<(), WasException<'gc>> {
    let flags = method.get_modifiers(jvm);
    let clazz = method.get_clazz(jvm).gc_lifeify();
    mname.set_clazz(jvm, clazz.clone());
    //static v. invoke_virtual v. jni_interface
    //see MethodHandles::init_method_MemberName
    let invoke_type_flag = ((if (flags & ACC_STATIC as i32) > 0 {
        REF_INVOKE_STATIC
    } else {
        let class_ptye = clazz.as_type(jvm);
        let class_name = class_ptye.unwrap_ref_type().try_unwrap_name().unwrap_or_else(|| unimplemented!("Handle arrays?"));
        let inited_class = check_initing_or_inited_class(jvm, int_state, class_name.into())?;
        if inited_class.view().is_interface() {
            REF_INVOKE_INTERFACE
        } else {
            //afaik this is more of an optimization than anything else, but expected by standard library
            if inited_class.view().is_final() {
                REF_INVOKE_SPECIAL
            } else {
                REF_INVOKE_VIRTUAL
            }
        }
    } as u32)
        << REFERENCE_KIND_SHIFT) as i32;
    let extra_flags = IS_METHOD | invoke_type_flag as u32;
    let mut modifiers = method.get_modifiers(jvm);
    if let Some(method_view) = method_view {
        update_modifiers_with_method_view(synthetic, &mut modifiers, method_view)
    }
    mname.set_flags(jvm, modifiers | extra_flags as i32);
    Ok(())
}

fn update_modifiers_with_method_view(synthetic: bool, modifiers: &mut i32, method_view: &MethodView) {
    if method_view.is_varargs() {
        *modifiers |= ACC_VARARGS as i32;
        if method_view.is_signature_polymorphic() {
            *modifiers &= !(ACC_VARARGS as i32);
        }
    }
    if method_view.is_native() {
        *modifiers |= ACC_NATIVE as i32;
    }
    if method_view.is_static() {
        *modifiers |= ACC_STATIC as i32;
    }
    if method_view.is_final() || method_view.is_signature_polymorphic() {
        *modifiers |= ACC_FINAL as i32;
    }
    if synthetic {
        *modifiers |= ACC_SYNTHETIC as i32;
    }
}

fn constructor_init<'gc>(jvm: &'gc JVMState<'gc>, mname: MemberName<'gc>, constructor: Constructor<'gc>, method_view: Option<&MethodView>, synthetic: bool) -> Result<(), WasException<'gc>> {
    let clazz = constructor.get_clazz(jvm);
    mname.set_clazz(jvm, clazz.clone());
    //static v. invoke_virtual v. jni_interface
    //see MethodHandles::init_method_MemberName
    let invoke_type_flag = ((REF_INVOKE_SPECIAL as i32) << REFERENCE_KIND_SHIFT) as i32;
    let extra_flags = IS_CONSTRUCTOR | invoke_type_flag as u32;
    let mut modifiers = constructor.get_modifiers(jvm);
    if let Some(method_view) = method_view {
        update_modifiers_with_method_view(synthetic, &mut modifiers, method_view)
    }
    mname.set_flags(jvm, modifiers | extra_flags as i32);
    Ok(())
}