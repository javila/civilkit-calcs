    let html = render("#settings {\"isUs\": false}\nr = 100ton*32.17404855643045ft/s^2");
    assert_contains("224", &html);
