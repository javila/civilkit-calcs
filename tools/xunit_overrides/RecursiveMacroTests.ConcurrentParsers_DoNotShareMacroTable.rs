    // C#: 32 Task.Run macro parsers, awaited together.
    let handles: Vec<_> = (0..32)
        .map(|i| {
            std::thread::spawn(move || {
                let macro_parser = TestMacroParser::new();
                let source = format!("#def only{i}$ = {i}\nonly{i}$\n");
                let (has_errors, expanded) = macro_parser.Parse(&source);
                assert!(!has_errors);
                assert_contains(&i.to_string(), &expanded);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
