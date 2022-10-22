#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(int_roundings)]
#![feature(box_syntax)]
#![feature(exclusive_range_pattern)]
#![feature(const_refs_to_cell)]
#![feature(strict_provenance_atomic_ptr)]
#![feature(once_cell)]

pub mod layout;
pub mod memory_regions;
pub mod early_startup;
pub mod allocated_object_types;
#[cfg(test)]
pub mod test;