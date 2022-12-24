use std::ops::Deref;

use itertools::Either;
use libc::c_void;

use classfile_view::view::method_view::MethodView;
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::compressed_classfile::code::{CInstructionInfo, CompressedCode};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::better_java_stack::frames::HasFrame;
use crate::function_instruction_count::FunctionExecutionCounter;
use crate::interpreter::arithmetic::{dadd, ddiv, dmul, dneg, drem, dsub, fadd, fdiv, fmul, fneg, frem, fsub, iadd, iand, idiv, imul, ineg, ior, irem, ishl, ishr, isub, iushr, ixor, ladd, land, lcmp, ldiv, lmul, lneg, lor, lrem, lshl, lshr, lsub, lushr, lxor};
use crate::interpreter::branch::{goto_, if_acmpeq, if_acmpne, if_icmpeq, if_icmpge, if_icmpgt, if_icmple, if_icmplt, if_icmpne, ifeq, ifge, ifgt, ifle, iflt, ifne, ifnonnull, ifnull};
use crate::interpreter::cmp::{dcmpg, dcmpl, fcmpg, fcmpl};
use crate::interpreter::common::invoke::dynamic::invoke_dynamic;
use crate::interpreter::common::invoke::interface::invoke_interface;
use crate::interpreter::common::invoke::special::invoke_special;
use crate::interpreter::common::invoke::static_::run_invoke_static;
use crate::interpreter::common::invoke::virtual_::invoke_virtual_instruction;
use crate::interpreter::common::special::invoke_instanceof;
use crate::interpreter::consts::{aconst_null, bipush, dconst_0, dconst_1, fconst_0, fconst_1, fconst_2, iconst_0, iconst_1, iconst_2, iconst_3, iconst_4, iconst_5, iconst_m1, lconst, sipush};
use crate::interpreter::conversion::{d2f, d2i, d2l, f2d, f2i, f2l, i2b, i2c, i2d, i2f, i2l, i2s, l2d, l2f, l2i};
use crate::interpreter::dup::{dup, dup2, dup2_x1, dup2_x2, dup_x1, dup_x2, swap};
use crate::interpreter::fields::{getfield, getstatic, putfield, putstatic};
use crate::interpreter::ldc::{ldc2_w, ldc_w};
use crate::interpreter::load::{aaload, aload, baload, caload, daload, dload, faload, fload, iaload, iload, laload, lload, saload};
use crate::interpreter::new::{anewarray, multi_a_new_array, new, newarray};
use crate::interpreter::pop::pop2;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterJavaValue, RealInterpreterStateGuard};
use crate::interpreter::special::{arraylength, checkcast};
use crate::interpreter::store::{aastore, astore, bastore, castore, dastore, dstore, fastore, fstore, iastore, istore, lastore, lstore, sastore};
use crate::interpreter::switch::{invoke_lookupswitch, tableswitch};
use crate::interpreter::throw::athrow;
use crate::interpreter::wide::wide;
use crate::JVMState;

pub fn run_single_instruction<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    interpreter_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>,
    instruct: &CInstructionInfo,
    function_counter: &FunctionExecutionCounter,
    _method: &MethodView,
    _code: &CompressedCode,
    current_pc: ByteCodeOffset,
) -> PostInstructionAction<'gc> {
    function_counter.increment();
    //hd/e
    //hd#readByte
    //io.netty.buffer.UnpooledHeapByteBuf#_getByte
    //io.netty.buffer.AbstractByteBuf#readByte
    //java.lang.UNIXProcess#initStreams
    //io.netty.channel.nio.NioEventLoop#run
    // sun.nio.ch.EPollSelectorImpl#doSelect
    // use classfile_view::view::ClassView;
    //#(java.lang.String)
    //java/util/TimeZone/getDisplayName
    //sun.util.locale.provider.LocaleServiceProviderPool#getLocalizedObjectImpl
    //sun.util.resources.OpenListResourceBundle#loadLookup
    //sun.util.resources.ja.TimeZoneNames_ja#getContents
    //java.util.ResourceBundle.Control#newBundle
    //java.util.ResourceBundle#findBundle
    //java.util.ResourceBundle#getBundleImpl
    //java.util.ResourceBundle.Control#getCandidateLocales
    //PowTests#testCrossProduct
    //sun.misc.Resource#getBytes
    //sun.security.provider.SHA5#implCompress0
    //sun.security.provider.ByteArrayAccess#b2lBig128
    //com.google.common.base.CharMatcher.RangesMatcher#RangesMatcher
    //"sun/font/FileFontStrike".getSlot0GlyphImagePtrs
    //sun.security.ssl.SSLContextImpl#engineInit
    //java.security.Provider.Service#newInstance
    //java.lang.reflect.Constructor#newInstance
    //sun.security.ssl.SSLContextImpl.DefaultSSLContext#DefaultSSLContext
    //sun.security.ssl.SSLContextImpl.DefaultManagersHolder#getTrustManagers
    //sun.security.ssl.TrustManagerFactoryImpl#engineInit(java.security.KeyStore)
    //sun.security.x509.CertificateExtensions#parseExtension
    // use classfile_view::view::ClassView;
    // if method.classview().name().jvm_representation(&jvm.string_pool).contains("sun/security/x509/CertificateExtensions") && method.name().0.to_str(&jvm.string_pool) == "parseExtension" ||
    //     (method.classview().name().jvm_representation(&jvm.string_pool).contains("sun/reflect/GeneratedConstructorAccessor1") && method.name().0.to_str(&jvm.string_pool) == "newInstance") /*||
    //     (method.classview().name().jvm_representation(&jvm.string_pool).contains("java/util/ResourceBundle$Control") && method.name().0.to_str(&jvm.string_pool) == "getCandidateLocales") ||
    //     (method.classview().name().jvm_representation(&jvm.string_pool).contains("DebuggingClass") && method.name().0.to_str(&jvm.string_pool) == "main")*/
    //     /*(method.name().0.to_str(&jvm.string_pool) == "<clinit>" || method.name().0.to_str(&jvm.string_pool) == "currentThread" ) &&
    //     (method.classview().name().jvm_representation(&jvm.string_pool).contains("java/lang/ref/Reference") || method.classview().name().jvm_representation(&jvm.string_pool).contains("java/lang/Thread"))*/{
    //     if let CInstructionInfo::ireturn | CInstructionInfo::return_ = instruct{
    //         interpreter_state.inner().debug_print_stack_trace(jvm);
    //     }
    //     dump_frame(interpreter_state, method, code, current_pc, instruct)
    // }
    // jvm.thread_state.debug_assert(jvm);
    match instruct {
        CInstructionInfo::aload(n) => aload(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::aload_0 => aload(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::aload_1 => aload(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::aload_2 => aload(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::aload_3 => aload(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::aaload => aaload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::aastore => aastore(interpreter_state.current_frame_mut()),
        CInstructionInfo::aconst_null => aconst_null(interpreter_state.current_frame_mut()),
        CInstructionInfo::anewarray(cp) => anewarray(jvm, interpreter_state, cp),
        CInstructionInfo::areturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::object()).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::arraylength => {
            arraylength(jvm, interpreter_state.current_frame_mut())
        }
        CInstructionInfo::astore(n) => astore(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::astore_0 => astore(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::astore_1 => astore(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::astore_2 => astore(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::astore_3 => astore(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::athrow => athrow(jvm, interpreter_state),
        CInstructionInfo::baload => baload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::bastore => bastore(interpreter_state.current_frame_mut()),
        CInstructionInfo::bipush(b) => bipush(interpreter_state.current_frame_mut(), *b),
        CInstructionInfo::caload => caload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::castore => castore(interpreter_state.current_frame_mut()),
        CInstructionInfo::checkcast(cp) => checkcast(jvm, interpreter_state, *cp),
        CInstructionInfo::d2f => d2f(interpreter_state.current_frame_mut()),
        CInstructionInfo::d2i => d2i(interpreter_state.current_frame_mut()),
        CInstructionInfo::d2l => d2l(interpreter_state.current_frame_mut()),
        CInstructionInfo::dadd => dadd(interpreter_state.current_frame_mut()),
        CInstructionInfo::daload => daload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dastore => dastore(interpreter_state.current_frame_mut()),
        CInstructionInfo::dcmpg => dcmpg(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dcmpl => dcmpl(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dconst_0 => dconst_0(interpreter_state.current_frame_mut()),
        CInstructionInfo::dconst_1 => dconst_1(interpreter_state.current_frame_mut()),
        CInstructionInfo::ddiv => ddiv(interpreter_state.current_frame_mut()),
        CInstructionInfo::dload(i) => dload(interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::dload_0 => dload(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::dload_1 => dload(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::dload_2 => dload(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::dload_3 => dload(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::dmul => dmul(interpreter_state.current_frame_mut()),
        CInstructionInfo::dneg => dneg(interpreter_state.current_frame_mut()),
        CInstructionInfo::drem => drem(interpreter_state.current_frame_mut()),
        CInstructionInfo::dreturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::DoubleType).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::dstore(i) => dstore(interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::dstore_0 => dstore(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::dstore_1 => dstore(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::dstore_2 => dstore(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::dstore_3 => dstore(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::dsub => dsub(interpreter_state.current_frame_mut()),
        CInstructionInfo::dup => dup(interpreter_state.current_frame_mut()),
        CInstructionInfo::dup_x1 => dup_x1(interpreter_state.current_frame_mut()),
        CInstructionInfo::dup_x2 => dup_x2(jvm, interpreter_state.inner().frame_ref().method_id().unwrap(), interpreter_state.current_frame_mut(), current_pc),
        CInstructionInfo::dup2 => dup2(jvm, interpreter_state.inner().frame_ref().method_id().unwrap(), interpreter_state.current_frame_mut(), current_pc),
        CInstructionInfo::dup2_x1 => dup2_x1(jvm, interpreter_state.inner().frame_ref().method_id().unwrap(), interpreter_state.current_frame_mut(), current_pc),
        CInstructionInfo::dup2_x2 => dup2_x2(jvm, interpreter_state.inner().frame_ref().method_id().unwrap(), interpreter_state.current_frame_mut(), current_pc),
        CInstructionInfo::f2d => f2d(interpreter_state.current_frame_mut()),
        CInstructionInfo::f2i => f2i(interpreter_state.current_frame_mut()),
        CInstructionInfo::f2l => f2l(interpreter_state.current_frame_mut()),
        CInstructionInfo::fadd => fadd(interpreter_state.current_frame_mut()),
        CInstructionInfo::faload => faload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fastore => fastore(interpreter_state.current_frame_mut()),
        CInstructionInfo::fcmpg => fcmpg(interpreter_state.current_frame_mut()),
        CInstructionInfo::fcmpl => fcmpl(interpreter_state.current_frame_mut()),
        CInstructionInfo::fconst_0 => fconst_0(interpreter_state.current_frame_mut()),
        CInstructionInfo::fconst_1 => fconst_1(interpreter_state.current_frame_mut()),
        CInstructionInfo::fconst_2 => fconst_2(interpreter_state.current_frame_mut()),
        CInstructionInfo::fdiv => fdiv(interpreter_state.current_frame_mut()),
        CInstructionInfo::fload(n) => fload(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::fload_0 => fload(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::fload_1 => fload(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::fload_2 => fload(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::fload_3 => fload(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::fmul => fmul(interpreter_state.current_frame_mut()),
        CInstructionInfo::fneg => fneg(interpreter_state.current_frame_mut()),
        CInstructionInfo::frem => frem(interpreter_state.current_frame_mut()),
        CInstructionInfo::freturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::FloatType).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::fstore(i) => fstore(interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::fstore_0 => fstore(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::fstore_1 => fstore(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::fstore_2 => fstore(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::fstore_3 => fstore(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::fsub => fsub(interpreter_state.current_frame_mut()),
        CInstructionInfo::getfield { desc, target_class, name } => getfield(jvm, interpreter_state.current_frame_mut(), *target_class, *name, desc),
        CInstructionInfo::getstatic { name, target_class, desc } => getstatic(jvm, interpreter_state, *target_class, *name, desc),
        CInstructionInfo::goto_(target) => goto_(*target as i32),
        CInstructionInfo::goto_w(target) => goto_(*target),
        CInstructionInfo::i2b => i2b(interpreter_state.current_frame_mut()),
        CInstructionInfo::i2c => i2c(interpreter_state.current_frame_mut()),
        CInstructionInfo::i2d => i2d(interpreter_state.current_frame_mut()),
        CInstructionInfo::i2f => i2f(interpreter_state.current_frame_mut()),
        CInstructionInfo::i2l => i2l(interpreter_state.current_frame_mut()),
        CInstructionInfo::i2s => i2s(interpreter_state.current_frame_mut()),
        CInstructionInfo::iadd => iadd(interpreter_state.current_frame_mut()),
        CInstructionInfo::iaload => iaload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iand => iand(interpreter_state.current_frame_mut()),
        CInstructionInfo::iastore => iastore(interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_m1 => iconst_m1(interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_0 => iconst_0(interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_1 => iconst_1(interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_2 => iconst_2(interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_3 => iconst_3(interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_4 => iconst_4(interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_5 => iconst_5(interpreter_state.current_frame_mut()),
        CInstructionInfo::idiv => idiv(interpreter_state.current_frame_mut()),
        CInstructionInfo::if_acmpeq(offset) => if_acmpeq(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_acmpne(offset) => if_acmpne(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmpeq(offset) => if_icmpeq(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmpne(offset) => if_icmpne(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmplt(offset) => if_icmplt(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmpge(offset) => if_icmpge(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmpgt(offset) => if_icmpgt(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmple(offset) => if_icmple(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifeq(offset) => ifeq(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifne(offset) => ifne(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::iflt(offset) => iflt(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifge(offset) => ifge(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifgt(offset) => ifgt(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifle(offset) => ifle(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifnonnull(offset) => ifnonnull(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifnull(offset) => ifnull(interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::iinc(iinc) => {
            let mut current_frame = interpreter_state.current_frame_mut();
            let val = current_frame.local_get(iinc.index, RuntimeType::IntType).unwrap_int();
            let res = val + iinc.const_ as i32;
            current_frame.local_set(iinc.index, InterpreterJavaValue::Int(res));
            PostInstructionAction::Next {}
        }
        CInstructionInfo::iload(n) => iload(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::iload_0 => iload(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::iload_1 => iload(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::iload_2 => iload(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::iload_3 => iload(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::imul => imul(interpreter_state.current_frame_mut()),
        CInstructionInfo::ineg => ineg(interpreter_state.current_frame_mut()),
        CInstructionInfo::instanceof(cp) => invoke_instanceof(jvm, interpreter_state.current_frame_mut(), *cp),
        // CInstructionInfo::invokedynamic(cp) => invoke_dynamic(jvm, interpreter_state, *cp),
        CInstructionInfo::invokeinterface { classname_ref_type, descriptor, method_name, count } => invoke_interface(jvm, interpreter_state, classname_ref_type.clone(), *method_name, descriptor, *count),
        CInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => invoke_special(jvm, interpreter_state, classname_ref_type.unwrap_object_name(), *method_name, descriptor),
        CInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => run_invoke_static(jvm, interpreter_state, classname_ref_type.clone(), *method_name, descriptor),
        CInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
            // dump_frame(interpreter_state,method,code);
            invoke_virtual_instruction(jvm, interpreter_state, *method_name, descriptor, *classname_ref_type)
        }
        CInstructionInfo::ior => ior(interpreter_state.current_frame_mut()),
        CInstructionInfo::irem => irem(interpreter_state.current_frame_mut()),
        CInstructionInfo::ireturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::IntType).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::ishl => ishl(interpreter_state.current_frame_mut()),
        CInstructionInfo::ishr => ishr(interpreter_state.current_frame_mut()),
        CInstructionInfo::istore(n) => istore(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::istore_0 => istore(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::istore_1 => istore(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::istore_2 => istore(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::istore_3 => istore(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::isub => isub(interpreter_state.current_frame_mut()),
        CInstructionInfo::iushr => iushr(interpreter_state.current_frame_mut()),
        CInstructionInfo::ixor => ixor(interpreter_state.current_frame_mut()),
        // CInstructionInfo::jsr(target) => jsr(interpreter_state, *target as i32),
        // CInstructionInfo::jsr_w(target) => jsr(interpreter_state, *target),
        CInstructionInfo::l2d => l2d(interpreter_state.current_frame_mut()),
        CInstructionInfo::l2f => l2f(interpreter_state.current_frame_mut()),
        CInstructionInfo::l2i => l2i(interpreter_state.current_frame_mut()),
        CInstructionInfo::ladd => ladd(interpreter_state.current_frame_mut()),
        CInstructionInfo::laload => laload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::land => land(interpreter_state.current_frame_mut()),
        CInstructionInfo::lastore => lastore(interpreter_state.current_frame_mut()),
        CInstructionInfo::lcmp => lcmp(interpreter_state.current_frame_mut()),
        CInstructionInfo::lconst_0 => lconst(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::lconst_1 => lconst(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::ldc(cldc2w) => ldc_w(jvm, interpreter_state, &cldc2w.as_ref()),
        CInstructionInfo::ldc_w(cldcw) => ldc_w(jvm, interpreter_state, &Either::Left(cldcw)),
        CInstructionInfo::ldc2_w(cldc2w) => ldc2_w(interpreter_state, cldc2w),
        CInstructionInfo::ldiv => ldiv(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lload(i) => lload(interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::lload_0 => lload(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::lload_1 => lload(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::lload_2 => lload(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::lload_3 => lload(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::lmul => lmul(interpreter_state.current_frame_mut()),
        CInstructionInfo::lneg => lneg(interpreter_state.current_frame_mut()),
        CInstructionInfo::lookupswitch(ls) => invoke_lookupswitch(&ls, interpreter_state.current_frame_mut()),
        CInstructionInfo::lor => lor(interpreter_state.current_frame_mut()),
        CInstructionInfo::lrem => lrem(interpreter_state.current_frame_mut()),
        CInstructionInfo::lreturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::LongType).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::lshl => lshl(interpreter_state.current_frame_mut()),
        CInstructionInfo::lshr => lshr(interpreter_state.current_frame_mut()),
        CInstructionInfo::lstore(n) => lstore(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::lstore_0 => lstore(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::lstore_1 => lstore(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::lstore_2 => lstore(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::lstore_3 => lstore(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::lsub => lsub(interpreter_state.current_frame_mut()),
        CInstructionInfo::lushr => lushr(interpreter_state.current_frame_mut()),
        CInstructionInfo::lxor => lxor(interpreter_state.current_frame_mut()),
        CInstructionInfo::monitorenter => {
            let obj = interpreter_state.current_frame_mut().pop(RuntimeType::object());
            let monitor = jvm.monitor_for(match obj.unwrap_object() {
                Some(x) => x,
                None => {
                    // interpreter_state.inner().debug_print_stack_trace(jvm);
                    panic!()
                }
            }.as_ptr() as *const c_void);
            monitor.lock(jvm, interpreter_state.inner()).unwrap();
            PostInstructionAction::Next {}
        }
        CInstructionInfo::monitorexit => {
            let obj = interpreter_state.current_frame_mut().pop(RuntimeType::object());
            let monitor = jvm.monitor_for(obj.unwrap_object().unwrap().as_ptr() as *const c_void);
            monitor.unlock(jvm, interpreter_state.inner()).unwrap();
            PostInstructionAction::Next {}
        }
        CInstructionInfo::multianewarray { type_, dimensions } => multi_a_new_array(jvm, interpreter_state, dimensions.get(), *type_),
        CInstructionInfo::new(cn) => new(jvm, interpreter_state, *cn),
        CInstructionInfo::newarray(a_type) => newarray(jvm, interpreter_state, *a_type),
        CInstructionInfo::nop => {
            PostInstructionAction::Next {}
        }
        CInstructionInfo::pop => {
            interpreter_state.current_frame_mut().pop(RuntimeType::LongType);
            PostInstructionAction::Next {}
        }
        CInstructionInfo::pop2 => pop2(jvm, interpreter_state.inner().frame_ref().method_id().unwrap(), interpreter_state.current_frame_mut(), current_pc),
        CInstructionInfo::putfield { name, desc, target_class } => putfield(jvm, interpreter_state, *target_class, *name, desc),
        CInstructionInfo::putstatic { name, desc, target_class } => putstatic(jvm, interpreter_state, *target_class, *name, desc),
        // CInstructionInfo::ret(local_var_index) => ret(jvm, interpreter_state.current_frame_mut(), *local_var_index as u16),
        // CInstructionInfo::return_ => return_(interpreter_state),
        CInstructionInfo::saload => saload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::sastore => sastore(interpreter_state.current_frame_mut()),
        CInstructionInfo::sipush(val) => sipush(interpreter_state.current_frame_mut(), *val),
        CInstructionInfo::swap => swap(interpreter_state.current_frame_mut()),
        CInstructionInfo::tableswitch(switch) => tableswitch(switch.deref(), interpreter_state.current_frame_mut()),
        CInstructionInfo::wide(w) => wide(interpreter_state.current_frame_mut(), w),
        // CInstructionInfo::EndOfCode => panic!(),
        CInstructionInfo::return_ => {
            PostInstructionAction::Return { res: None }
        }
        CInstructionInfo::invokedynamic(cp) => {
            invoke_dynamic(jvm, interpreter_state, *cp)
        }
        instruct => {
            interpreter_state.inner().debug_print_stack_trace(jvm);
            dbg!(instruct);
            todo!()
        }
    }
}

pub fn dump_frame(interpreter_state: &mut RealInterpreterStateGuard, method: &MethodView, _code: &CompressedCode, _current_pc: ByteCodeOffset, instruct: &CInstructionInfo) {
    let local_var_slots = method.local_var_slots();
    eprint!("Local Vars:");
    for i in 0..local_var_slots {
        let raw = interpreter_state.current_frame_mut().local_get(i, RuntimeType::LongType).to_raw();
        eprint!(" {:X}", raw);
    }
    eprintln!();
    eprint!("Operand Stack:");
    for i in 0..interpreter_state.current_stack_depth_from_start {
        let raw = interpreter_state.current_frame_mut().operand_stack_get(i, RuntimeType::LongType).to_raw();
        eprint!(" {:X}", raw);
    }
    eprintln!();
    dbg!(instruct.better_debug_string(&interpreter_state.inner().jvm().string_pool));
}
