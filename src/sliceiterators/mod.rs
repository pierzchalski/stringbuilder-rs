use std::io;
use std::mem;
use std::iter;

#[cfg(test)]
mod tests;

pub type Flatten<I, J> = iter::FlatMap<I, J, fn(J) -> J>;

pub fn flatten<I: Iterator<Item=J>, J: IntoIterator>(iter: I) -> Flatten<I, J> {
    fn id<A>(a: A) -> A { a };
    iter.flat_map(id)
}

pub struct Insert<'a, A: 'a, I, J> {
    original: I,
    insert: J,
    split: ::std::option::IntoIter<&'a [A]>,
    idx: usize,
    done: usize,
}

impl<'a, A: 'a, I, J> Insert<'a, A, I, J> {
    pub fn new(original: I, insert: J, idx: usize) -> Self {
        Insert {
            original: original,
            insert: insert,
            split: None.into_iter(),
            idx: idx,
            done: 0,
        }
    }
}

impl<'a, A: 'a, I, J> Iterator for Insert<'a, A, I, J> where
    I: Iterator<Item=&'a [A]>,
    J: Iterator<Item=&'a [A]>,
{
    type Item = &'a [A];
    fn next(&mut self) -> Option<Self::Item> {
        if self.done < self.idx {
            if let Some(next) = self.original.next() {
                self.done += next.len();
                if self.done > self.idx {
                    let excess = self.done - self.idx;
                    let head_len = next.len() - excess;
                    let head = &next[0..head_len];
                    let tail = &next[head_len..];
                    self.split = Some(tail).into_iter();
                    Some(head)
                } else {
                    Some(next)
                }
            } else {
                self.insert.next()
            }
        } else {
            self.insert.next().or_else(|| self.split.next()).or_else(|| self.original.next())
        }
    }
}

pub struct Skip<I> {
    iterator: I,
    count: usize,
    skipped: bool,
}

impl<'a, A: 'a, I> Iterator for Skip<I> where
    I: Iterator<Item=&'a [A]>,
{
    type Item = &'a [A];
    fn next(&mut self) -> Option<Self::Item> {
        if self.skipped {
            self.iterator.next()
        } else {
            self.skipped = true;
            let mut done = 0;
            loop {
                if let Some(next) = self.iterator.next() {
                    done += next.len();
                    if done > self.count {
                        let excess = done - self.count;
                        let head_len = next.len() - excess;
                        let tail = &next[head_len..];
                        return Some(tail);
                    }
                } else {
                    return None;
                }
            }
        }
    }
}

impl<I> Skip<I> {
    pub fn new(iterator: I, count: usize) -> Self {
        Skip {
            iterator: iterator,
            count: count,
            skipped: false,
        }
    }
}

pub struct Take<I> {
    iterator: I,
    done: usize,
    count: usize,
}

impl<'a, A: 'a, I> Iterator for Take<I> where
    I: Iterator<Item=&'a [A]>,
{
    type Item = &'a [A];
    fn next(&mut self) -> Option<Self::Item> {
        if self.done >= self.count {
            None
        } else {
            self.iterator.next().map(|next| {
                self.done += next.len();
                if self.done > self.count {
                    let excess = self.done - self.count;
                    let head_len = next.len() - excess;
                    &next[0..head_len]
                } else {
                    next
                }
            })
        }
    }
}

impl<I> Take<I> {
    pub fn new(iterator: I, count: usize) -> Self {
        Take {
            iterator: iterator,
            count: count,
            done: 0,
        }
    }
}

pub struct Reader<I: Iterator> {
    iterator: I,
    split: Option<I::Item>,
}

impl<'a, I> io::Read for Reader<I> where
    I: Iterator<Item=&'a [u8]>,
{
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        use ::std::io::Write;
        let mut count = 0;
        loop {
            if let Some(next) = mem::replace(&mut self.split, None).or_else(|| self.iterator.next()) {
                let done = try!(buf.write(next));
                count += done;
                if done < next.len() {
                    self.split = Some(&next[done..]);
                    return Ok(count);
                }
            } else {
                return Ok(count);
            }
        }
    }
}

impl<I: Iterator> Reader<I> {
    fn new(iterator: I) -> Self {
        Reader{
            iterator: iterator,
            split: None,
        }
    }
}
