    let html = Render("#markdown\n- one\n  - two\n- three\n#end markdown", &[]);
    // Assert.Matches(@"<li>one\s*<ul>\s*<li>two</li>\s*</ul>\s*</li>", html)
    let squeezed: String = html.split_whitespace().collect();
    assert_contains("<li>one<ul><li>two</li></ul></li>", &squeezed);
    assert_equal(2.0, Count(&html, "<ul>"));
