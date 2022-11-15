use std::path::PathBuf;
use crate::parse::FileType;


#[derive(Debug)]
pub enum ParsedOpenJDKTest {
    Test {
        file_type: FileType,
        defining_file_path: PathBuf,
        bug_num: Option<String>,
        summary: Option<String>,
        author: Option<String>,
        requires: Option<String>,
        run: Option<String>,
        comment: Option<String>,
        build: Option<String>,
        library: Option<String>,
        key: Option<String>,
        modules: Option<String>,
        compile: Option<String>,
        ignore: Option<String>,
        clean: Option<String>
    }
}

//https://openjdk.org/jtreg/tag-spec.html
pub mod tokenize;
pub mod parse;
