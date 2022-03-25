#![feature(box_syntax)]

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::mem::transmute;
use std::path::PathBuf;
use std::sync::Arc;
use ntest::timeout;

use crossbeam::thread::Scope;
use tempdir::TempDir;
use wtf8::Wtf8Buf;
use classfile_writer::WriterContext;

use rust_jvm::main_run;
use rust_jvm_common::classfile::{ACC_PUBLIC, ACC_STATIC, AttributeInfo, AttributeType, Class, Classfile, Code, ConstantInfo, ConstantKind, CPIndex, Instruction, InstructionInfo, InvalidConstant, MethodInfo, Utf8};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::compressed_classfile::CompressedClassfileStringPool;
use rust_jvm_common::compressed_classfile::names::{CompressedClassName};
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use rust_jvm_common::EXPECTED_CLASSFILE_MAGIC;
use rust_jvm_common::ptype::{PType, ReferenceType};
use slow_interpreter::java_values::GC;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::loading::Classpath;
use slow_interpreter::options::JVMOptions;

pub struct TestVMConfig {
    main_class_name: String,
    class_path_dir: TempDir,
    psvm_bytecode:Vec<TestByteCode>,
    string_pool: CompressedClassfileStringPool
}

impl Default for TestVMConfig {
    fn default() -> Self {
        Self {
            main_class_name: "foo/bar/TestMain".to_string(),
            class_path_dir: TempDir::new("rust-jvm-test").unwrap(),
            psvm_bytecode: vec![TestByteCode::Return],
            string_pool: CompressedClassfileStringPool::new()
        }
    }
}

#[derive(Copy, Clone,PartialEq,Hash,Debug,Eq)]
pub enum TestByteCode {
    IConst0,
    IReturn,
    Return,
    ALoad(u16),
}

impl TestVMConfig {
    fn string_to_utf8_entry(rust_string: String) -> Utf8 {
        let wtf8 = Wtf8Buf::from_string(rust_string);
        Utf8 {
            length: wtf8.len() as u16,
            string: wtf8,
        }
    }

    fn add_utf8_to_constant_pool(classfile: &mut Classfile, rust_string: String) -> CPIndex {
        let utf8 = Self::string_to_utf8_entry(rust_string);
        let res = classfile.constant_pool.len() as CPIndex;
        classfile.constant_pool.push(ConstantInfo { kind: ConstantKind::Utf8(utf8) });
        res
    }

    fn init_this_class(classfile: &mut Classfile, name: String) {
        let name_index = Self::add_utf8_to_constant_pool(classfile, name);
        let this_index = classfile.constant_pool.len() as CPIndex;
        classfile.constant_pool.push(ConstantInfo { kind: ConstantKind::Class(Class { name_index }) });
        classfile.this_class = this_index;
    }

    fn add_method_with_code(classfile: &mut Classfile, name: String, descriptor: MethodDescriptor, access_flags: u16, code: Code) {
        let method_name_index = Self::add_utf8_to_constant_pool(classfile, name);
        let desc_string = descriptor.jvm_representation();
        let desc_string_index = Self::add_utf8_to_constant_pool(classfile, desc_string);
        let code_string_index = Self::add_utf8_to_constant_pool(classfile, "Code".to_string());
        let attributes = vec![AttributeInfo {
            attribute_name_index: code_string_index,
            attribute_length: 0,
            attribute_type: AttributeType::Code(code),
        }];
        classfile.methods.push(MethodInfo {
            access_flags,
            name_index: method_name_index,
            descriptor_index: desc_string_index,
            attributes,
        })
    }

    fn calc_max_stack(code: &[TestByteCode]) -> u16 {
        let mut stack = 0i32;
        let mut max_stack = 0u16;
        for op in code {
            stack += match op {
                TestByteCode::IConst0 => 1,
                TestByteCode::IReturn => -1i32,
                TestByteCode::Return => 0,
                TestByteCode::ALoad(_) => 1,
            };
            if stack > max_stack as i32 {
                max_stack = stack as u16;
            }
        }
        max_stack
    }

    fn calc_max_locals(code: &[TestByteCode], desc: &MethodDescriptor, is_static: bool) -> u16 {
        let mut max_locals = (desc.parameter_types.len() + if is_static { 0 } else { 1 }) as u16;
        for op in code {
            let local_var_index = match op {
                TestByteCode::IConst0 => 0,
                TestByteCode::IReturn => 0,
                TestByteCode::Return => 0,
                TestByteCode::ALoad(n) => {
                    *n
                }
            };
            if local_var_index >= max_locals {
                max_locals = local_var_index + 1;
            }
        }
        max_locals
    }

    fn convert_to_parsed_code(code: &[TestByteCode]) -> Vec<Instruction> {
        let mut current_offset = 0;
        code.iter().map(|op| {
            let instruct_info = match op {
                TestByteCode::IConst0 => InstructionInfo::iconst_0,
                TestByteCode::IReturn => InstructionInfo::ireturn,
                TestByteCode::Return => InstructionInfo::return_,
                TestByteCode::ALoad(n) => {
                    if *n <= u8::MAX as u16 {
                        InstructionInfo::aload(*n as u8)
                    } else {
                        todo!()
                    }
                }
            };
            let size = instruct_info.size();
            let res = Instruction {
                offset: current_offset,
                size,
                instruction: instruct_info,
            };
            current_offset += size;
            res
        }).collect()
    }

    fn convert_parsed_code_to_unparsed_code(code: &[Instruction]) -> Vec<u8> {
        let mut bytes = vec![];
        let mut writer = WriterContext{ writer: &mut bytes };
        for instruct in code {
            assert_eq!(instruct.offset as usize, writer.writer.len());
            writer.write_instruction_info(&instruct.instruction).unwrap();
        }
        bytes
    }

    fn add_method_with_byte_code(classfile: &mut Classfile, name: String, desc: MethodDescriptor, access_flags: u16, code: Vec<TestByteCode>) {
        let max_locals = Self::calc_max_locals(code.as_slice(), &desc, access_flags & ACC_STATIC > 0);
        let max_stack = Self::calc_max_stack(code.as_slice());
        let code = Self::convert_to_parsed_code(code.as_slice());
        let code_raw = Self::convert_parsed_code_to_unparsed_code(code.as_slice());
        let code = Code {
            attributes: vec![],
            max_stack,
            max_locals,
            code_raw,
            code,
            exception_table: vec![],
        };
        Self::add_method_with_code(classfile, name, desc, access_flags, code)
    }

    fn add_psvm(classfile: &mut Classfile, code: Vec<TestByteCode>) {
        let string_array_type = vec![PType::Ref(ReferenceType::Array(box PType::Ref(ReferenceType::Class(ClassName::Str("java/lang/String".to_string())))))];
        let psvm_method_desc = MethodDescriptor { parameter_types: string_array_type, return_type: PType::VoidType };
        let access_flags = ACC_PUBLIC | ACC_STATIC;
        Self::add_method_with_byte_code(classfile, "main".to_string(), psvm_method_desc, access_flags, code)
    }

    fn fake_main_class(&self, psvm_code: Vec<TestByteCode>) -> Classfile {
        let mut res = Classfile {
            magic: EXPECTED_CLASSFILE_MAGIC,
            minor_version: 0,
            major_version: 52,
            constant_pool: vec![ConstantInfo { kind: (ConstantKind::InvalidConstant(InvalidConstant {}))}],
            access_flags: 0,
            this_class: 1,
            super_class: 0,
            interfaces: vec![],
            fields: vec![],
            methods: vec![],
            attributes: vec![],
        };
        Self::init_this_class(&mut res, self.main_class_name.to_string());
        Self::add_psvm(&mut res, psvm_code);
        res
    }

    fn fake_classpath(&self) -> Classpath {
        let mut cached_classes = HashMap::new();
        let fake_main_class = self.fake_main_class(self.psvm_bytecode.clone());
        let compressed_classfile_name = self.string_pool.add_name(class_name(&fake_main_class).get_referred_name(),true);
        cached_classes.insert(CompressedClassName(compressed_classfile_name),Arc::new(fake_main_class));
        Classpath::from_dirs_with_cache(vec![self.class_path_dir.path().to_path_buf().into_boxed_path(),PathBuf::from("/home/francis/Desktop/jdk8u232-b09/jre/lib/").into_boxed_path()],cached_classes)
    }

    fn _write_fake_main_class(&self) {
        let path_to_write = self.class_path_dir.path().join(&self.main_class_name);
        std::fs::create_dir_all(path_to_write.parent().unwrap()).unwrap();
        let _classfile_file = File::create(path_to_write).unwrap();
        todo!()
    }


    pub fn create_vm<'l>(self, _with_vm: impl for<'gc> FnOnce(&JVMState<'gc>)) {
        // self.write_fake_main_class();
        let fake_classpath = self.fake_classpath();
        let jvm_options = JVMOptions::new(ClassName::Str(self.main_class_name.to_string()), fake_classpath, vec![], OsString::from("/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/amd64/libjava.so"), OsString::new(), false, false, vec![], false, false, false, true);
        let gc: GC<'l> = GC::new(early_startup::get_regions());
        let gc_ref: &'l GC<'l> = unsafe { transmute(&gc) };
        crossbeam::scope(|scope: Scope<'l>| {
            let (_, jvm) = JVMState::new(jvm_options, scope, gc_ref,self.string_pool);
            let jvm_ref: &'l JVMState<'l> = Box::leak(box jvm);
            main_run(vec![], jvm_ref);
        })
            .expect("idk why this would happen")
    }
}



#[test]
#[timeout(6000)]
pub fn test_return_0() {
    let test_vm_config = TestVMConfig::default();
    test_vm_config.create_vm(|_jvm| {})
}


