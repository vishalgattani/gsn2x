///
/// Wraps `s` at each `width`-th character adding `wrapstr` as a kind of line ending.
///
pub fn wordwrap(s: &str, width: u32, wrapstr: &str) -> String {
    let mut out = Vec::<String>::new();
    for line in s.lines() {
        let mut cur_line = String::new();
        for word in line.split_ascii_whitespace() {
            if cur_line.len() + word.len() > width as usize {
                out.push(cur_line);
                cur_line = String::new();
            } else if !cur_line.is_empty() {
                cur_line.push(' ');
            }
            cur_line.push_str(word);
        }
        if !cur_line.is_empty() {
            out.push(cur_line);
        }
    }
    out.join(wrapstr)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn simple() {
        let input = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.";
        let expected = concat!(
            "Lorem ipsum dolor sit amet, consetetur sadipscing\n",
            "elitr, sed diam nonumy eirmod tempor invidunt ut\n",
            "labore et dolore magna aliquyam erat, sed diam\n",
            "voluptua. At vero eos et accusam et justo duo\n",
            "dolores et ea rebum. Stet clita kasd gubergren, no\n",
            "sea takimata sanctus est Lorem ipsum dolor sit\n",
            "amet. Lorem ipsum dolor sit amet, consetetur\n",
            "sadipscing elitr, sed diam nonumy eirmod tempor\n",
            "invidunt ut labore et dolore magna aliquyam erat,\n",
            "sed diam voluptua. At vero eos et accusam et justo\n",
            "duo dolores et ea rebum. Stet clita kasd gubergren,\n",
            "no sea takimata sanctus est Lorem ipsum dolor sit\n",
            "amet."
        );
        let out = wordwrap(input, 50, "\n");
        assert_eq!(out, expected);
    }
    #[test]
    fn shorter() {
        let input = "Lorem ipsum dolor sit amet, consetetur";
        let expected = "Lorem ipsum dolor sit amet, consetetur".to_owned(); // make explicit heap allocation to prevent Short value
        let out = wordwrap(input, 50, "\n");
        assert_eq!(out, expected);
    }
    #[test]
    fn empty_line() {
        let input = " ";
        let expected = "".to_owned(); // make explicit heap allocation to prevent Short value
        let out = wordwrap(input, 50, "\n");
        assert_eq!(out, expected);
    }

    #[test]
    fn wrap_string() {
        let input = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt";
        let expected = concat!(
            "Lorem ipsum dolor sit amet, consetetur sadipscing<br align=\"left\"/>",
            "elitr, sed diam nonumy eirmod tempor invidunt",
        );
        let out = wordwrap(input, 50, "<br align=\"left\"/>");
        assert_eq!(out, expected);
    }

    #[test]
    fn with_newlines() {
        let input = "Lorem ipsum dolor sit amet,\nconsetetur sadipscing\nelitr, sed diam nonumy eirmod tempor invidunt";
        let expected = concat!(
            "Lorem ipsum dolor sit amet,<br align=\"left\"/>",
            "consetetur sadipscing<br align=\"left\"/>",
            "elitr, sed diam nonumy eirmod tempor invidunt",
        );
        let out = wordwrap(input, 50, "<br align=\"left\"/>");
        assert_eq!(out, expected);
    }
}
