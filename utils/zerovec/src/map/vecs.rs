// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::ule::*;
use crate::VarZeroVec;
use crate::ZeroVec;
use std::mem;

pub trait ZeroVecLike<'a, T> {
    type NeedleType: ?Sized;
    type GetType: ?Sized;
    fn binary_search(&self, k: &Self::NeedleType) -> Result<usize, usize>;
    fn get(&self, index: usize) -> Option<&Self::GetType>;
    fn insert(&mut self, index: usize, value: T);
    fn remove(&mut self, index: usize) -> T;
    fn replace(&mut self, index: usize, value: T) -> T;
    fn len(&self) -> usize;
}

impl<'a, T> ZeroVecLike<'a, T> for ZeroVec<'a, T>
where
    T: AsULE + Ord + Copy,
{
    type NeedleType = T;
    type GetType = T::ULE;
    fn binary_search(&self, k: &T) -> Result<usize, usize> {
        self.binary_search(k)
    }
    fn get(&self, index: usize) -> Option<&T::ULE> {
        self.get_ule_ref(index)
    }
    fn insert(&mut self, index: usize, value: T) {
        self.make_mut().insert(index, value.as_unaligned())
    }
    fn remove(&mut self, index: usize) -> T {
        T::from_unaligned(&self.make_mut().remove(index))
    }
    fn replace(&mut self, index: usize, value: T) -> T {
        let vec = self.make_mut();
        T::from_unaligned(&mem::replace(&mut vec[index], value.as_unaligned()))
    }
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'a, T> ZeroVecLike<'a, T> for VarZeroVec<'a, T>
where
    T: AsVarULE + Clone,
    T::VarULE: Ord,
{
    type NeedleType = T::VarULE;
    type GetType = T::VarULE;
    fn binary_search(&self, k: &T::VarULE) -> Result<usize, usize> {
        self.binary_search(k)
    }
    fn get(&self, index: usize) -> Option<&T::VarULE> {
        self.get(index)
    }
    fn insert(&mut self, index: usize, value: T) {
        self.make_mut().insert(index, value)
    }
    fn remove(&mut self, index: usize) -> T {
        self.make_mut().remove(index)
    }
    fn replace(&mut self, index: usize, value: T) -> T {
        let vec = self.make_mut();
        mem::replace(&mut vec[index], value)
    }
    fn len(&self) -> usize {
        self.len()
    }
}
