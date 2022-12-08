// #![feature(vec_into_raw_parts)]
//
// use std::ptr::NonNull;
// use std::sync::atomic::AtomicUsize;
// use atomic::Atomic;
//
// /// Eventually consistent atomic resizable vec.
//
//
// pub trait TypeIsAtomic{
//
// }
//
// impl TypeIsAtomic for u64 {
//
// }
//
// impl <T> TypeIsAtomic for NonNull<T> {
//
// }
//
// pub struct  NativeVec<T: TypeIsAtomic>{
//     inner: Atomic<NonNull<NativeVecInner<T>>>
// }
//
// impl <T: TypeIsAtomic> NativeVec<T>{
//     pub fn new() -> Self{
//         Self{
//             inner: Atomic::new(NonNull::new(Box::into_raw(Box::new(NativeVecInner::new(vec![])))).unwrap())
//         }
//     }
// }
//
//
// pub struct NativeVecInner<T: TypeIsAtomic>{
//     elems: *const Atomic<T>,
//     len: AtomicUsize,
//     capacity: usize
// }
//
// impl <T: TypeIsAtomic> NativeVecInner<T>{
//     pub fn new(values: Vec<Atomic<T>>) -> Self{
//         let (ptr, len, capacity) = values.into_raw_parts();
//         Self{
//             elems: ptr,
//             len: AtomicUsize::new(len),
//             capacity
//         }
//     }
//
//     pub fn resize(&self, ) {
//
//     }
// }
//
// //todo abandon this and back with a resizable mmap based vec b/c that seems the only safe way I can think of to do this atomically.
// // or just lose values