use crate::{AllocatedHandle, NewJavaValueHandle};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::arithmetic_exception::ArithmeticException;
use crate::stdlib::java::lang::array_out_of_bounds_exception::ArrayOutOfBoundsException;
use crate::stdlib::java::lang::boolean::Boolean;
use crate::stdlib::java::lang::byte::Byte;
use crate::stdlib::java::lang::char::Char;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::class_loader::ClassLoader;
use crate::stdlib::java::lang::double::Double;
use crate::stdlib::java::lang::float::Float;
use crate::stdlib::java::lang::illegal_argument_exception::IllegalArgumentException;
use crate::stdlib::java::lang::index_out_of_bounds_exception::IndexOutOfBoundsException;
use crate::stdlib::java::lang::int::Int;
use crate::stdlib::java::lang::interrupted_exception::InterruptedException;
use crate::stdlib::java::lang::invoke::call_site::CallSite;
use crate::stdlib::java::lang::invoke::lambda_form::basic_type::BasicType;
use crate::stdlib::java::lang::invoke::lambda_form::LambdaForm;
use crate::stdlib::java::lang::invoke::method_handle::MethodHandle;
use crate::stdlib::java::lang::invoke::method_handles::lookup::Lookup;
use crate::stdlib::java::lang::invoke::method_type::MethodType;
use crate::stdlib::java::lang::invoke::method_type_form::MethodTypeForm;
use crate::stdlib::java::lang::long::Long;
use crate::stdlib::java::lang::member_name::MemberName;
use crate::stdlib::java::lang::no_such_method_exception::NoSuchMethodError;
use crate::stdlib::java::lang::null_pointer_exception::NullPointerException;
use crate::stdlib::java::lang::reflect::constructor::Constructor;
use crate::stdlib::java::lang::reflect::field::Field;
use crate::stdlib::java::lang::reflect::method::Method;
use crate::stdlib::java::lang::short::Short;
use crate::stdlib::java::lang::stack_trace_element::StackTraceElement;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::lang::throwable::Throwable;
use crate::stdlib::java::math::big_integer::BigInteger;
use crate::stdlib::java::nio::direct_byte_buffer::DirectByteBuffer;
use crate::stdlib::java::nio::heap_byte_buffer::HeapByteBuffer;
use crate::stdlib::java::security::access_control_context::AccessControlContext;
use crate::stdlib::java::security::protection_domain::ProtectionDomain;
use crate::stdlib::java::util::concurrent::concurrent_hash_map::ConcurrentHashMap;
use crate::stdlib::java::util::concurrent::concurrent_hash_map::node::Node;
use crate::stdlib::java::util::hashtable::entry::Entry;
use crate::stdlib::java::util::properties::Properties;
use crate::stdlib::sun::misc::launcher::Launcher;
use crate::stdlib::sun::misc::unsafe_::Unsafe;
use crate::stdlib::sun::reflect::constant_pool::ConstantPool;
use crate::stdlib::sun::reflect::generics::tree::class_signature::ClassSignature;

pub trait OwnedCastAble<'gc> where Self: Sized {
    fn normal_object(self) -> AllocatedNormalObjectHandle<'gc>;
    fn cast_throwable(self) -> Throwable<'gc> {
        Throwable { normal_object: self.normal_object() }
    }
    fn cast_access_control_context(self) -> AccessControlContext<'gc> {
        AccessControlContext { normal_object: self.normal_object() }
    }
    fn cast_heap_byte_buffer(self) -> HeapByteBuffer<'gc> {
        HeapByteBuffer { normal_object: self.normal_object() }
    }
    fn cast_big_integer(self) -> BigInteger<'gc> {
        BigInteger { normal_object: self.normal_object() }
    }
    fn cast_concurrent_hash_map(self) -> ConcurrentHashMap<'gc> {
        ConcurrentHashMap { normal_object: self.normal_object() }
    }
    fn cast_concurrent_hash_map_node(self) -> Node<'gc> {
        Node { normal_object: self.normal_object() }
    }
    fn cast_entry(self) -> Entry<'gc> {
        Entry { normal_object: self.normal_object() }
    }
    fn cast_properties(self) -> Properties<'gc> {
        Properties { normal_object: self.normal_object() }
    }
    fn cast_stack_trace_element(self) -> StackTraceElement<'gc> {
        StackTraceElement { normal_object: self.normal_object() }
    }
    fn cast_member_name(self) -> MemberName<'gc> {
        MemberName { normal_object: self.normal_object() }
    }
    fn cast_method(self) -> Method<'gc> {
        Method { normal_object: self.normal_object() }
    }
    fn cast_constructor(self) -> Constructor<'gc> {
        Constructor { normal_object: self.normal_object() }
    }
    fn cast_field(self) -> Field<'gc> {
        Field { normal_object: self.normal_object() }
    }
    fn cast_lambda_form_basic_type(self) -> BasicType<'gc> {
        BasicType { normal_object: self.normal_object() }
    }
    fn cast_lambda_form(self) -> LambdaForm<'gc> {
        LambdaForm { normal_object: self.normal_object() }
    }
    fn cast_call_site(self) -> CallSite<'gc> {
        CallSite { normal_object: self.normal_object() } //todo every cast is an implicit npe
    }
    fn cast_method_type(self) -> MethodType<'gc> {
        MethodType { normal_object: self.normal_object() }
    }
    fn cast_method_type_form(self) -> MethodTypeForm<'gc> {
        MethodTypeForm { normal_object: self.normal_object() }
    }
    fn cast_unsafe(self) -> Unsafe<'gc> {
        Unsafe { normal_object: self.normal_object() }
    }
    fn cast_method_handle(self) -> MethodHandle<'gc> {
        MethodHandle { normal_object: self.normal_object() }
    }
    fn cast_lookup(self) -> Lookup<'gc> {
        Lookup { normal_object: self.normal_object() }
    }
    fn cast_array_out_of_bounds_exception(self) -> ArrayOutOfBoundsException<'gc> {
        ArrayOutOfBoundsException { normal_object: self.normal_object() }
    }
    fn cast_class_signature(self) -> ClassSignature<'gc> {
        ClassSignature { normal_object: self.normal_object() }
    }
    fn cast_arithmetic_exception(self) -> ArithmeticException<'gc> {
        ArithmeticException { normal_object: self.normal_object() }
    }

    fn cast_illegal_argument_exception(self) -> IllegalArgumentException<'gc> {
        IllegalArgumentException { normal_object: self.normal_object() }
    }

    fn cast_class_loader(self) -> ClassLoader<'gc> {
        ClassLoader { normal_object: self.normal_object() }
    }

    fn cast_class(self) -> JClass<'gc> {
        JClass { normal_object: self.normal_object() }
    }

    fn cast_string(self) -> JString<'gc> {
        JString { normal_object: self.normal_object() }
    }

    fn cast_boolean(self) -> Boolean<'gc> {
        Boolean { normal_object: self.normal_object() }
    }

    fn cast_byte(self) -> Byte<'gc> {
        Byte { normal_object: self.normal_object() }
    }

    fn cast_char(self) -> Char<'gc> {
        Char { normal_object: self.normal_object() }
    }

    fn cast_short(self) -> Short<'gc> {
        Short { normal_object: self.normal_object() }
    }

    fn cast_int(self) -> Int<'gc> {
        Int { normal_object: self.normal_object() }
    }

    fn cast_float(self) -> Float<'gc> {
        Float { normal_object: self.normal_object() }
    }

    fn cast_long(self) -> Long<'gc> {
        Long { normal_object: self.normal_object() }
    }

    fn cast_double(self) -> Double<'gc> {
        Double { normal_object: self.normal_object() }
    }

    fn cast_launcher(self) -> Launcher<'gc> {
        Launcher { normal_object: self.normal_object() }
    }

    fn cast_constant_pool(self) -> ConstantPool<'gc> {
        ConstantPool { normal_object: self.normal_object() }
    }

    fn cast_null_pointer_exception(self) -> NullPointerException<'gc> {
        NullPointerException { normal_object: self.normal_object() }
    }

    fn cast_index_out_of_bounds_exception(self) -> IndexOutOfBoundsException<'gc> {
        IndexOutOfBoundsException { normal_object: self.normal_object() }
    }

    fn cast_no_such_method_error(self) -> NoSuchMethodError<'gc> {
        NoSuchMethodError { normal_object: self.normal_object() }
    }

    fn cast_direct_byte_buffer(self) -> DirectByteBuffer<'gc> {
        DirectByteBuffer { normal_object: self.normal_object() }
    }

    fn cast_protection_domain(self) -> ProtectionDomain<'gc> {
        ProtectionDomain { normal_object: self.normal_object() }
    }

    fn cast_interrupted_exception(self) -> InterruptedException<'gc> {
        InterruptedException { normal_object: self.normal_object() }
    }
}

impl<'gc> OwnedCastAble<'gc> for AllocatedHandle<'gc> {
    fn normal_object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.unwrap_normal_object()
    }
}

impl<'gc> OwnedCastAble<'gc> for AllocatedNormalObjectHandle<'gc> {
    fn normal_object(self) -> AllocatedNormalObjectHandle<'gc> {
        self
    }
}

impl<'gc> OwnedCastAble<'gc> for NewJavaValueHandle<'gc> {
    fn normal_object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.unwrap_object().unwrap().unwrap_normal_object()
    }
}