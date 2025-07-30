pub fn happy_test() {
    assert_eq!(1, 1);
}

pub fn it_fails_intentionally() {
    assert_eq!(1, 2, "This is an intentional failure!");
}
