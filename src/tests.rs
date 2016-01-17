use super::Concat;
use super::Builder;

#[test]
fn simple_concat() {
    let result = "hello, world!".to_owned();
    let s1 = "hello, ";
    let s2 = "world!".to_owned();
    let c: Concat<&'static str, String> = Concat(s1, s2);
    let s1: String = c.chars().collect();
    let s2: String = c.chars().collect();
    let s3: String = c.build();

    assert_eq!(s1, result);
    assert_eq!(s2, result);
    assert_eq!(s3, result);
}

#[test]
fn concat_ext() {
    let s1 = "hello, ";
    let s2 = "world!".to_owned();

    let res1 = "hello, world!".to_owned();
    let res2 = "world!hello, ".to_owned();

    assert_eq!(s1.concat(&s2).build(), res1);
    assert_eq!(s2.concat(s1).build(), res2);
}

#[test]
fn multiple_ref() {
    let s1 = "yolo ";
    let s2 = "swagger ";
    let res = "yolo swagger yolo swagger ".to_owned();

    let b = s1.concat(s2);
    let b = b.concat(s1);
    let b = b.concat(s2);

    assert_eq!(b.build(), res);
}

#[test]
fn simple_insert() {
    let s1 = "hello,";
    let s2 = " weird ";
    let s3 = "world!";
    let s4 = "onderful";
    let s5 = ", w";
    let res = "hello, wonderful, weird world!".to_owned();

    let b = s1.concat(s2);
    let b = b.concat(s3);
    let b = b.insert(s4.concat(s5), 8);

    println!("{:?}", b);

    assert_eq!(b.build(), res);
}

#[cfg(feature = "benchmarks")]
mod benchmarks {
    extern crate test;

    const STR_COUNT: usize = 1<<15;
    const TEST_STR: &'static str = "yolo swagger";

    #[bench]
    fn insert_chars(b: &mut test::Bencher) {
        let vs = vec![TEST_STR.to_owned(); STR_COUNT];
        let mut res = String::with_capacity(TEST_STR.len() * STR_COUNT);

        b.iter(|| {
            for s in vs.iter() {
                for c in s.chars() {
                    res.push(c);
                }
            }
        });
    }

    #[bench]
    fn insert_slices(b: &mut test::Bencher) {
        let vs = vec![TEST_STR.to_owned(); STR_COUNT];
        let mut res = String::with_capacity(TEST_STR.len() * STR_COUNT);

        b.iter(|| {
            for s in vs.iter() {
                res.push_str(&*s);
            }
        });
    }
}
