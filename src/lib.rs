#[cfg(test)]
mod tests{
    #[test]
    fn pass() {
        assert!(true);
    }
    

    #[test]
    fn fail() {
        panic!("Oh no!");
    }
}