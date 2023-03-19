use std::fmt::{Display, Formatter};

pub struct FullNameBuilder {
    group_name: String,
}

impl FullNameBuilder {
    pub fn new(group_name: impl Into<String>) -> Self {
        let group_name = group_name.into();
        Self {
            group_name,
        }
    }

    pub fn new_name(&self, human_name: impl Into<String>) -> Result<FullName,BadName> {
        let group_name = self.group_name.to_string();
        let human_name = human_name.into();
        if human_name.contains("."){
            return Err(BadName{})
        }
        Ok(FullName {
            group_name,
            human_name,
        })
    }
}

#[derive(Debug)]
pub struct BadName;

impl Display for BadName{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Names should not contain .")
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FullName {
    group_name: String,
    human_name: String,
}

