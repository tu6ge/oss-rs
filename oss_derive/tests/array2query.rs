use oss_derive::array2query;

fn main() {
    assert!(true);
}

struct Query(u8);

#[array2query(3)]
impl From<[u8; 1]> for Query {
    fn from(arr: [u8; 1]) -> Self {
        Self(arr.len() as u8)
    }
}

#[test]
fn test_from() {
    let query: Query = [1u8].into();
    assert!(query.0 == 1);

    let query: Query = [1u8, 2].into();
    assert!(query.0 == 2);

    let query: Query = [1u8, 2, 3].into();
    assert!(query.0 == 3);
}
