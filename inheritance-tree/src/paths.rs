use itertools::Itertools;
use crate::{Bit, ClassID};

#[derive(Debug)]
pub enum InheritanceTreePath<'a> {
    Owned {
        inner: Vec<Bit>
    },
    Borrowed {
        inner: &'a [Bit],
    },
}

impl<'a> InheritanceTreePath<'a> {
    pub fn concat<'any>(&self, other: &InheritanceTreePath) -> InheritanceTreePath<'any> {
        InheritanceTreePath::Owned {
            inner: self.as_slice().iter().cloned().chain(other.as_slice().iter().cloned()).collect_vec()
        }
    }

    pub fn as_slice(&self) -> &[Bit] {
        match self {
            InheritanceTreePath::Owned { inner } => {
                inner.as_slice()
            }
            InheritanceTreePath::Borrowed { inner } => {
                inner
            }
        }
    }

    pub fn to_owned(self) -> Vec<Bit> {
        match self {
            InheritanceTreePath::Owned {
                inner
            } => {
                inner
            }
            InheritanceTreePath::Borrowed { inner } => {
                inner.iter().cloned().collect_vec()
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    pub fn split_1(&self) -> (Bit, InheritanceTreePath) {
        (self.as_slice()[0], InheritanceTreePath::Borrowed { inner: &self.as_slice()[1..] })
    }
}


#[derive(Debug)]
pub enum InheritanceClassIDPath<'a> {
    Owned {
        inner: Vec<ClassID>,
    },
    Borrowed {
        inner: &'a [ClassID],
    },
}

impl<'a> InheritanceClassIDPath<'a> {
    pub fn from_owned(owned: Vec<ClassID>) -> Self {
        Self::Owned {
            inner: owned
        }
    }

    pub fn as_slice(&self) -> &[ClassID] {
        match self {
            InheritanceClassIDPath::Owned { inner } => {
                inner.as_slice()
            }
            InheritanceClassIDPath::Borrowed { inner } => {
                inner
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }


    pub fn split_1(&self) -> (ClassID, InheritanceClassIDPath) {
        (self.as_slice()[0], InheritanceClassIDPath::Borrowed { inner: &self.as_slice()[1..] })
    }
}
