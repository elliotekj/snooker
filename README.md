# Snooker - Lightweight spam detection for blog comments

[![Crates.io](https://meritbadge.herokuapp.com/snooker)](https://crates.io/crates/snooker)
[![Docs](https://docs.rs/snooker/badge.svg)](https://docs.rs/snooker)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/elliotekj/snooker/blob/master/LICENSE)

This crate provides a pure-Rust implementation of Jonathan Snook's [spam
detection
algorithm](https://snook.ca/archives/other/effective_blog_comment_spam_blocker)
for blog comments.

As described in the afore-linked post, it works on a points system. Points are
awarded and deducted based on a variety of rules. If a comments final score is
greater than or equal to 1, the comment is considered valid. If the comments
final score is 0 then it's considered to be worth of moderating. If the
comments final score is below 0 then it's considered to be spam. Each comment
starts with a score of 0.

## Installation

If you're using Cargo, just add Snooker to your `Cargo.toml`:

```toml
[dependencies]
snooker = "0.1.0"
```

## Example

Snooker gives the example comment below a score of **-10** based off of the following patterns:

- The `body` has less that 2 links in it: **+2 points**
- The `body` is more that 20 characters long but contains 1 link: **+1 point**
- The link in the `body` contains one keyword considered spammy ("free"): **-1
  point**
- The `body` contains one phrase considered spammy ("limited time only"): **-1
  point**
- The `body` starts with a word considered spammy when it's the first word of
  the comment ("nice"): **-10 points**
- The `author` field doesn't contain `http://` or `https://`: **+0 points**
  (unchanged)
- The `url` field contains a keyword considered spammy ("free"): **-1 point**
- None of the URLs use a TLD considered spammy: **+0 points** (unchanged)
- None of the URLs are longer that 30 characters: **+0 points** (unchanged)
- No consonant groups were found: **+0 points** (unchanged)
- No data was provided about the comments previously submitted with this email
  address: **+0 points** (unchanged)

```rust
use snooker::{Comment, Snooker, Status};

let comment = Comment {
    author: Some("Johnny B. Goode".to_string()),
    url: Some("http://my-free-ebook.com".to_string()),
    body: String::from("
        <p>Nice post! Check out our free (for a limited time only) eBook
        <a href=\"http://my-free-ebook.com\">here</a> that's totally relevant</p>
    "),
    previously_accepted_for_email: None,
    previously_rejected_for_email: None,
    previous_comment_bodies: None,
};

let snooker_result = Snooker::new(comment);
assert_eq!(snooker_result.score, -10);
assert_eq!(snooker_result.status, Status::Spam);
```

## License

Snooker is released under the MIT [`LICENSE`](/elliotekj/snooker/blob/master/LICENSE).

## About

This crate was written by [Elliot Jackson](https://elliotekj.com).

- Blog: [https://elliotekj.com](https://elliotekj.com)
- Email: elliot@elliotekj.com
