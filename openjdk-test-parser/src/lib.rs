use std::path::PathBuf;
use crate::parse::FileType;


#[derive(Debug)]
pub enum ParsedOpenJDKTest {
    Test {
        file_type: FileType,
        defining_file_path: PathBuf,
        bug_num: Option<Vec<u64>>,
        summary: Option<String>,
        author: Option<String>,
    }
}

//https://openjdk.org/jtreg/tag-spec.html
pub mod tokenize;
pub mod parse;
