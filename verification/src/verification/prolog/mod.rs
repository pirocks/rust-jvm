use rust_jvm_common::loading::{JVMState, ClassEntry};
use crate::verification::prolog::prolog_info_writer::{PrologGenContext, ExtraDescriptors, gen_prolog};
use std::sync::Arc;
use rust_jvm_common::classfile::Classfile;
use classfile_parser::classfile::parse_class_file;
use std::fs::File;
use std::io;
use std::io::{BufWriter, BufReader, Lines, BufRead, Write};
use std::process::{Stdio, ChildStdout, Command, Child, ChildStdin};
use crate::verification::prolog::PrologOutput::NeedsAnotherClass;
use std::time::Duration;
use std::thread::sleep;
use std::collections::HashMap;
use tempfile::NamedTempFile;
use rust_jvm_common::classnames::class_name_legacy;
use regex::Regex;
use log::trace;

pub fn prolog_verify(state: &JVMState, to_verify: &HashMap<ClassEntry, Arc<Classfile>>) -> Option<String> {
    for (class_entry, _current_class_to_verify) in to_verify.iter() {
        let (mut prolog, mut prolog_input, mut output_lines, mut context) = init_prolog(&state);
        let generated_prolog_defs_file: NamedTempFile = NamedTempFile::new().expect("Error creating tempfile");
        trace!("tempfile for prolog defs created at: {}", generated_prolog_defs_file.path().as_os_str().to_str().expect("Could not convert path to str"));
        gen_prolog(&mut context, &mut generated_prolog_defs_file.as_file()).unwrap();
        write!(&mut prolog_input, "['{}'].\n", generated_prolog_defs_file.path().as_os_str().to_os_string().to_str().expect("Could not convert path to string")).unwrap();
        prolog_input.flush().unwrap();

        let generated_defs_res = read_true_false_another_class(&mut output_lines);
        match generated_defs_res {
            PrologOutput::True => {
                trace!("Prolog accepted verification info");
                let classes: Vec<String> = context.to_verify.iter().map(|class: &Arc<Classfile>| {
                    class_name_legacy(class)
                }).collect();
                dbg!(classes);
            }
            PrologOutput::False => { panic!() }
            NeedsAnotherClass(_) => { panic!() }
        }
        let current_name = &class_entry.name;
        let mut current_class_package = class_entry.packages.join("/");
        if current_class_package.len() != 0 {
            current_class_package.push_str("/");
        }
        trace!("Verifying '{}{}'", current_class_package, current_name);
        write!(&mut prolog_input, "class_is_type_safe(class('{}{}', bl)).\n\n", current_class_package, current_name).unwrap();
        prolog_input.flush().unwrap();
        write!(&mut prolog_input, "\n\n").unwrap();
        prolog_input.flush().unwrap();


        let loading_attempt_res = read_true_false_another_class(&mut output_lines);
        prolog.kill().expect("Unable to kill prolog");
        prolog.wait().expect("Unable to await prolog death");
        match loading_attempt_res {
            PrologOutput::True => {
                trace!("Successfully verified {}", class_entry);
            }
            PrologOutput::False => {
                sleep(Duration::from_secs(20000));
                panic!()
            }
            NeedsAnotherClass(s) => {
                trace!("Need to load {} first", s);
                return Some(s);
            }
        }
    }
    return None;
//verification was successful
}

fn init_prolog(state: &JVMState) -> (Child, BufWriter<ChildStdin>, Lines<BufReader<ChildStdout>>, PrologGenContext) {
    let mut prolog = Command::new("/usr/bin/swipl")//only tested with swi-prolog, other prologs may work.
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn prolog");
    let prolog_output = BufReader::new(prolog.stdout.take().expect("error reading prolog output"));
    let mut prolog_input = BufWriter::new(prolog.stdin.take().expect("error getting prolog input stream"));
    let mut output_lines = prolog_output.lines();
    let context = init_prolog_context(&state, unimplemented!());
    prolog_initial_defs(&mut prolog_input).unwrap();
    let initial_defs_written = read_true_false_another_class(&mut output_lines);
    match initial_defs_written {
        PrologOutput::True => {
            trace!("Initial prolog defs accepted by prolog.");
        }
        PrologOutput::False => { panic!() }
        NeedsAnotherClass(_) => { panic!() }
    }
    (prolog, prolog_input, output_lines, context)
}

#[allow(non_snake_case)]
enum PrologOutput {
    True,
    False,
    NeedsAnotherClass(String),
}

fn read_true_false_another_class(lines: &mut Lines<BufReader<ChildStdout>>) -> PrologOutput {
    //todo make sure regex follows official rules for java identifiers
    let need_to_load_regex = Regex::new("Need to load:('([A-Za-z/$_]+)'|([A-Za-z/$_]+))").expect("Error parsing regex.");
    loop {
        let cur = lines.next();
        let r = match cur {
            None => { panic!()/* continue*/ }
            Some(res) => { res }
        };
        let s = r.unwrap();
        if s.contains("true") {
            assert!(!s.contains("false"));
            return PrologOutput::True;
        } else if s.contains("false") {
            assert!(!s.contains("true"));
            dbg!("false");
            return PrologOutput::False;
        } else if need_to_load_regex.is_match(s.as_str()) {//todo pattern needs string const
            let captures = need_to_load_regex.captures(s.as_str()).unwrap();
            let class_name = captures.get(3).unwrap().as_str().to_string();
            return PrologOutput::NeedsAnotherClass(class_name);
        }
    }
}

fn init_prolog_context<'s>(state: &'s JVMState, loading_in_progress: &Vec<ClassEntry>) -> PrologGenContext<'s> {
    let mut to_verify = Vec::new();
    for class_entry in loading_in_progress.iter() {
        add_to_verify(state, &mut to_verify, class_entry)
    }
//    for class_entry in &state.partial_load {
//        add_to_verify(state, &mut to_verify, class_entry)
//    }
    for class_entry in state.loaders[&"bl".to_string()].loaded.read().unwrap().keys().into_iter() {
        add_to_verify(state, &mut to_verify, class_entry)
    }
    let context: PrologGenContext<'s> = PrologGenContext { state, to_verify, extra: ExtraDescriptors { extra_method_descriptors: Vec::new(), extra_field_descriptors: Vec::new() } };
    (context)
}

fn add_to_verify(state: &JVMState, to_verify: &mut Vec<Arc<Classfile>>, class_entry: &ClassEntry) -> () {
    let path = state.indexed_classpath.get(class_entry).unwrap();
//    let mut p = ParsingContext { f: File::open(path).expect("This is a bug"), constant_pool: None };
    let class_file = parse_class_file(File::open(path).expect("This is a bug"));
    to_verify.push(class_file)
}

pub fn prolog_initial_defs(w: &mut dyn Write) -> Result<(), io::Error> {
    write!(w, "['src/verification/verification.pl'].\n")?;
    w.flush()?;
    Ok(())
}

pub mod unified_types;
pub mod prolog_info_writer;
pub mod code_writer;
