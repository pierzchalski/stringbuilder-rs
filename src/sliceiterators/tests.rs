use ::sliceiterators::{Skip, Reader, Insert, Take};
use ::std::io::Read;
use ::std::slice;
use ::std::iter;

#[derive(Debug)]
struct Test(Vec<Vec<u8>>);
impl Test {
    fn slices<'a>(&'a self) -> iter::Map<slice::Iter<'a, Vec<u8>>, fn(&Vec<u8>) -> &[u8]> {
        fn slice(v: &Vec<u8>) -> &[u8] {
            &v[..]
        }
        self.0.iter().map(slice)
    }
    fn new(n_rows: usize, n_cols: usize) -> (Self, Vec<u8>) {
        let mut vself = vec![];
        let mut vflat = vec![];
        let mut count = 0usize;
        for _ in 0..n_rows {
            let mut row = vec![];
            for _ in 0..n_cols {
                vflat.push(count as u8);
                row.push(count as u8);
                count += 1;
            }
            vself.push(row);
        }
        (Test(vself), vflat)
    }
}

#[test]
fn test_vecs_creation() {
    let (v, flat) = Test::new(2, 3);
    assert_eq!(v.0, vec![vec![0, 1, 2], vec![3, 4, 5]]);
    let vflat: Vec<u8> = v.slices().flat_map(|slice| slice.iter().cloned()).collect();
    assert_eq!(vflat, flat);
}

#[test]
fn simple_skip() {
    let (v, _) = Test::new(3, 3);
    let i = v.slices();
    let i = Skip::new(i, 5);
    let mut result = vec!();
    Reader::new(i).read_to_end(&mut result).unwrap();
    assert_eq!(result, vec!(5, 6, 7, 8));
}

#[test]
fn big_simple_skip() {
    let (test, flat) = Test::new(3, 10);

    for i in 0..flat.len() {
        let iter_flat: Vec<u8> = test.slices().flat_map(|slice| slice.iter().cloned()).skip(i).collect();
        let skip = test.slices();
        let skip: Vec<u8> = Skip::new(skip, i).flat_map(|slice| slice.iter().cloned()).collect();
        let expected: Vec<u8> = flat.iter().cloned().skip(i).collect();
        assert_eq!(iter_flat, expected);
        assert_eq!(skip, expected);
    }
}

#[test]
fn big_simple_take() {
    let (test, flat) = Test::new(3, 4);

    for i in 0..flat.len() {
        let iter_flat: Vec<u8> = test.slices().flat_map(|slice| slice.iter().cloned()).take(i).collect();
        let take = test.slices();
        let take: Vec<u8> = Take::new(take, i).flat_map(|slice| slice.iter().cloned()).collect();
        let expected: Vec<u8> = flat.iter().cloned().take(i).collect();
        assert_eq!(iter_flat, expected);
        assert_eq!(take, expected);
    }
}

#[test]
fn simple_insert() {
    let (v, _) = Test::new(3, 3);
    let i = v.slices();
    let in1 = &[4, 3, 2];
    let in1 = Some(&in1[..]).into_iter();
    let in2 = &[3, 4];
    let in2 = Some(&in2[..]).into_iter();
    let i = Insert::new(i, in1, 6);
    let i = Insert::new(i, in2, 9);
    let mut result = vec!();
    Reader::new(i).read_to_end(&mut result).unwrap();
    assert_eq!(result, vec!(0, 1, 2, 3, 4, 5, 4, 3, 2, 3, 4, 6, 7, 8));
}

mod benchmarks {
    use ::sliceiterators::Skip;
    use ::sliceiterators::Reader;
    use ::std::io::Read;
    use super::Test;

    extern crate test;

    const N_ROWS: usize = 300;
    const N_COLS: usize = 300;
    const SKIPS: usize = 107;
    const SKIP_DIFF: usize = 841;

    #[bench]
    fn slice_iter(b: &mut test::Bencher) {
        let (v, flat) = Test::new(N_ROWS, N_COLS);

        b.iter(|| {
            for s in 0..SKIPS {
                let skip = s * SKIP_DIFF;
                let mut iter = Reader::new(Skip::new(v.slices(), skip));
                let mut res = vec![];
                iter.read_to_end(&mut res).unwrap();
                assert_eq!(&res[..], &flat[skip..]);
            }
        });
    }

    #[bench]
    fn byte_iter(b: &mut test::Bencher) {
        let (v, flat) = Test::new(N_ROWS, N_COLS);
        b.iter(|| {
            for s in 0..SKIPS {
                let skip = s * SKIP_DIFF;
                let iter = v.slices().flat_map(|slice| slice.iter().cloned()).skip(skip);
                let res: Vec<u8> = iter.collect();
                assert_eq!(&res[..], &flat[skip..]);
            }
        });
    }
}
