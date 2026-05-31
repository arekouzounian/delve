pub fn min<O: PartialOrd>(a: O, b: O) -> O {
    if a < b {
        return a;
    }

    b
}
