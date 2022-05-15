

pub mod auth;
pub mod bucket;
pub mod object;

pub fn abc()->i32{
    2
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
