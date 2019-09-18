use std::borrow::Borrow;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::io::Lines;
use std::process::{ChildStdout, Stdio};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use log::{info, trace, warn};
use regex::Regex;
use tempfile::NamedTempFile;

use class_loading::JVMClassesState;
use classfile::{ACC_ABSTRACT, ACC_ANNOTATION, ACC_BRIDGE, ACC_ENUM, ACC_FINAL, ACC_INTERFACE, ACC_MODULE, ACC_NATIVE, ACC_PRIVATE, ACC_PROTECTED, ACC_PUBLIC, ACC_STATIC, ACC_STRICT, ACC_SUPER, ACC_SYNTHETIC, ACC_TRANSIENT, ACC_VOLATILE, AttributeInfo, Classfile, code_attribute, FieldInfo, MethodInfo, parse_class_file};
use classfile::attribute_infos::AttributeType;
use classfile::constant_infos::{ConstantInfo, ConstantKind};
use classfile::parsing_util::ParsingContext;
use verification::code_verification::write_parse_code_attribute;
use verification::PrologOutput::{NeedsAnotherClass, True};
use verification::types::{parse_field_descriptor, parse_method_descriptor, write_type_prolog};

use self::prolog_initial_defs::prolog_initial_defs;
use verification::prolog_info_defs::{PrologGenContext, ExtraDescriptors, class_name, gen_prolog};

#[derive(Debug)]
pub struct NeedsToLoadAnotherClass {
    pub another_class: Box<String>
}

pub fn verify(state: &JVMClassesState) -> Option<String> {
    let mut prolog = Command::new("/usr/bin/prolog")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn prolog");
    let mut prolog_output = BufReader::new(prolog.stdout.take().expect("error reading prolog output"));
    let mut prolog_input = prolog.stdin.take().expect("error getting prolog input stream");
    let mut output_lines = prolog_output.lines();
    let mut context = init_prolog_context(&state);

    prolog_initial_defs(&mut prolog_input).unwrap();
    let initial_defs_written = read_true_false_another_class(&mut output_lines);
    match initial_defs_written {
        True => {
            trace!("Initial prolog defs accepted by prolog.");
        },
        PrologOutput::False => { panic!() },
        NeedsAnotherClass(_) => { panic!() },
    }

    let mut generated_prolog_defs_file = NamedTempFile::new().expect("Error creating tempfile");
    trace!("tempfile for prolog defs created at: {}", generated_prolog_defs_file.path().as_os_str().to_str().expect("Could not convert path to str"));
    gen_prolog(&mut context, &mut generated_prolog_defs_file.as_file()).unwrap();
    write!(&mut prolog_input, "['{}'].\n", generated_prolog_defs_file.path().as_os_str().to_os_string().to_str().expect("Could not convert path to string")).unwrap();
    prolog_input.flush().unwrap();

    let generated_defs_res = read_true_false_another_class(&mut output_lines);
    match generated_defs_res {
        True => {
            trace!("Prolog accepted verification info");
            let classes : Vec<String> = context.to_verify.iter().map(|class: &Classfile| {
                class_name(class)
            }).collect();
            dbg!(classes);
        },
        PrologOutput::False => { panic!() },
        NeedsAnotherClass(_) => { panic!() },
    }

    write!(&mut prolog_input, "classIsTypeSafe(class('java/lang/Object', bl)).\n",).unwrap();
    prolog_input.flush().unwrap();

    let loading_attempt_res = read_true_false_another_class(&mut output_lines);
    prolog.kill().expect("Unable to kill prolog");
    match loading_attempt_res {
        True => {
            return None;
        },
        PrologOutput::False => { panic!() },
        NeedsAnotherClass(s) => {
            trace!("Need to load {} first",s);
            return Some(s);
        },
    }
}

enum PrologOutput {
    True,
    False,
    NeedsAnotherClass(String),
}

fn read_true_false_another_class(lines: &mut Lines<BufReader<ChildStdout>>) -> PrologOutput {
    let need_to_load_regex = Regex::new("Need to load:('([A-Za-z/]+)'|([A-Za-z/]+))").expect("Error parsing regex.");
    loop {
        let cur = lines.next();
        dbg!(&cur);
        let r = cur.unwrap();
        let s = r.unwrap();
        if s.contains("true.") {
            assert!(!s.contains("false."));
            return PrologOutput::True;
        } else if s.contains("false.") {
            assert!(!s.contains("true."));
            dbg!("false");
            return PrologOutput::False;
        } else if need_to_load_regex.is_match(s.as_str()) {//todo pattern needs string const
            let captures = need_to_load_regex.captures(s.as_str()).unwrap();
//            dbg!(&captures);
            let class_name = captures.get(3).unwrap().as_str().to_string();
//            dbg!("got class name");
            return PrologOutput::NeedsAnotherClass(class_name);
        }
    }
}

fn init_prolog_context<'s>(state: &'s JVMClassesState) -> PrologGenContext<'s> {
    let mut to_verify = Vec::new();
    for class_entry in &state.loading_in_progress {
        let path = state.indexed_classpath.get(class_entry).unwrap();
        let mut p = ParsingContext { f: File::open(path).expect("This is a bug") };
        let class_file = parse_class_file(&mut p);
        to_verify.push(class_file)
    }
    let mut context: PrologGenContext<'s> = PrologGenContext { state, to_verify, extra: ExtraDescriptors { extra_method_descriptors: Vec::new(), extra_field_descriptors: Vec::new() } };
    (context)
}

pub mod prolog_initial_defs;

pub mod prolog_info_defs;
pub mod code_verification;

pub mod types;

pub mod instruction_parser;