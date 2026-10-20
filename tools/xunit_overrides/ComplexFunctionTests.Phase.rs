    // C#: calc.Run($"phase({_sqrt3.ToString(CultureInfo.InvariantCulture)}/2 + i/2)")
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run(format!("phase({}/2 + i/2)", calcpad_core::netfmt::double_to_string(_sqrt3())));
    let mut im = calc.Imaginary();
    assert_equal_tol(Math::PI / 6.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
