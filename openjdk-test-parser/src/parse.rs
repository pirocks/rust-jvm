use std::{fs, io};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::vec::IntoIter;
use itertools::{Itertools, peek_nth, PeekNth};
use crate::ParsedOpenJDKTest;
use crate::tokenize::{TestCommentTagToken, TestCommentTokenJoined, TokenError, tokenize_test_comment_content};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestParseError {
    #[error("no tests in file")]
    ContainsNoTest,
    #[error("incompatible file type")]
    IncompatibleFileType,
    #[error(transparent)]
    TokenError(#[from] TokenError),
    #[error("io error reading file")]
    IO(#[from] io::Error)
}


#[derive(Debug, Eq, PartialEq)]
pub enum FileType {
    Java,
    Bash,
    Html,
}

fn file_type_from_path(file_path: impl AsRef<Path>) -> Option<FileType> {
    Some(match file_path.as_ref().extension()?.as_bytes() {
        b"java" => {
            FileType::Java
        }
        b"sh" => {
            FileType::Bash
        }
        b"html" => {
            FileType::Html
        }
        _ => {
            return None
        }
    })
}

fn extract_comments_java(contents: &str) -> Vec<&str> {
    let mut res = vec![];
    let mut remaining = contents;
    loop {
        let comment_start = match remaining.find("/*") {
            None => {
                return res;
            }
            Some(comment_start) => {
                comment_start
            }
        };
        let comment_end = match remaining[comment_start..].find("*/") {
            None => {
                return res;
            }
            Some(comment_end) => {
                comment_end
            }
        };
        res.push(&remaining[comment_start..comment_end]);
        remaining = &remaining[(comment_end + 2)..];
    }
}

fn find_test_comment(comments: Vec<&str>) -> Result<&str, TestParseError> {
    comments.into_iter().find(|comment| comment.contains("@test")).ok_or(TestParseError::ContainsNoTest)
}

pub(crate) fn parse_java_test_file(file_path: PathBuf, tokens: Vec<TestCommentTokenJoined>) -> Result<ParsedOpenJDKTest, TestParseError>{
    let mut bug_nums = None;
    let mut summary = None;
    let mut author = None;


    let mut peekable_iter = peek_nth(tokens);
    loop {
        let token = match peekable_iter.next() {
            None => break,
            Some(token) => {
                token
            }
        };
        match token {
            TestCommentTokenJoined::Tag(tag) => {
                match tag {
                    TestCommentTagToken::Test => {
                        continue
                    }
                    TestCommentTagToken::Bug => {
                        bug_nums = Some(parse_bug(&mut peekable_iter));
                    }
                    TestCommentTagToken::Summary => {
                        summary = Some(parse_summary(&mut peekable_iter));
                    }
                    TestCommentTagToken::Author => {
                        author = Some(parse_author(&mut peekable_iter));
                    }
                    TestCommentTagToken::Comment => {
                        todo!()
                    }
                    TestCommentTagToken::Library => {
                        todo!()
                    }
                    TestCommentTagToken::Key => {
                        todo!()
                    }
                    TestCommentTagToken::Modules => {
                        todo!()
                    }
                    TestCommentTagToken::Requires => {
                        todo!()
                    }
                    TestCommentTagToken::EnablePreview => {
                        todo!()
                    }
                    TestCommentTagToken::Run => {
                        todo!()
                    }
                }
            }
            TestCommentTokenJoined::NewLine => {}
            TestCommentTokenJoined::CommentContentString(_) => {}
        }
    }

    Ok(ParsedOpenJDKTest::Test {
        file_type: FileType::Java,
        defining_file_path: file_path,
        bug_num: bug_nums,
        summary,
        author,
    })
}

fn parse_author(peekable_iter: &mut PeekNth<IntoIter<TestCommentTokenJoined>>) -> String {
    peekable_iter.next().unwrap().unwrap_comment_content_string().trim().to_string()
}

fn parse_bug(peekable_iter: &mut PeekNth<IntoIter<TestCommentTokenJoined>>) -> Vec<u64> {
    let numbers = peekable_iter.next().unwrap().unwrap_comment_content_string().trim().split(" ").map(|str| u64::from_str(str).unwrap()).collect_vec();
    let newline = peekable_iter.next().unwrap();
    newline.unwrap_newline();
    numbers
}

fn parse_summary(peekable_iter: &mut PeekNth<IntoIter<TestCommentTokenJoined>>) -> String {
    let mut summary = String::new();
    loop {
        if let None | Some(TestCommentTokenJoined::Tag(_)) = peekable_iter.peek() {
            break;
        }
        match peekable_iter.next().unwrap() {
            TestCommentTokenJoined::Tag(_) => {
                panic!()
            }
            TestCommentTokenJoined::NewLine => {
                summary.push('\n');
            }
            TestCommentTokenJoined::CommentContentString(string) => {
                summary.push_str(string.as_str());
            }
        }
    }
    summary.trim().to_string()
}

pub fn parse_test_file(file_path: PathBuf) -> Result<ParsedOpenJDKTest, TestParseError> {
    let file_type = file_type_from_path(file_path.as_path()).ok_or(TestParseError::IncompatibleFileType)?;
    match file_type {
        FileType::Java => {
            let contents = fs::read_to_string(file_path.as_path())?;
            let comments = extract_comments_java(contents.as_str());
            let test_comment = find_test_comment(comments)?;
            let tokens = tokenize_test_comment_content(test_comment)?;
            parse_java_test_file(file_path, tokens)
        }
        FileType::Bash => {
            todo!()
        }
        FileType::Html => {
            todo!()
        }
    }
}


#[cfg(test)]
pub mod test {}