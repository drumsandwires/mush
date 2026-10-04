pub fn describe() -> String {
    "This is the mish-core library.".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = describe();
        assert_eq!(result, "This is the mish-core library.".to_owned());
    }
}
