#[derive(Debug, Copy, Clone)]
pub enum TestCommentTagToken {
    Test,
    Bug,
    Summary,
    Author,
    Comment,
    Library,
    Key,
    Modules,
    Requires,
    EnablePreview,
    Run,
}

#[derive(Debug)]
enum TestCommentToken {
    Tag(TestCommentTagToken),
    NewLine,
    CommentContentChar(char),
}

#[derive(Debug)]
pub enum TestCommentTokenJoined {
    Tag(TestCommentTagToken),
    NewLine,
    CommentContentString(String),
}

impl TestCommentTokenJoined {
    pub fn unwrap_newline(&self) {
        match self {
            TestCommentTokenJoined::NewLine => {}
            TestCommentTokenJoined::Tag(_) |
            TestCommentTokenJoined::CommentContentString(_) => {
                panic!()
            }
        }
    }

    pub fn unwrap_comment_content_string(&self) -> &str {
        match self {
            TestCommentTokenJoined::Tag(_) |
            TestCommentTokenJoined::NewLine => {
                panic!()
            }
            TestCommentTokenJoined::CommentContentString(string) => {
                string.as_str()
            }
        }
    }

    pub fn unwrap_tag(&self) -> TestCommentTagToken {
        match self {
            TestCommentTokenJoined::Tag(tag_) => {
                *tag_
            }
            TestCommentTokenJoined::NewLine |
            TestCommentTokenJoined::CommentContentString(_) => {
                panic!()
            }
        }
    }
}

const TOKEN_TYPES: phf::Map<&'static str, TestCommentTagToken> = phf::phf_map! {
    "@test" => TestCommentTagToken::Test,
    "@bug" => TestCommentTagToken::Bug,
    "@summary" => TestCommentTagToken::Summary,
    "@author" => TestCommentTagToken::Author,
    "@comment" => TestCommentTagToken::Comment,
    "@library" => TestCommentTagToken::Library,
    "@key" => TestCommentTagToken::Key,
    "@modules" => TestCommentTagToken::Modules,
    "@requires" => TestCommentTagToken::Requires,
    "@enablepreview" => TestCommentTagToken::EnablePreview,
    "@run" => TestCommentTagToken::Run,
};

fn tokenize_test_comment_content_impl(str: &str) -> Result<Vec<TestCommentToken>, TokenError> {
    let mut current_tokens = vec![];
    let mut current_str = str;
    'outer_loop: loop {
        if let Some(current_char) = current_str.chars().next() {
            match current_char {
                '@' => {
                    if &current_str[1..] == "@#" {
                        current_tokens.push(TestCommentToken::CommentContentChar('@'));
                        current_tokens.push(TestCommentToken::CommentContentChar('#'));
                        current_str = &current_str[2..];
                        continue 'outer_loop;
                    }
                    for (string, token_type) in &TOKEN_TYPES {
                        if current_str.starts_with(string) {
                            current_tokens.push(TestCommentToken::Tag(*token_type));
                            current_str = &current_str[string.len()..];
                            continue 'outer_loop;
                        }
                    }
                }
                '\n' => {
                    current_tokens.push(TestCommentToken::NewLine);
                    current_str = &current_str[1..];
                }
                _ => {
                    current_tokens.push(TestCommentToken::CommentContentChar(current_char));
                    current_str = &current_str[1..];
                }
            }
        } else {
            return Ok(current_tokens);
        }
    }
}

fn join_adjacent_tokens(to_join: Vec<TestCommentToken>) -> Vec<TestCommentTokenJoined> {
    let mut current_string = String::new();
    let mut res = vec![];
    for test_comment_token in to_join.into_iter() {
        if let TestCommentToken::Tag(_) | TestCommentToken::NewLine = &test_comment_token {
            if !current_string.is_empty() {
                res.push(TestCommentTokenJoined::CommentContentString(current_string.clone()));
                current_string.clear();
            }
        }

        match test_comment_token {
            TestCommentToken::Tag(test_comment_tag) => {
                res.push(TestCommentTokenJoined::Tag(test_comment_tag));
            }
            TestCommentToken::NewLine => {
                res.push(TestCommentTokenJoined::NewLine);
            }
            TestCommentToken::CommentContentChar(char_) => {
                current_string.push(char_);
            }
        }
    }
    res
}

pub fn tokenize_test_comment_content(content: &str) -> Result<Vec<TestCommentTokenJoined>, TokenError> {
    Ok(join_adjacent_tokens(tokenize_test_comment_content_impl(content)?))
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenError {}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::parse::{FileType, parse_java_test_file};
    use crate::ParsedOpenJDKTest;
    use crate::tokenize::{tokenize_test_comment_content};

    const EXAMPLE_TEST_1: &str = "
@test
* @bug 4645302
* @summary Socket with OP_WRITE would get selected only once
* @author kladko
";

    #[test]
    pub fn test_example_1() {
        let res = parse_java_test_file(PathBuf::new(), tokenize_test_comment_content(EXAMPLE_TEST_1).unwrap()).unwrap();
        match res {
            ParsedOpenJDKTest::Test { file_type, bug_num, summary, author, .. } => {
                assert_eq!(file_type, FileType::Java);
                assert_eq!(bug_num, Some(vec![4645302]));
                assert_eq!(summary, Some("Socket with OP_WRITE would get selected only once\n*".to_string()));
                assert_eq!(author, Some("kladko".to_string()));
            }
        }
    }

    const EXAMPLE_TEST_2: &str = "/*
 * @test
 * @bug 4902952 4905407 4916149 8057793
 * @summary Tests that the scale of zero is propagated properly and has the
 * proper effect and that setting the scale to zero does not mutate the
 * BigDecimal.
 * @author Joseph D. Darcy
 */";

    #[test]
    pub fn test_example_2() {
        let res = parse_java_test_file(PathBuf::new(), tokenize_test_comment_content(EXAMPLE_TEST_2).unwrap()).unwrap();
        match res {
            ParsedOpenJDKTest::Test { file_type, bug_num, summary, author, .. } => {
                assert_eq!(file_type, FileType::Java);
                assert_eq!(bug_num, Some(vec![4902952,4905407,4916149,8057793]));
                assert_eq!(summary, Some("Tests that the scale of zero is propagated properly and has the
 * proper effect and that setting the scale to zero does not mutate the
 * BigDecimal.
 *".to_string()));
                assert_eq!(author, Some("Joseph D. Darcy".to_string()));
            }
        }
    }
}
