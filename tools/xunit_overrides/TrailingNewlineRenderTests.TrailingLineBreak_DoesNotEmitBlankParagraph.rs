    // [MemberData(nameof(SingleTrailingLineBreakCases))]
    for source in ["5 = 5\n", "5 = 5\r\n", "5 = 5\r"] {
        let html = Render(source);
        assert_equal(0.0, CountOccurrences(&html, "<p>&nbsp;</p>"));
        assert_equal(0.0, CountOccurrences(&html, "<p></p>"));
    }
