use itertools::Itertools;
use crate::{Bit, ClassID};
use std::simd::{u8x32};

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

    fn set_bit(target: &mut u8, bit_i: u8, set_to: Bit){
        assert!(bit_i <= 8);
        match set_to {
            Bit::Set => {
                *target |= 1u8 << bit_i
            }
            Bit::UnSet => {
                *target &= !(1u8 << bit_i)
            }
        }

    }

    pub fn to_bit_path256(&self) -> Result<BitPath256,DoesNotFit>{
        let bit_len = self.as_slice().len().try_into().map_err(|_|DoesNotFit)?;
        let mut bit_path = [0u8;32];
        let mut valid_mask = [0u8;32];
        for (i,bit) in self.as_slice().iter().enumerate(){
            let vec_i = i/8;
            let bit_i = (i % 8) as u8;
            Self::set_bit(bit_path.get_mut(vec_i).unwrap(), bit_i, *bit);
            Self::set_bit(valid_mask.get_mut(vec_i).unwrap(), bit_i, Bit::Set);
        }
        Ok(BitPath256{
            bit_len,
            valid_mask,
            bit_path
        })
    }
}

#[derive(Debug)]
pub struct DoesNotFit;


#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct BitPath256{
    pub valid_mask: [u8;32],
    pub bit_path: [u8;32],
    pub bit_len: u8,
}

impl BitPath256{
    pub fn bit_path_to_vector(&self) -> u8x32{
        u8x32::from_array(self.bit_path)
    }

    pub fn mask_to_vector(&self) -> u8x32{
        u8x32::from_array(self.valid_mask)
    }

    pub fn is_subpath_of(&self, super_path: &BitPath256) -> bool{
        if self.bit_len < super_path.bit_len {
            return false;
        }
        let super_path_mask = super_path.mask_to_vector();
        let super_path = super_path.bit_path_to_vector();
        let self_path = self.bit_path_to_vector();
        let compared = super_path ^ self_path;
        let should_be_all_zero = compared & super_path_mask;
        should_be_all_zero == u8x32::splat(0)
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


impl From<Vec<ClassID>>  for InheritanceClassIDPath<'_>{
    fn from(class_ids: Vec<ClassID>) -> Self {
        InheritanceClassIDPath::Owned { inner: class_ids }
    }
}

