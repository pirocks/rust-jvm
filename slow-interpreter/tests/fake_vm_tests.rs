use iced_x86::CC_b::c;
use wtf8::Wtf8Buf;
use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classfile::{AttributeInfo, AttributeType, Class, Classfile, Code, ConstantInfo, ConstantKind, CPIndex, MethodInfo, String_, Utf8};
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use rust_jvm_common::EXPECTED_CLASSFILE_MAGIC;
use rust_jvm_common::ptype::PType;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::options::JVMOptions;

pub struct TestVMConfig {
    main_class_name: String
}

impl Default for TestVMConfig {
    fn default() -> Self {
        todo!()
    }
}

pub enum TestByteCode{
    IConst0,
    IReturn,
    Return,
}

impl TestVMConfig {

    fn string_to_utf8_entry(rust_string: String) -> Utf8 {
        let wtf8 =  Wtf8Buf::from_string(rust_string);
        Utf8{
            length: wtf8.len() as u16,
            string: wtf8
        }
    }

    fn add_utf8_to_constant_pool(classfile: &mut Classfile, rust_string: String) -> CPIndex{
        let utf8 = Self::string_to_utf8_entry(name);
        classfile.constant_pool.push(ConstantInfo{ kind: ConstantKind::Utf8(utf8) });
        classfile.constant_pool.len() as CPIndex
    }

    fn init_this_class(classfile: &mut Classfile, name: String){
        let name_index= Self::add_utf8_to_constant_pool(classfile,name);
        classfile.constant_pool.push(ConstantInfo{ kind: ConstantKind::Class(Class{ name_index }) });
        let this_index= classfile.constant_pool.len() as CPIndex;
        classfile.this_class = this_index;
    }

    fn add_method_with_code(classfile: &mut Classfile, name: String, descriptor: MethodDescriptor, access_flags: u16, code: Code){
        let method_name_index= Self::add_utf8_to_constant_pool(classfile,name);
        let desc_string= descriptor.jvm_representation();
        let desc_string_index = Self::add_utf8_to_constant_pool(classfile, desc_string);
        let code_string_index= Self::add_utf8_to_constant_pool(classfile, "Code".to_string());
        let attributes = vec![AttributeInfo{
            attribute_name_index: code_string_index,
            attribute_length: 0,
            attribute_type: AttributeType::Code(code)
        }];
        classfile.methods.push(MethodInfo{
            access_flags,
            name_index: method_name_index,
            descriptor_index: desc_string_index,
            attributes
        })
    }

    fn add_psvm(classfile:&mut Classfile, code: Vec<TestByteCode>){

    }

    fn fake_main_class(&self) -> Classfile{
        let mut res = Classfile{
            magic: EXPECTED_CLASSFILE_MAGIC,
            minor_version: 0,
            major_version: 52,
            constant_pool: vec![],
            access_flags: 0,
            this_class: 1,
            super_class: 0,
            interfaces: vec![],
            fields: vec![],
            methods: vec![],
            attributes: vec![]
        };
        Self::init_this_class(&mut res,self.main_class_name.to_string());

        res
    }

    pub fn create_vm<'gc_life>() -> JVMState<'gc_life> {
        JVMState::new(JVMOptions::new())
    }
}


