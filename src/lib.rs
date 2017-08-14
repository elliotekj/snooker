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
    static ref LINK_RE: Regex = Regex::new(r#"<a[^>]*href=["'](https?://)?([\da-zA-Z.-]+)\.([a-z.]{2,6})["'][^>]*>"#).unwrap();
}

static NAUGHTY_TLDS: [&str; 3] = ["de", "pl", "cn"];

impl Snooker {
    pub fn new(comment: Comment) -> Self {
        Snooker {
            score: 0,
            status: Status::Moderate,
            comment: comment,
        }
    }

    pub fn process_links(&mut self) {
        let mut link_count = 0;

        for c in LINK_RE.captures_iter(&self.comment.body) {
            // Count the number of links
            link_count += 1;

            // Check for certain TLDs

            let tld = &c[3];

            for naughty_tld in NAUGHTY_TLDS.iter() {
                if &tld == naughty_tld {
                    self.score -= 1;
                }
            }
        }

        if link_count <= 2 {
            self.score += 2;
        } else {
            self.score -= link_count;
        }

        println!("{}", self.score);
    }
}

pub fn process_comment(comment: Comment) -> Snooker {
    let mut snooker = Snooker::new(comment);

    snooker.process_links();

    snooker
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
            body: String::from("<p>This <a href=\"https://elliotekj.com\">comment</a> \
            has more <a href=\"https://elliotekj.de\">than</a> 20 characters in \
            it but <a href=\"https://elliotekj.com\">contains</a> 3 links.</p>"),
        };

        process_comment(comment);
    }
}
