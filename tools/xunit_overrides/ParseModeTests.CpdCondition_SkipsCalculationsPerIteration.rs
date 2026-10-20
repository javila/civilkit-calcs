    let html = Render("zzz = 0\n#for i = 1 : 4\n#cpd i > 2\nzzz = zzz + 10\n#end cpd\n#loop\nyyy = zzz", &[]);
    // Assert.Matches(@"yyy[\s\S]*20", html)
    let i = html.find("yyy").expect("no yyy");
    assert_contains("20", &html[i..]);
    assert_does_not_contain("class=\"err\"", &html);
