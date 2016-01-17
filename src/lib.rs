#![cfg(all(feature = "benchmarks", test))]
#![feature(test)]
extern crate test;

use std::iter;

#[cfg(test)]
mod tests;

pub trait Builder<'a> {
    type CharsIter: Iterator<Item=char>;
    type StrsIter: Iterator<Item=&'a str>;
    fn len(&'a self) -> usize;
    fn chars(&'a self) -> Self::CharsIter;
    fn slices(&'a self) -> Self::StrsIter;
    fn build(&'a self) -> String {
        let mut s = String::with_capacity(self.len());
        for ss in self.slices() {
            s.push_str(ss);
        }
        s
    }
    fn concat<B2>(&'a self, other: B2) -> Concat<&'a Self, B2> { Concat(self, other) }
    fn insert<B2>(&'a self, other: B2, idx: usize) -> Insert<&'a Self, B2> {
        debug_assert!(idx < self.len());
        Insert {
            orig: self,
            insert: other,
            idx: idx,
        }
    }
}

impl<'a, T> Builder<'a> for &'a T where T: Builder<'a> {
    type CharsIter = <T as Builder<'a>>::CharsIter;
    type StrsIter = <T as Builder<'a>>::StrsIter;
    fn chars(&'a self) -> Self::CharsIter {
        (*self).chars()
    }
    fn slices(&'a self) -> Self::StrsIter {
        (*self).slices()
    }
    fn len(&'a self) -> usize {
        (*self).len()
    }
}

#[derive(Debug)]
pub struct Once<A>(Option<A>);

impl<A> Once<A> {
    pub fn new(a: A) -> Self {
        Once(Some(a))
    }
}

impl<A> Iterator for Once<A> {
    type Item = A;
    fn next(&mut self) -> Option<A> {
        std::mem::replace(&mut self.0, None)
    }
}

impl<'a> Builder<'a> for &'a str {
    type CharsIter = std::str::Chars<'a>;
    type StrsIter = Once<&'a str>;
    fn chars(&'a self) -> Self::CharsIter {
        (*self).chars()
    }
    fn slices(&'a self) -> Self::StrsIter {
        Once::new(self)
    }
    fn len(&'a self) -> usize {
        (*self).len()
    }
}

impl<'a> Builder<'a> for String {
    type CharsIter = std::str::Chars<'a>;
    type StrsIter = Once<&'a str>;
    fn chars(&'a self) -> Self::CharsIter {
        let s: &str = &*self;
        s.chars()
    }
    fn slices(&'a self) -> Self::StrsIter {
        Once::new(self)
    }
    fn len(&self) -> usize {
        self.len()
    }
}

#[derive(Debug)]
pub struct Concat<B1, B2>(B1, B2);

impl<'a, B1: 'a, B2: 'a> Builder<'a> for Concat<B1, B2> where
    B1: Builder<'a>,
    B2: Builder<'a>,
{
    type CharsIter = iter::Chain<<B1 as Builder<'a>>::CharsIter, <B2 as Builder<'a>>::CharsIter>;
    type StrsIter = iter::Chain<<B1 as Builder<'a>>::StrsIter, <B2 as Builder<'a>>::StrsIter>;
    fn chars(&'a self) -> Self::CharsIter {
        let Concat(ref b1, ref b2) = *self;
        b1.chars().chain(b2.chars())
    }
    fn slices(&'a self) -> Self::StrsIter {
        let Concat(ref b1, ref b2) = *self;
        b1.slices().chain(b2.slices())
    }
    fn len(&'a self) -> usize {
        self.0.len() + self.1.len()
    }
}

#[derive(Debug)]
pub struct Insert<B1, B2> {
    orig: B1,
    insert: B2,
    idx: usize,
}

pub struct InsertCharsIter<I1, I2> {
    orig: I1,
    insert: I2,
    n: usize,
}

pub struct InsertStrsIter<'a, I1, I2> {
    orig: I1,
    split: Option<&'a str>,
    insert: I2,
    idx: usize,
    orig_done: usize,
}

impl<'a, B1, B2> Builder<'a> for Insert<B1, B2> where
    B1: Builder<'a>,
    B2: Builder<'a>,
{
    type CharsIter = InsertCharsIter<B1::CharsIter, B2::CharsIter>;
    type StrsIter = InsertStrsIter<'a, B1::StrsIter, B2::StrsIter>;
    fn chars(&'a self) -> Self::CharsIter {
        InsertCharsIter {
            orig: self.orig.chars(),
            insert: self.insert.chars(),
            n: self.idx,
        }
    }
    fn slices(&'a self) -> Self::StrsIter {
        InsertStrsIter {
            orig: self.orig.slices(),
            split: None,
            insert: self.insert.slices(),
            idx: self.idx,
            orig_done: 0,
        }
    }
    fn len(&'a self) -> usize {
        self.orig.len() + self.insert.len()
    }
}

impl<I1, I2> Iterator for InsertCharsIter<I1, I2> where
    I1: Iterator<Item=char>,
    I2: Iterator<Item=char>,
{
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        if self.n > 0 {
            self.n -= 1;
            self.orig.next()
        } else {
            self.insert.next().or(self.orig.next())
        }
    }
}

impl<'a, I1, I2> Iterator for InsertStrsIter<'a, I1, I2> where
    I1: Iterator<Item=&'a str>,
    I2: Iterator<Item=&'a str>,
{
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.orig_done < self.idx {
            let slice = self.orig.next().unwrap();
            self.orig_done += slice.len();
            if self.orig_done <= self.idx {
                Some(slice)
            } else {
                let excess = self.orig_done - self.idx;
                let ret_len = slice.len() - excess;
                let head = &slice[0..ret_len];
                let tail = &slice[ret_len..];
                self.split = Some(tail);
                Some(head)
            }
        } else {
            self.insert.next()
                .or_else(|| std::mem::replace(&mut self.split, None))
                .or_else(|| self.orig.next())
        }
    }
}
