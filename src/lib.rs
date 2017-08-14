#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::{Regex, Captures};

#[derive(Debug)]
pub enum Status {
    Valid,
    Moderate,
    Spam,
}

#[derive(Debug)]
pub struct Comment {
    pub author: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub body: String,
}

#[derive(Debug)]
pub struct Snooker {
    pub score: i8,
    pub status: Status,
    pub comment: Comment,
}

lazy_static! {
    // Matches links, capturing the value in their `href`:
    static ref LINK_RE: Regex = Regex::new(r#"<a[^>]*href=["']((https?://)?([\da-zA-Z.-]+)\.([a-zA-Z]{2,10})[/]?([?]?[\S]*))["'][^>]*>"#).unwrap();

    // Matches 5 or more consonants in a row:
    static ref CONSONANTS_RE: Regex = Regex::new(r#"(?i)[b-z&&[^eiou]]{5,}"#).unwrap();

    // Matches all HTML tags:
    static ref HTML_TAGS_RE: Regex = Regex::new(r#"<[^>]*>"#).unwrap();
}

static NAUGHTY_TLDS: [&str; 3] = ["de", "pl", "cn"];
static URL_SPAM_WORDS: [&str; 5] = [".html", ".info", "?", "&", "free"];

impl Snooker {
    pub fn new(comment: Comment) -> Self {
        Snooker {
            score: 0,
            status: Status::Moderate,
            comment: comment,
        }
    }

    pub fn process_links(&mut self) -> i8 {
        let mut link_count = 0;

        for c in LINK_RE.captures_iter(&self.comment.body) {
            // Count the number of links
            link_count += 1;

            // Check for certain TLDs

            let tld = &c[4];

            for naughty_tld in NAUGHTY_TLDS.iter() {
                if &tld == naughty_tld {
                    self.score -= 1;
                }
            }

            // Check for certains words & characters

            let url = &c[1];

            for word in URL_SPAM_WORDS.iter() {
                if url.contains(word) {
                    self.score -= 1;
                }
            }

            // Check the length of the URL:
            if url.len() > 30 {
                self.score -= 1;
            }

            // Check for 5 consonants or more in a row:
            self.score -= count_consonants(url) as i8;
        }

        if link_count <= 2 {
            self.score += 2;
        } else {
            self.score -= link_count;
        }

        link_count
    }

    pub fn check_body_length(&mut self, link_count: i8) {
        let stripped = HTML_TAGS_RE.replace_all(&self.comment.body, "");
        let trimmed_len = stripped.trim().len();

        if trimmed_len > 20 && link_count == 0 {
            self.score += 2;
        } else if trimmed_len > 20 {
            self.score += 1;
        } else {
            self.score -= 1;
        }
    }
}

pub fn process_comment(comment: Comment) -> Snooker {
    let mut snooker = Snooker::new(comment);

    let link_count = snooker.process_links();
    snooker.check_body_length(link_count);

    println!("{}", snooker.score);

    snooker
}

pub fn count_consonants(s: &str) -> u8 {
    let mut count = 0;

    for c in CONSONANTS_RE.captures_iter(s) {
        if &c[0] != "http" && &c[0] != "https" {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let comment = Comment {
            author: None,
            email: None,
            url: None,
            body: String::from("<p>This <a href=\"https://elliotekjjjj-free.com\">comment</a> has more <a href=\"https://elliotekj.de\">than</a> 20 characters in it but <a href=\"https://elliotekj.com?some=paramsthatmakethismorethanthirty\">contains</a> 3 links.</p>"),
        };

        process_comment(comment);
    }
}
