    // Assert.Matches("<div[^>]*data-source-line=\"2\"", ...)
    let html = Render("#html\n<div>x</div>\n#end html", &[("debug", true)]);
    let i = html.find("<div").expect("no <div");
    let tag_end = html[i..].find('>').map(|j| i + j).unwrap_or(html.len());
    assert_contains("data-source-line=\"2\"", &html[i..tag_end]);
