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
    Build,
    Compile,
    Ignore,
    Clean,
    Empty,
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

fn tokenize_test_comment_content_impl(str: &str) -> Result<Vec<TestCommentToken>, TokenError> {
    let mut current_tokens = vec![];
    let mut current_str = str;
    let mut in_tag = false;
    'outer_loop: loop {
        if let Some(current_char) = current_str.chars().next() {
            match current_char {
                '@' => {
                    if in_tag{
                        current_tokens.push(TestCommentToken::CommentContentChar(current_char));
                        current_str = &current_str[1..];
                        continue;
                    }
                    in_tag = true;
                    if current_str.starts_with("@#") {
                        current_tokens.push(TestCommentToken::CommentContentChar('@'));
                        current_tokens.push(TestCommentToken::CommentContentChar('#'));
                        current_str = &current_str[2..];
                        continue 'outer_loop;
                    }

                    //todo handle this by stripping start from comment and only looking at tags at beggining of line
                    if current_str.starts_with("@code") {
                        for char in "@code".chars(){
                            current_tokens.push(TestCommentToken::CommentContentChar(char));
                        }
                        current_str = &current_str["@code".len()..];
                        continue 'outer_loop;
                    }
                    if current_str.starts_with("@sun") {
                        for char in "@sun".chars(){
                            current_tokens.push(TestCommentToken::CommentContentChar(char));
                        }
                        current_str = &current_str["@sun".len()..];
                        continue 'outer_loop;
                    }
                    let (token_type, string) = if current_str.starts_with("@test") {
                        (TestCommentTagToken::Test, "@test")
                    } else if current_str.starts_with("@bug") {
                        (TestCommentTagToken::Bug, "@bug")
                    } else if current_str.starts_with("@summary") {
                        (TestCommentTagToken::Summary, "@summary")
                    } else if current_str.starts_with("@author") {
                        (TestCommentTagToken::Author, "@author")
                    } else if current_str.starts_with("@comment") {
                        (TestCommentTagToken::Comment, "@comment")
                    } else if current_str.starts_with("@library") {
                        (TestCommentTagToken::Library, "@library")
                    } else if current_str.starts_with("@key") {
                        (TestCommentTagToken::Key, "@key")
                    } else if current_str.starts_with("@modules") {
                        (TestCommentTagToken::Modules, "@modules")
                    } else if current_str.starts_with("@requires") {
                        (TestCommentTagToken::Requires, "@requires")
                    } else if current_str.starts_with("@enablepreview") {
                        (TestCommentTagToken::EnablePreview, "@enablepreview")
                    } else if current_str.starts_with("@run") {
                        (TestCommentTagToken::Run, "@run")
                    } else if current_str.starts_with("@build") {
                        (TestCommentTagToken::Build, "@build")
                    } else if current_str.starts_with("@compile") {
                        (TestCommentTagToken::Compile, "@compile")
                    } else if current_str.starts_with("@ignore") {
                        (TestCommentTagToken::Ignore, "@ignore")
                    } else if current_str.starts_with("@clean") {
                        (TestCommentTagToken::Clean, "@clean")
                    } else if current_str.starts_with("@ ") {
                        (TestCommentTagToken::Empty, "@ ")
                    }else {
                        dbg!(current_str);
                        todo!()
                    };
                    current_tokens.push(TestCommentToken::Tag(token_type));
                    current_str = &current_str[string.len()..];
                }
                '\n' => {
                    in_tag = false;
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
                assert_eq!(bug_num, Some("4645302\n*".to_string()));
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
                assert_eq!(bug_num, Some("4902952 4905407 4916149 8057793\n *".to_string()));
                assert_eq!(summary, Some("Tests that the scale of zero is propagated properly and has the
 * proper effect and that setting the scale to zero does not mutate the
 * BigDecimal.
 *".to_string()));
                assert_eq!(author, Some("Joseph D. Darcy".to_string()));
            }
        }
    }
}
