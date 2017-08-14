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
    pub email: Option<String>,
    pub url: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Breakdown {
    pub reason: String,
    pub weight: i8,
}

#[derive(Debug, Clone)]
pub struct Snooker {
    pub score: i8,
    pub status: Status,
    pub breakdown: Vec<Breakdown>,
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

static NAUGHTY_TLDS: [&str; 3] = ["de", "pl", "cn"];
static URL_SPAM_WORDS: [&str; 5] = [".html", ".info", "?", "&", "free"];
static BODY_SPAM_FIRST_WORDS: [&str; 4] = ["interesting", "sorry", "nice", "cool"];

impl Snooker {
    pub fn new(comment: Comment) -> Self {
        Snooker {
            score: 0,
            status: Status::Moderate,
            breakdown: Vec::new(),
            comment: comment,
        }
    }

    pub fn check_body_links(&mut self) -> i8 {
        let mut link_count = 0;
        let body_clone = self.comment.body.clone();

        for c in A_TAG_RE.captures_iter(&body_clone) {
            // Count the number of links
            link_count += 1;

            process_single_link(c, self);
        }

        if link_count < 2 {
            self.score += 2;

            self.breakdown.push(Breakdown {
                reason: "Body contains less than 2 links".to_string(),
                weight: 2,
            });
        } else {
            self.score -= link_count;

            self.breakdown.push(Breakdown {
                reason: "Body contains 2 or more links".to_string(),
                weight: -link_count,
            });
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

            self.breakdown.push(Breakdown {
                reason: "Body is over 20 chars and has 0 links".to_string(),
                weight: 2,
            });
        } else if trimmed_len > 20 {
            self.score += 1;

            self.breakdown.push(Breakdown {
                reason: "Body is over 20 chars and has at least 1 link".to_string(),
                weight: 1,
            });
        } else {
            self.score -= 1;

            self.breakdown.push(Breakdown {
                reason: "Body is under 20 chars".to_string(),
                weight: -1,
            });
        }
    }

    pub fn check_body_for_spam_phrases(&mut self) {
        let mut spam_phrase_count = 0;

        for p in spam_phrases::SPAM_PHRASES.iter() {
            if self.comment.body.to_lowercase().contains(p) {
                spam_phrase_count += 1;
            }
        }

        self.score -= spam_phrase_count;

        self.breakdown.push(Breakdown {
            reason: format!("Body contains {} spam phrases", spam_phrase_count),
            weight: -spam_phrase_count,
        });
    }

    pub fn check_body_first_word(&mut self) {
        let stripped = HTML_TAGS_RE.replace_all(&self.comment.body, "");
        let first_word = stripped.split_whitespace().next().unwrap().to_lowercase();

        for w in BODY_SPAM_FIRST_WORDS.iter() {
            if first_word.contains(w) {
                self.score -= 10;

                self.breakdown.push(Breakdown {
                    reason: format!("Body starts with spam word \"{}\"", w),
                    weight: -10,
                });
            }
        }

    }

    pub fn check_author_for_http(&mut self) {
        if let Some(ref a) = self.comment.author {
            if a.to_lowercase().contains("http://") || a.to_lowercase().contains("https://") {
                self.score -= 2;

                self.breakdown.push(Breakdown {
                    reason: "Author contains \"http://\" or \"https://\"".to_string(),
                    weight: -2,
                });
            }
        }
    }
}

pub fn process_comment(comment: Comment) -> Snooker {
    let mut snooker = Snooker::new(comment);

    let link_count = snooker.check_body_links();
    snooker.check_body_length(link_count);
    snooker.check_body_for_spam_phrases();
    snooker.check_body_first_word();
    snooker.check_url();
    snooker.check_author_for_http();

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

    for naughty_tld in NAUGHTY_TLDS.iter() {
        if &tld == naughty_tld {
            snooker.score -= 1;

            snooker.breakdown.push(Breakdown {
                reason: format!("Single URL contains spammy TLD \"{}\"", naughty_tld),
                weight: -1,
            });

            break;
        }
    }

    // Check for certains words & characters

    let url = &c[1];

    for word in URL_SPAM_WORDS.iter() {
        if url.to_lowercase().contains(word) {
            snooker.score -= 1;

            snooker.breakdown.push(Breakdown {
                reason: format!("Single URL contains spam word \"{}\"", word),
                weight: -1,
            });
        }
    }

    // Check the length of the URL:
    if url.len() > 30 {
        snooker.score -= 1;

        snooker.breakdown.push(Breakdown {
            reason: "Single URL is over 30 chars".to_string(),
            weight: -1,
        });
    }

    // Check for 5 consonants or more in a row:
    let consonant_count = count_consonant_collections(url) as i8;

    snooker.score -= consonant_count;

    if consonant_count > 0 {
        snooker.breakdown.push(Breakdown {
            reason: format!("String contains {} consonant groups", consonant_count),
            weight: -consonant_count,
        });
    }
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
            email: None,
            url: None,
            body: String::from("
                <p>Cool, this <a href=\"https://elliotekj.com\">comment</a> has more <a\
                href=\"https://elliotekj.de\">than</a> 20 characters in it but contains\
                2 links.</p>
            "),
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
        //
        // Expected: -1

        let comment = Comment {
            author: Some("Elliot Jackson".to_string()),
            email: None,
            url: Some("http://someexample.com?getit=free".to_string()),
            body: String::from("
                <p>Have you been turned down? Get our special promotion</p>
            "),
        };

        let snooker_result = process_comment(comment);

        assert_eq!(snooker_result.score, -1);
        assert_eq!(snooker_result.status, Status::Spam);
    }
}
