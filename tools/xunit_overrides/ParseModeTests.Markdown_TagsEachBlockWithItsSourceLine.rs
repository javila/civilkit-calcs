    let html = Render("#markdown\n# Title\n\nparagraph\n#end markdown", &[("debug", true)]);
    // Assert.Matches("<h1[^>]*data-source-line=\"2\"[^>]*>Title</h1>", html)
    let i = html.find("<h1").expect("no <h1");
    let tag_end = i + html[i..].find('>').expect("unclosed <h1");
    assert_contains("data-source-line=\"2\"", &html[i..tag_end]);
    assert!(html[tag_end..].starts_with(">Title</h1>"), "{}", html);
    // Assert.Matches("<p[^>]*data-source-line=\"4\"[^>]*>paragraph</p>", html)
    let i = html.find(">paragraph</p>").expect("no paragraph");
    let tag_start = html[..i].rfind("<p").expect("no <p");
    assert_contains("data-source-line=\"4\"", &html[tag_start..i]);
