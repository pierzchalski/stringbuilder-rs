use ::sliceiterators::Skip;
use ::sliceiterators::Reader;
use ::std::io::Read;

#[test]
fn simple_drop() {
    let i = vec!(vec!(1, 2, 3), vec!(4, 5, 6), vec!(7, 8, 9));
    let i = (&i).into_iter().map(|v| &v[..]);
    let i = Skip::new(i, 5);
    let mut result = vec!();
    Reader::new(i).read_to_end(&mut result).unwrap();
    assert_eq!(result, vec!(6, 7, 8, 9));
}

mod benchmarks {
    use ::sliceiterators::Skip;
    use ::sliceiterators::Reader;
    use ::std::io::Read;

    extern crate test;

    struct Test(Vec<Vec<u8>>);
    impl Test {
        fn new() -> Self {
            Test(vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]])
        }
        fn slices<'a>(&'a self) -> Box<Iterator<Item=&'a [u8]> + 'a> {
            Box::new(self.0.iter().map(|v| &v[..]))
        }
    }

    #[bench]
    fn slice_iter(b: &mut test::Bencher) {
        let test = Test::new();

        let mut res = vec![];

        b.iter(|| {
            Reader::new(Skip::new(test.slices(), 5)).read_to_end(&mut res).unwrap();
        });

        assert_eq!(res, vec![5, 6, 7, 8]);
    }

    #[bench]
    fn byte_iter(b: &mut test::Bencher) {
        let test = Test::new();

        let mut res: Vec<u8> = vec![];

        b.iter(|| {
            res = test.slices().flat_map(|slice| slice.iter().map(|u| *u)).skip(5).collect();
        });

        assert_eq!(res, vec![5, 6, 7, 8]);
    }
}
