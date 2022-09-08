use std::ptr::null_mut;


use jvmti_jni_bindings::{jclass, jint, JNIEnv, jobject, jstring};
use rust_jvm_common::loading::{ClassLoadingError, LoaderName};
use slow_interpreter::better_java_stack::frames::PushableFrame;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::java::lang::class_loader::ClassLoader;
use slow_interpreter::java::NewAsObjectOrJavaValue;
use slow_interpreter::java_values::Object;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::rust_jni::interface::jni::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, to_object};
use slow_interpreter::sun::misc::launcher::ext_class_loader::ExtClassLoader;
use slow_interpreter::sun::misc::launcher::Launcher;
use slow_interpreter::utils::pushable_frame_todo;

#[no_mangle]
unsafe extern "system" fn JVM_CurrentLoadedClass(env: *mut JNIEnv) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    /*let ptype = int_state.current_frame().class_pointer(jvm).cpdtype();
    match get_or_create_class_object(jvm, ptype, pushable_frame_todo()) {
        Ok(class_obj) => to_object(class_obj.to_gc_managed().into()),
        Err(_) => null_mut(),
    }*/
    todo!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CurrentClassLoader(env: *mut JNIEnv) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let loader_name = int_state.current_loader(jvm);
    loader_name_to_native_obj(jvm, int_state, loader_name)
}

unsafe fn loader_name_to_native_obj<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, loader_name: LoaderName) -> jobject {
    new_local_ref_public(jvm.get_loader_obj(loader_name).map(|loader| loader.object().to_gc_managed()), todo!()/*int_state*/)
}

//from Java_java_lang_SecurityManager_classLoaderDepth0
////**
//      * Returns the stack depth of the most recently executing method
//      * from a class defined using a non-system class loader.  A non-system
//      * class loader is defined as being a class loader that is not equal to
//      * the system class loader (as returned
//      * by {@link ClassLoader#getSystemClassLoader}) or one of its ancestors.
//      * <p>
//      * This method will return
//      * -1 in the following three cases:
//      * <ol>
//      *   <li>All methods on the execution stack are from classes
//      *   defined using the system class loader or one of its ancestors.
//      *
//      *   <li>All methods on the execution stack up to the first
//      *   "privileged" caller
//      *   (see {@link java.security.AccessController#doPrivileged})
//      *   are from classes
//      *   defined using the system class loader or one of its ancestors.
//      *
//      *   <li> A call to <code>checkPermission</code> with
//      *   <code>java.security.AllPermission</code> does not
//      *   result in a SecurityException.
//      *
//      * </ol>
//      *
//      * @return the depth on the stack frame of the most recent occurrence of
//      *          a method from a class defined using a non-system class loader.
//      *
//      * @deprecated This type of security checking is not recommended.
//      *  It is recommended that the <code>checkPermission</code>
//      *  call be used instead.
//      *
//      * @see   java.lang.ClassLoader#getSystemClassLoader() getSystemClassLoader
//      * @see   #checkPermission(java.security.Permission) checkPermission
//      */
#[no_mangle]
unsafe extern "system" fn JVM_ClassLoaderDepth(env: *mut JNIEnv) -> jint {
    let int_state = get_state(env);
    todo!()
}
//todo need to call loadClassInternal when I am loading a class
//todo and also checkPackageAccess

#[no_mangle]
unsafe extern "system" fn JVM_LoadClass0(env: *mut JNIEnv, obj: jobject, currClass: jclass, currClassName: jstring) -> jclass {
    panic!("As far as I can tell this method isn't used by anything so its curious this code is being run");
}

// from Java_sun_misc_VM_latestUserDefinedLoader0
/// /*
//      * Returns first non-privileged class loader on the stack (excluding
//      * reflection generated frames) or the extension class loader if only
//      * class loaded by the boot class loader and extension class loader are
//      * found on the stack.
//      */
#[no_mangle]
unsafe extern "system" fn JVM_LatestUserDefinedLoader(env: *mut JNIEnv) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    todo!();/*for stack_entry in int_state.cloned_stack_snapshot(jvm) {
        if !stack_entry.privileged_frame() {
            return new_local_ref_public(jvm.get_loader_obj(stack_entry.loader()).map(|class_loader| class_loader.object().to_gc_managed()), int_state);
        }
    }*/
    return new_local_ref_public(
        todo!()/*        match ExtClassLoader::get_ext_class_loader(jvm, int_state) {
            Ok(res) => res,
            Err(_) => todo!(),
        }
            .object().to_gc_managed()
            .into()*/,
        int_state
    );
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassLoader(env: *mut JNIEnv, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let runtime_class = from_jclass(jvm, cls).as_runtime_class(jvm);
    let loader_name = jvm.classes.read().unwrap().get_initiating_loader(&runtime_class);
    loader_name_to_native_obj(jvm, int_state, loader_name)
}