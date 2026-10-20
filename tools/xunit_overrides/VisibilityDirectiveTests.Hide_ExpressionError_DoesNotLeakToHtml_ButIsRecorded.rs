    let mut parser = new_expression_parser(&[("Debug", true)]);
    parser.parse("#hide\nbad\n#end hide\naaa1 = 1", true);
    assert_does_not_contain("err", &parser.html_result);
    assert_contains("aaa1", &parser.html_result);
    assert_eq!(parser.errors_list.len(), 1);
    assert_contains("bad", &parser.errors_list[0].message);
