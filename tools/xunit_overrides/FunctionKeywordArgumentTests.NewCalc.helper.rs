// private static TestCalc NewCalc() => new(new());
fn NewCalc() -> TestCalc {
    TestCalc::new(ms())
}
