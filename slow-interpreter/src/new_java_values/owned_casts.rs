use crate::{AllocatedHandle, NewJavaValueHandle};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::array_out_of_bounds_exception::ArrayOutOfBoundsException;
use crate::stdlib::java::lang::invoke::call_site::CallSite;
use crate::stdlib::java::lang::invoke::lambda_form::basic_type::BasicType;
use crate::stdlib::java::lang::invoke::lambda_form::LambdaForm;
use crate::stdlib::java::lang::invoke::method_handle::MethodHandle;
use crate::stdlib::java::lang::invoke::method_handles::lookup::Lookup;
use crate::stdlib::java::lang::invoke::method_type::MethodType;
use crate::stdlib::java::lang::invoke::method_type_form::MethodTypeForm;
use crate::stdlib::java::lang::long::Long;
use crate::stdlib::java::lang::member_name::MemberName;
use crate::stdlib::java::lang::reflect::constructor::Constructor;
use crate::stdlib::java::lang::reflect::field::Field;
use crate::stdlib::java::lang::reflect::method::Method;
use crate::stdlib::java::lang::stack_trace_element::StackTraceElement;
use crate::stdlib::java::lang::throwable::Throwable;
use crate::stdlib::java::math::big_integer::BigInteger;
use crate::stdlib::java::nio::heap_byte_buffer::HeapByteBuffer;
use crate::stdlib::java::security::access_control_context::AccessControlContext;
use crate::stdlib::java::util::concurrent::concurrent_hash_map::ConcurrentHashMap;
use crate::stdlib::java::util::concurrent::concurrent_hash_map::node::Node;
use crate::stdlib::java::util::hashtable::entry::Entry;
use crate::stdlib::java::util::properties::Properties;
use crate::stdlib::sun::misc::unsafe_::Unsafe;
use crate::stdlib::sun::reflect::generics::tree::class_signature::ClassSignature;

pub trait OwnedCastAble<'gc> where Self: Sized {
    fn normal_object(self) -> AllocatedNormalObjectHandle<'gc>;
    fn cast_throwable(self) -> Throwable<'gc> {
        Throwable { normal_object: self.normal_object() }
    }
    fn cast_access_control_context(&self) -> AccessControlContext<'gc> {
        todo!()
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
    fn cast_long(self) -> Long<'gc> {
        Long { normal_object: self.normal_object() }
    }
    fn cast_array_out_of_bounds_exception(self) -> ArrayOutOfBoundsException<'gc> {
        ArrayOutOfBoundsException { normal_object: self.normal_object() }
    }
    fn cast_class_signature(self) -> ClassSignature<'gc> {
        ClassSignature { normal_object: self.normal_object() }
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