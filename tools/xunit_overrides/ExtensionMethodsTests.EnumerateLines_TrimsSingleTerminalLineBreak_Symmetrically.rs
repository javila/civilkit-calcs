    // [MemberData(nameof(EnumerateLinesCases))]
    let cases: [(&str, &[&str]); 17] = [
        ("", &[]),
        ("a", &["a"]),
        ("a\n", &["a"]),
        ("a\r\n", &["a"]),
        ("a\r", &["a"]),
        ("a\nb", &["a", "b"]),
        ("a\rb", &["a", "b"]),
        ("a\nb\n", &["a", "b"]),
        ("a\r\nb\r\n", &["a", "b"]),
        ("a\rb\r", &["a", "b"]),
        ("a\n\nb\n", &["a", "", "b"]),
        ("a\nb\n\n", &["a", "b", ""]),
        ("a\r\nb\r\n\r\n", &["a", "b", ""]),
        ("a\rb\r\r", &["a", "b", ""]),
        ("\n", &[""]),
        ("\r\n", &[""]),
        ("\r", &[""]),
    ];
    for (source, expected) in cases {
        let lines = calcpad_core::extension_methods::enumerate_lines(source);
        assert_eq!(expected, lines.as_slice(), "source {source:?}");
    }
