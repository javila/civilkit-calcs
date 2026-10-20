    // C#: 32 Task.Run parsers with alternating isUs settings, awaited together.
    let handles: Vec<_> = (0..32)
        .map(|i| {
            std::thread::spawn(move || {
                let is_us = i % 2 == 0;
                let html = render(&format!("#settings {{\"isUs\": {}}}\nr = 100ton*32.17404855643045ft/s^2", if is_us { "true" } else { "false" }));
                assert_contains(if is_us { "200" } else { "224" }, &html);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
