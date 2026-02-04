use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn test_panic(i in 0..10) {
        assert!(i < 0);
    }
}
