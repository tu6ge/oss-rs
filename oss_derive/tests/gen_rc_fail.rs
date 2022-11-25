use oss_derive::oss_gen_rc;

fn main() {
    assert!(true);
}

struct ArcPointer;

#[allow(dead_code)]
struct Demo<T> {
    inner: T,
    string: &'static str,
}

impl<T> Demo<T> {
    fn new(inner: T, string: &'static str) -> Self {
        Self { inner, string }
    }
}

#[oss_gen_rc]
impl PartialEq<Demo<ArcPointer>> for Demo<ArcPointer> {
    fn eq(&self, other: &Demo<ArcPointer>) -> bool {
        self.string == other.string
    }
}

#[test]
fn test_arc_eq() {
    let val1 = Demo::new(ArcPointer, "foo1");
    let val2 = Demo::new(ArcPointer, "foo1");

    assert!(val1 == val2);
}

#[cfg(feature = "blocking")]
#[test]
fn test_rc_eq() {
    let val1 = Demo::new(RcPointer, "foo1");
    let val2 = Demo::new(RcPointer, "foo1");

    assert!(val1 == val2);
}
