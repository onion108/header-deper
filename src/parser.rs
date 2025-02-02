// vimtcompile:cargo test

fn eat_whitespace(source: &str) -> &str {
    for (pos, ch) in source.char_indices() {
        if !ch.is_whitespace() {
            return &source[pos..]
        }
    }
    source
}

fn eat_string(source: &str) -> &str {
    match_string(source).0
}

fn eat_comment(source: &str) -> &str {
    if source.starts_with("//") {
        let rem = &source[("//".len())..];
        let newline = rem.find('\n');
        if let Some(idx) = newline {
            &rem[idx..]
        } else {
            ""
        }
    } else if source.starts_with("/*") {
        let rem = &source[("/*".len())..];
        let newline = rem.find("*/");
        if let Some(idx) = newline {
            &rem[idx..]
        } else {
            ""
        }

    } else {
        source
    }
}


fn match_include(source: &str) -> Option<&str> {
    let mut res = source;
    while !res.starts_with("#include") {
        if res.is_empty() {
            return None
        }
        // TODO: Unicode thingy ([1..] isn't unicode safe)
        res = &res[1..];
        res = eat_whitespace(res);
        res = eat_string(res);
        res = eat_comment(res);
    }
    res = &res[("#include".len())..];
    Some(res)
}

/// Try match a string, returning remaining and the result.
fn match_string(source: &str) -> (&str, Option<String>) {
    let mut remain = eat_whitespace(source);
    if remain.is_empty() {
        return (remain, None)
    }

    // Match starting quote.
    if !remain.starts_with("\"") {
        return (remain, None)
    }
    remain = &remain[1..];

    // Match content of the string.
    enum State {
        Normal,
        Escape,
        End
    }
    let mut content = String::new();
    let mut state = State::Normal;
    let mut end_marker = None;
    for (i, ch) in remain.char_indices() {
        match state {
            State::Normal => {
                if ch == '\\' {
                    state = State::Escape;
                    continue;
                }

                if ch == '"' {
                    state = State::End;
                    continue;
                }

                content.push(ch);
            }
            State::Escape => {
                match ch {
                    'n' => content.push('\n'),
                    't' => content.push('\t'),
                    c => content.push(c)
                };
                state = State::Normal;
            }
            State::End => {
                end_marker = Some(i);
                break;
            }
        }
    }
    if let Some(marker_idx) = end_marker {
        (&remain[marker_idx..], Some(content))
    } else {
        if matches!(state, State::End) {
            ("", Some(content))
        } else {
            (remain, None)
        }
    }
}

fn match_include_line(source: &str) -> (&str, Option<String>, bool) {
    let rem = match_include(source);
    
    if let Some(rem) = rem {
        let (l, r) = match_string(rem);
        (l, r, true)
    } else {
        (source, None, false)
    }
}

pub fn parse_includes(source: &str) -> Vec<String> {
    let mut remain = eat_whitespace(source);
    let mut result = Vec::new();
    while let (rem, content, true) = match_include_line(remain) {
        remain = rem;
        if let Some(content) = content {
            result.push(content);
        }
    }
    result
}

#[allow(unused)]
mod test {
    use crate::parser::{eat_string, eat_whitespace, match_include, match_string, parse_includes};

    #[test]
    fn test_eat_whitespace() {
        assert_eq!(eat_whitespace("  k  "), "k  ");
        assert_eq!(eat_whitespace("tk  "), "tk  ");
        assert_eq!(eat_whitespace(""), "");
    }

    #[test]
    fn test_match_include() {
        assert_eq!(match_include("kaljbdflkjbaljfb"), None);
        assert_eq!(match_include("#include"), Some(""));
        assert_eq!(match_include("#includelkejfhqghbbqwhebdkhqbekjdhb"), Some("lkejfhqghbbqwhebdkhqbekjdhb"));
        assert_eq!(match_include(" d    dlkjh #includelkejfhqghbbqwhebdkhqbekjdhb"), Some("lkejfhqghbbqwhebdkhqbekjdhb"));
    }

    #[test]
    fn test_match_string() {
        assert_eq!(match_string("\"hello, world\"abc"), ("abc", Some("hello, world".to_string())));
        assert_eq!(match_string("\"hello, world\""), ("", Some("hello, world".to_string())));
        assert_eq!(match_string("\"hello, \\\\world\"abc"), ("abc", Some("hello, \\world".to_string())));
        assert_eq!(match_string("\"hello, \\\"world\"abc"), ("abc", Some("hello, \"world".to_string())));
        assert_eq!(match_string("  \"hello, \\\\world\"abc"), ("abc", Some("hello, \\world".to_string())));
        assert_eq!(match_string("a  \"hello, \\\\world\"abc").1, None);
    }

    #[test]
    fn test_eat_string() {
        assert_eq!(eat_string("\"abhdflasbfhabdf\\\"akjhljahdf\""), "");
        assert_eq!(eat_string("\"abhdflasbfhabdf\\\"akjhljahdf\"djalfjlh"), "djalfjlh");
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            parse_includes(r##"
            // Some comment here
            // #include "commented"
            #include <fuckyou2.h>
            #include "aime.h"
            #include "aime.c"
            #include <fuckyou.h>
            "there is #include "in_between string"fuck"
            "##),
            vec!["aime.h".to_string(), "aime.c".to_string()]
        );
    }
}

