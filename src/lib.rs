#[macro_use] extern crate lazy_static;
extern crate regex;

mod spam_phrases;

use regex::{Regex, Captures};

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Valid,
    Moderate,
    Spam,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub author: Option<String>,
    pub url: Option<String>,
    pub body: String,
    pub previously_accepted_for_email: Option<isize>,
    pub previously_rejected_for_email: Option<isize>,
    pub previous_comment_bodies: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Snooker {
    pub score: isize,
    pub status: Status,
    pub comment: Comment,
}

lazy_static! {
    // Matches links, capturing the value in their `href`:
    static ref A_TAG_RE: Regex = Regex::new(r#"<a[^>]*href=["']((https?://)?([\da-zA-Z.-]+)\.([a-zA-Z]{2,10})[/]?([?]?[\S]*))["'][^>]*>"#).unwrap();
    static ref URL_RE: Regex = Regex::new(r#"((https?://)?([\da-zA-Z.-]+)\.([a-zA-Z]{2,10})[/]?([?]?[\S]*))"#).unwrap();

    // Matches 5 or more consonants in a row:
    static ref CONSONANTS_RE: Regex = Regex::new(r#"(?i)[b-z&&[^eiou]]{5,}"#).unwrap();

    // Matches all HTML tags:
    static ref HTML_TAGS_RE: Regex = Regex::new(r#"<[^>]*>"#).unwrap();
}

static SPAM_TLDS: [&str; 3] = ["de", "pl", "cn"];
static URL_SPAM_WORDS: [&str; 5] = [".html", ".info", "?", "&", "free"];
static BODY_SPAM_FIRST_WORDS: [&str; 4] = ["interesting", "sorry", "nice", "cool"];

impl Snooker {
    pub fn new(comment: Comment) -> Self {
        Snooker {
            score: 0,
            status: Status::Moderate,
            comment: comment,
        }
    }

    pub fn check_body_links(&mut self) -> i8 {
        let mut link_count: i8 = 0;
        let body_clone = self.comment.body.clone();

        for c in A_TAG_RE.captures_iter(&body_clone) {
            // Count the number of links
            link_count += 1;

            process_single_link(c, self);
        }

        if link_count < 2 {
            self.score += 2;
        } else {
            self.score -= link_count as isize;
        }

        link_count
    }

    pub fn check_url(&mut self) {
        let url_option = self.comment.clone().url;

        if let Some(url) = url_option {
            if let Some(c) = URL_RE.captures(&url) {
                process_single_link(c, self);
            };
        };
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

    pub fn check_body_for_spam_phrases(&mut self) {
        let mut spam_phrase_count: i8 = 0;

        for p in spam_phrases::SPAM_PHRASES.iter() {
            if self.comment.body.to_lowercase().contains(p) {
                spam_phrase_count += 1;
            }
        }

        self.score -= spam_phrase_count as isize;
    }

    pub fn check_body_first_word(&mut self) {
        let stripped = HTML_TAGS_RE.replace_all(&self.comment.body, "");
        let first_word = stripped.split_whitespace().next().unwrap().to_lowercase();

        for w in BODY_SPAM_FIRST_WORDS.iter() {
            if first_word.contains(w) {
                self.score -= 10;
            }
        }
    }

    pub fn check_body_of_previous_for_matches(&mut self) {
        if let Some(ref previous_comments) = self.comment.previous_comment_bodies {
            let lowercase_body = self.comment.body.trim().to_lowercase();

            for pc in previous_comments {
                let lowercase_pc = pc.trim().to_lowercase();

                if lowercase_pc == lowercase_body {
                    self.score -= 1;
                }
            }
        }
    }

    pub fn check_author_for_http(&mut self) {
        if let Some(ref a) = self.comment.author {
            if a.to_lowercase().contains("http://") || a.to_lowercase().contains("https://") {
                self.score -= 2;
            }
        }
    }

    pub fn count_emails_previous_statuses(&mut self) {
        if let Some(c) = self.comment.previously_accepted_for_email {
            self.score += c;
        }

        if let Some(c) = self.comment.previously_rejected_for_email {
            self.score -= c;
        }
    }
}

pub fn process_comment(comment: Comment) -> Snooker {
    let mut snooker = Snooker::new(comment);

    let link_count = snooker.check_body_links();
    snooker.check_body_length(link_count);
    snooker.check_body_for_spam_phrases();
    snooker.check_body_first_word();
    snooker.check_body_of_previous_for_matches();
    snooker.check_url();
    snooker.check_author_for_http();
    snooker.count_emails_previous_statuses();

    if snooker.score >= 1 {
        snooker.status = Status::Valid;
    } else if snooker.score == 0 {
        snooker.status = Status::Moderate;
    } else {
        snooker.status = Status::Spam;
    }

    snooker
}

pub fn count_consonant_collections(s: &str) -> u8 {
    let mut count = 0;

    for c in CONSONANTS_RE.captures_iter(s) {
        if &c[0] != "http" && &c[0] != "https" {
            count += 1;
        }
    }

    count
}

fn process_single_link(c: Captures, snooker: &mut Snooker) {
    // Check for certain TLDs

    let tld = &c[4];

    for spam_tld in SPAM_TLDS.iter() {
        if &tld == spam_tld {
            snooker.score -= 1 as isize;

            break;
        }
    }

    // Check for certains words & characters

    let url = &c[1];

    for word in URL_SPAM_WORDS.iter() {
        if url.to_lowercase().contains(word) {
            snooker.score -= 1 as isize;
        }
    }

    // Check the length of the URL:
    if url.len() > 30 {
        snooker.score -= 1 as isize;
    }

    // Check for 5 consonants or more in a row:
    snooker.score -= count_consonant_collections(url) as isize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spam_1() {
        // Author contains "https://" → -2
        // Body contains 2 links → -2
        // Body is over 20 chars with 2 links → +1
        // Body starts with "Cool" → -10
        // One of the body URLs has a spammy TLD → -1
        //
        // Expected: -14

        let comment = Comment {
            author: Some("https://elliotekj.com".to_string()),
            url: None,
            body: String::from("
                <p>Cool, this <a href=\"https://elliotekj.com\">comment</a> has more <a\
                href=\"https://elliotekj.de\">than</a> 20 characters in it but contains\
                2 links.</p>
            "),
            previously_accepted_for_email: None,
            previously_rejected_for_email: None,
            previous_comment_bodies: None,
        };

        let snooker_result = process_comment(comment);
        assert_eq!(snooker_result.score, -14);
        assert_eq!(snooker_result.status, Status::Spam);
    }

    #[test]
    fn spam_2() {
        // Body is over 20 chars and contains no links → +2
        // Body has less than 2 links → +2
        // Body contains 2 spam phrases → -2
        // URL has "free" and one param in it → -2
        // URL is over 30 characters → -1
        // 2 previous comments by this email address have the same body → -2
        //
        // Expected: -3

        let previous_comment_bodies = vec![
            String::from("
                <p>Have you been turned down? Get our special promotion</p>
            "),
            String::from("
                <p>Have you been turned down? Get our special promotion</p>
            "),
        ];

        let comment = Comment {
            author: Some("Elliot Jackson".to_string()),
            url: Some("http://someexample.com?getit=free".to_string()),
            body: String::from("
                <p>Have you been turned down? Get our special promotion</p>
            "),
            previously_accepted_for_email: None,
            previously_rejected_for_email: None,
            previous_comment_bodies: Some(previous_comment_bodies),
        };

        let snooker_result = process_comment(comment);
        assert_eq!(snooker_result.score, -3);
        assert_eq!(snooker_result.status, Status::Spam);
    }
}
