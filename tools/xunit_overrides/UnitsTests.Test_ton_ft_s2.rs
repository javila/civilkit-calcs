    // Settings.IsUs defaults to true (US); only ExpressionParser.Parse reads it.
    let html = render("r = 100ton*32.17404855643045ft/s^2");
    assert_contains("200", &html);
