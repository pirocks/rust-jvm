use itertools::Either;
use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jclass, JNIEnv, jobject, JVM_REF_invokeInterface, JVM_REF_invokeSpecial, JVM_REF_invokeStatic, JVM_REF_invokeVirtual, JVM_REF_newInvokeSpecial};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::mhn_consts::{IS_CONSTRUCTOR, IS_FIELD, IS_METHOD, IS_TYPE, REFERENCE_KIND_MASK, REFERENCE_KIND_SHIFT};
use slow_interpreter::better_java_stack::frames::PushableFrame;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::interpreter::common::invoke::dynamic::resolvers::methods::{ResolutionError, resolve_invoke_interface, resolve_invoke_special, resolve_invoke_static, resolve_invoke_virtual};
use slow_interpreter::interpreter_util::new_object;
use slow_interpreter::java_values::{ByAddressAllocatedObject, ExceptionReturn};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw, new_local_ref_internal_new};
use slow_interpreter::rust_jni::native_util::from_object_new;
use slow_interpreter::stdlib::java::lang::member_name::MemberName;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::utils::{get_all_fields, unwrap_or_npe};
use crate::init::init;


#[no_mangle]
pub unsafe extern "system" fn Java_java_lang_invoke_MethodHandleNatives_resolve(env: *mut JNIEnv, _c: jclass, member_name: jobject, _caller: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);

    let member_name = match from_object_new(jvm, member_name) {
        Some(allocated_handle) => allocated_handle.cast_member_name(),
        None => todo!(),
    };
    //so as far as I can find this is undocumented.
    //so as far as I can figure out we have a method name and a class
    //we lookup for a matching method, throw various kinds of exceptions if it doesn't work
    // and return a brand new object
    match resolve_impl(jvm, int_state, member_name) {
        Ok(res) => {
            new_local_ref_internal_new(res.full_object_ref(), int_state)
        }
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            jobject::invalid_default()
        }
    }
}

enum ResolveAssertionCase {
    // CAST,
    LinkToStatic,
    LinkToSpecial,
    ZeroL,
    MAKE,
    ArgL0,
    GetObjectUnsafe,
    IdentityL,
}

/*
// unofficial modifier flags, used by HotSpot:
static final int BRIDGE    = 0x00000040;
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

fn resolve_impl<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, member_name: MemberName<'gc>) -> Result<MemberName<'gc>, WasException<'gc>> {
    let assertion_case = if &member_name.get_name(jvm).to_rust_string(jvm) == "cast" &&
        member_name.get_clazz(jvm).gc_lifeify().as_type(jvm).unwrap_class_type() == CClassName::class() &&
        member_name.to_string(jvm, int_state)?.unwrap().to_rust_string(jvm) == "java.lang.Class.cast(Object)Object/invokeVirtual" {
        None
    } else if &member_name.get_name(jvm).to_rust_string(jvm) == "linkToStatic" {
        assert_eq!(member_name.get_flags(jvm), 100728832);
        assert!(member_name.get_resolution(jvm).unwrap_object().is_some());
        ResolveAssertionCase::LinkToStatic.into()
    } else if &member_name.get_name(jvm).to_rust_string(jvm) == "zero_L" {
        assert_eq!(member_name.get_flags(jvm), 100728832);
        ResolveAssertionCase::ZeroL.into()
    } else if &member_name.get_name(jvm).to_rust_string(jvm) == "linkToSpecial" && member_name.to_string(jvm, int_state)?.unwrap().to_rust_string(jvm) == "java.lang.invoke.MethodHandle.linkToSpecial(Object,Object,MemberName)Object/invokeStatic" {
        assert_eq!(member_name.get_flags(jvm), 100728832);
        ResolveAssertionCase::LinkToSpecial.into()
    } else if &member_name.get_name(jvm).to_rust_string(jvm) == "make" && member_name.to_string(jvm, int_state)?.unwrap().to_rust_string(jvm) == "java.lang.invoke.BoundMethodHandle$Species_L.make(MethodType,LambdaForm,Object)BoundMethodHandle/invokeStatic" {
        assert_eq!(member_name.get_flags(jvm), 100728832);
        ResolveAssertionCase::MAKE.into()
    } else if member_name.to_string(jvm, int_state)?.unwrap().to_rust_string(jvm) == "java.lang.invoke.BoundMethodHandle$Species_L.argL0/java.lang.Object/getField" {
        assert_eq!(member_name.get_flags(jvm), 17039360);
        ResolveAssertionCase::ArgL0.into()
    } else if member_name.to_string(jvm, int_state)?.unwrap().to_rust_string(jvm) == "sun.misc.Unsafe.getObject(Object,long)Object/invokeVirtual" {
        assert_eq!(member_name.get_flags(jvm), 83951616);
        // assert_eq!(member_name.get_type(jvm).cast_object().to_string(jvm, int_state)?.unwrap().to_rust_string(jvm), "(Object,long)Object");
        ResolveAssertionCase::GetObjectUnsafe.into()
    } else if member_name.to_string(jvm, int_state)?.unwrap().to_rust_string(jvm) == "java.lang.invoke.LambdaForm.identity_L(Object)Object/invokeStatic" {
        assert_eq!(member_name.get_flags(jvm), 100728832);
        // assert_eq!(member_name.get_type(jvm).cast_object().to_string(jvm, int_state)?.unwrap().to_rust_string(jvm), "(Object,long)Object");
        ResolveAssertionCase::IdentityL.into()
    } else {
        None
    };

    let type_java_value = member_name.get_type(jvm);
    let flags_val = member_name.get_flags(jvm);
    let ref_kind = ((flags_val >> REFERENCE_KIND_SHIFT) & REFERENCE_KIND_MASK as i32) as u32;
    let all_kinds = IS_METHOD | IS_CONSTRUCTOR | IS_FIELD | IS_TYPE;
    let kind = (flags_val & (all_kinds as i32)) as u32;
    match kind {
        IS_FIELD => {
            let all_fields = get_all_fields(jvm, int_state, member_name.get_clazz(jvm).gc_lifeify().as_runtime_class(jvm), true)?;

            let name = FieldName(jvm.string_pool.add_name(member_name.get_name(jvm).to_rust_string(jvm), false));

            let typejclass = unwrap_or_npe(jvm, int_state, (|| { Some(member_name.get_type(jvm).unwrap_object()?.cast_class()) })())?;
            let target_ptype = typejclass.as_type(jvm);
            let (res_c, res_i) = all_fields
                .iter()
                .find(|(c, i)| {
                    let view = c.view();
                    let field = view.field(*i);
                    field.field_name() == name && field.field_type() == target_ptype
                })
                .unwrap();

            let correct_flags = res_c.view().field(*res_i).access_flags();
            let new_flags = ((flags_val as u32) | (correct_flags as u32)) as i32;

            // we don't update clazz, since the specific versio may not be assignable unlike the generic, or like if we get an inherited field stuff breaks if we update
            member_name.set_flags(jvm, new_flags);
        }
        IS_METHOD => {
            if ref_kind == JVM_REF_invokeVirtual {
                let (resolve_result, method_i, class) = match resolve_invoke_virtual(jvm, int_state, member_name.clone())? {
                    Ok(ok) => ok,
                    Err(err) => match err {
                        ResolutionError::Linkage => {
                            throw_linkage_error(jvm, int_state)?;
                            unreachable!()
                        }
                    },
                };
                init(jvm, int_state, member_name.clone(), resolve_result.new_java_value(), Either::Left(Some(&class.view().method_view_i(method_i))), false)?;
            } else if ref_kind == JVM_REF_invokeStatic {
                let mut synthetic = false;
                let (resolve_result, method_i, class) = match resolve_invoke_static(jvm, int_state, member_name.clone(), &mut synthetic)? {
                    Ok(ok) => ok,
                    Err(err) => match err {
                        ResolutionError::Linkage => {
                            throw_linkage_error(jvm, int_state)?;
                            unreachable!()
                        }
                    },
                };
                let method_id = jvm.method_table.write().unwrap().get_method_id(class.clone(), method_i);
                jvm.resolved_method_handles.write().unwrap().insert(ByAddressAllocatedObject::Owned(member_name.clone().object()), method_id);
                init(jvm, int_state, member_name.clone(), resolve_result.new_java_value(), Either::Left(Some(&class.view().method_view_i(method_i))), synthetic)?;
            } else if ref_kind == JVM_REF_invokeInterface {
                let (resolve_result, method_i, class) = match resolve_invoke_interface(jvm, int_state, member_name.clone())? {
                    Ok(ok) => ok,
                    Err(err) => match err {
                        ResolutionError::Linkage => {
                            throw_linkage_error(jvm, int_state)?;
                            unreachable!()
                        }
                    },
                };
                init(jvm, int_state, member_name.clone(), resolve_result.new_java_value(), Either::Left(Some(&class.view().method_view_i(method_i))), false)?;
            } else if ref_kind == JVM_REF_invokeSpecial {
                let (resolve_result, method_i, class) = match resolve_invoke_special(jvm, int_state, member_name.clone())? {
                    Ok(ok) => ok,
                    Err(err) => match err {
                        ResolutionError::Linkage => {
                            throw_linkage_error(jvm, int_state)?;
                            unreachable!()
                        }
                    },
                };
                init(jvm, int_state, member_name.clone(), resolve_result.new_java_value(), Either::Left(Some(&class.view().method_view_i(method_i))), false)?;
            } else {
                panic!()
            }
        }
        IS_CONSTRUCTOR => {
            if ref_kind == JVM_REF_invokeSpecial {
                todo!()
            } else if ref_kind == JVM_REF_newInvokeSpecial {
                //todo dup
                //todo shouldn't this be resolve_invoke_constructor
                let (resolve_result, method_i, class) = match resolve_invoke_special(jvm, int_state, member_name.clone())? {
                    Ok(ok) => ok,
                    Err(err) => match err {
                        ResolutionError::Linkage => {
                            throw_linkage_error(jvm, int_state)?;
                            unreachable!()
                        }
                    },
                };
                init(jvm, int_state, member_name.clone(), resolve_result.new_java_value(), Either::Left(Some(&class.view().method_view_i(method_i))), false)?;
            } else {
                todo!()
            }
        }
        _ => panic!(),
    }
    let clazz = member_name.get_clazz(jvm).gc_lifeify();
    let _clazz_as_runtime_class = clazz.as_runtime_class(jvm);
    let _name = member_name.get_name(jvm).to_rust_string(jvm);
    let _type_ = type_java_value.unwrap_object().unwrap().unwrap_normal_object();
    if let Some(assertion_case) = assertion_case {
        match assertion_case {
            ResolveAssertionCase::LinkToStatic => {
                assert_eq!(&member_name.get_name(jvm).to_rust_string(jvm), "linkToStatic");
                assert_eq!(member_name.get_flags(jvm), 100733208);
                assert!(member_name.get_resolution(jvm).unwrap_object().is_some());
                assert_eq!(member_name.get_resolution(jvm).cast_member_name().get_flags(jvm), 100728832);
            }
            ResolveAssertionCase::ZeroL => {}
            ResolveAssertionCase::LinkToSpecial => {
                assert_eq!(&member_name.get_name(jvm).to_rust_string(jvm), "linkToSpecial");
                assert_eq!(member_name.get_flags(jvm), 100733208);
                assert!(member_name.get_resolution(jvm).unwrap_object().is_some());
                assert_eq!(member_name.get_resolution(jvm).cast_member_name().get_flags(jvm), 100728832);
            }
            ResolveAssertionCase::MAKE => {
                assert_eq!(&member_name.to_string(jvm, int_state)?.unwrap().to_rust_string(jvm), "java.lang.invoke.BoundMethodHandle$Species_L.make(MethodType,LambdaForm,Object)BoundMethodHandle/invokeStatic");
                assert_eq!(member_name.get_flags(jvm), 100728840);
                assert!(member_name.get_resolution(jvm).unwrap_object().is_some());
                assert_eq!(member_name.get_resolution(jvm).cast_member_name().get_flags(jvm), 100728832);
            }
            ResolveAssertionCase::ArgL0 => {
                assert_eq!(member_name.get_flags(jvm), 17039376);
            }
            ResolveAssertionCase::GetObjectUnsafe => {
                assert_eq!(member_name.get_flags(jvm), 117506305);
            }
            ResolveAssertionCase::IdentityL => {
                assert_eq!(member_name.get_flags(jvm), 100728842);
                assert!(member_name.get_resolution(jvm).unwrap_object().is_some());
                assert_eq!(member_name.get_resolution(jvm).cast_member_name().get_flags(jvm), 100728832);
            }
        }
    }

    Ok(member_name)
}

fn throw_linkage_error<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
    let linkage_error = check_initing_or_inited_class(jvm, int_state, CClassName::linkage_error().into())?;
    let object = new_object(jvm, int_state, &linkage_error, false);
    return Err(WasException { exception_obj: object.cast_throwable() });
}
