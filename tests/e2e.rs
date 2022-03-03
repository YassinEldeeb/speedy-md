use speedy_md::Parser;
use std::fs;

#[test]
fn it_works() {
    let parser = Parser::new();

    let content =
        fs::read_to_string("./tests/fixtures/sample.md").expect("`./sample.md` has been deleted!");
    let html = parser.get_html(&content);

    let should_be = "<h1><b><em>I'm</em></b> super <b>C</b>hunky</h1><h1>I'm <b>super</b> <em>c</em>hunky</h1><h2><em>I</em>'m a <code>v</code>ery <code>big</code></h2><h2>I'm less <em><del><b>chu</b></del></em>nky</h2><h3>I'm kinda big</h3><h3>I'm kinda chunky</h3><h4>I'm not that big</h4><h5>I'm very small</h5><h6>Bro, what are you talking about</h6><p>Hey, what's going on **_ove_**r there? Yeah y'all 😡</p><p>This is my `code` btw.</p><blockquote><p>Yassin Eldeeb said:</p><p>I'm super dumb!</p></blockquote>";
    assert_eq!(should_be, html);
}
