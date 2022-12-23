use std::collections::HashSet;

use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::class_names::{CClassName, CompressedClassName};
use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;

pub struct DirectInvokeWhitelist {
    inner: HashSet<(CClassName, MethodName, CMethodDescriptor)>,
}


impl DirectInvokeWhitelist {
    pub fn new() -> Self {
        let mut inner = HashSet::new();

        Self::double_addressing_natives(&mut inner);
        Self::single_addressing_natives(&mut inner);
        Self::double_addressing_natives_volatile(&mut inner);

        //ordered and compare and swap among others still needed
        Self::strict_math(&mut inner);

        DirectInvokeWhitelist {
            inner,
        }
    }

    pub fn is_direct_invoke_whitelisted(&self, class: CClassName, method: MethodName, desc: CMethodDescriptor) -> bool {
        self.inner.contains(&(class, method, desc))
    }

    fn strict_math(inner: &mut HashSet<(CompressedClassName, MethodName, CompressedMethodDescriptor)>) {
        inner.insert((CClassName::strict_math(), MethodName::method_sqrt(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_sin(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_cos(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_tan(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_asin(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_acos(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_atan(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_exp(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_log(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_log10(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_cbrt(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_IEEEremainder(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType, CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_atan2(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType, CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_pow(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType, CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_sinh(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_cosh(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_tanh(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_hypot(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType, CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_expm1(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
        inner.insert((CClassName::strict_math(), MethodName::method_log1p(), CompressedMethodDescriptor { arg_types: vec![CompressedParsedDescriptorType::DoubleType], return_type: CompressedParsedDescriptorType::DoubleType }));
    }

    fn single_addressing_natives(inner: &mut HashSet<(CompressedClassName, MethodName, CompressedMethodDescriptor)>) {
        let u = CClassName::unsafe_();
        let get_methods = vec![
            (MethodName::method_getInt(), CPDType::IntType),
            (MethodName::method_getObject(), CPDType::object()),
            (MethodName::method_getBoolean(), CPDType::BooleanType),
            (MethodName::method_getByte(), CPDType::ByteType),
            (MethodName::method_getShort(), CPDType::ShortType),
            (MethodName::method_getChar(), CPDType::CharType),
            (MethodName::method_getLong(), CPDType::LongType),
            (MethodName::method_getFloat(), CPDType::FloatType),
            (MethodName::method_getDouble(), CPDType::DoubleType),
        ];
        for (method_name, return_type) in get_methods {
            inner.insert((u, method_name, CMethodDescriptor { arg_types: vec![CPDType::LongType], return_type }));
        }

        let put_methods = vec![
            (MethodName::method_putInt(), CPDType::IntType),
            (MethodName::method_putObject(), CPDType::object()),
            (MethodName::method_putBoolean(), CPDType::BooleanType),
            (MethodName::method_putByte(), CPDType::ByteType),
            (MethodName::method_putShort(), CPDType::ShortType),
            (MethodName::method_putChar(), CPDType::CharType),
            (MethodName::method_putLong(), CPDType::LongType),
            (MethodName::method_putFloat(), CPDType::FloatType),
            (MethodName::method_putDouble(), CPDType::DoubleType),
        ];

        for (method_name, to_put) in put_methods {
            inner.insert((u, method_name, CMethodDescriptor::void_return(vec![CPDType::LongType, to_put])));
        }
    }


    fn double_addressing_natives(inner: &mut HashSet<(CompressedClassName, MethodName, CompressedMethodDescriptor)>) {
        let u = CClassName::unsafe_();
        let get_methods = vec![
            (MethodName::method_getInt(), CPDType::IntType),
            (MethodName::method_getObject(), CPDType::object()),
            (MethodName::method_getBoolean(), CPDType::BooleanType),
            (MethodName::method_getByte(), CPDType::ByteType),
            (MethodName::method_getShort(), CPDType::ShortType),
            (MethodName::method_getChar(), CPDType::CharType),
            (MethodName::method_getLong(), CPDType::LongType),
            (MethodName::method_getFloat(), CPDType::FloatType),
            (MethodName::method_getDouble(), CPDType::DoubleType),
        ];
        for (method_name, return_type) in get_methods {
            inner.insert((u, method_name, CMethodDescriptor { arg_types: vec![CPDType::object(), CPDType::LongType], return_type }));
        }

        let put_methods = vec![
            (MethodName::method_putInt(), CPDType::IntType),
            (MethodName::method_putObject(), CPDType::object()),
            (MethodName::method_putBoolean(), CPDType::BooleanType),
            (MethodName::method_putByte(), CPDType::ByteType),
            (MethodName::method_putShort(), CPDType::ShortType),
            (MethodName::method_putChar(), CPDType::CharType),
            (MethodName::method_putLong(), CPDType::LongType),
            (MethodName::method_putFloat(), CPDType::FloatType),
            (MethodName::method_putDouble(), CPDType::DoubleType),
        ];

        for (method_name, to_put) in put_methods {
            inner.insert((u, method_name, CMethodDescriptor::void_return(vec![CPDType::object(), CPDType::LongType, to_put])));
        }
    }

    fn double_addressing_natives_volatile(inner: &mut HashSet<(CompressedClassName, MethodName, CompressedMethodDescriptor)>) {
        let u = CClassName::unsafe_();
        let get_methods = vec![
            (MethodName::method_getIntVolatile(), CPDType::IntType),
            (MethodName::method_getObjectVolatile(), CPDType::object()),
            (MethodName::method_getBooleanVolatile(), CPDType::BooleanType),
            (MethodName::method_getByteVolatile(), CPDType::ByteType),
            (MethodName::method_getShortVolatile(), CPDType::ShortType),
            (MethodName::method_getCharVolatile(), CPDType::CharType),
            (MethodName::method_getLongVolatile(), CPDType::LongType),
            (MethodName::method_getFloatVolatile(), CPDType::FloatType),
            (MethodName::method_getDoubleVolatile(), CPDType::DoubleType),
        ];
        for (method_name, return_type) in get_methods {
            inner.insert((u, method_name, CMethodDescriptor { arg_types: vec![CPDType::object(), CPDType::LongType], return_type }));
        }

        let put_methods = vec![
            (MethodName::method_putIntVolatile(), CPDType::IntType),
            (MethodName::method_putObjectVolatile(), CPDType::object()),
            (MethodName::method_putBooleanVolatile(), CPDType::BooleanType),
            (MethodName::method_putByteVolatile(), CPDType::ByteType),
            (MethodName::method_putShortVolatile(), CPDType::ShortType),
            (MethodName::method_putCharVolatile(), CPDType::CharType),
            (MethodName::method_putLongVolatile(), CPDType::LongType),
            (MethodName::method_putFloatVolatile(), CPDType::FloatType),
            (MethodName::method_putDoubleVolatile(), CPDType::DoubleType),
        ];

        for (method_name, to_put) in put_methods {
            inner.insert((u, method_name, CMethodDescriptor::void_return(vec![CPDType::object(), CPDType::LongType, to_put])));
        }
    }
}
