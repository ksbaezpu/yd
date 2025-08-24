use regex::Error as RegexError;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};
use std::env;

fn is_chinese(text: &str) -> Result<bool, RegexError> {
    let re = Regex::new(r"[\u{4e00}-\u{9fff}]")?;
    Ok(re.is_match(text))
}

fn get_translation(word: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
        ),
    );

    let url = format!("https://www.youdao.com/result?word={}&lang=en", word);
    let response = match client.get(&url).headers(headers).send() {
        Ok(resp) => resp,
        Err(_) => return "Failed to fetch translation.".to_string(),
    };

    let html = match response.text() {
        Ok(text) => text,
        Err(_) => return "Failed to read response.".to_string(),
    };

    let document = Html::parse_document(&html);
    let mut translations = Vec::new();
    let mut phonetics = Vec::new();

    if let Ok(true) = is_chinese(word) {
        let word_exp_selector = Selector::parse("li.word-exp-ce.mcols-layout").unwrap();
        let point_selector = Selector::parse("a.point").unwrap();

        for exp in document.select(&word_exp_selector) {
            if let Some(word_text) = exp.select(&point_selector).next() {
                translations.push(word_text.text().collect::<String>());
            }
        }
    } else {
        let trans_container_selector = Selector::parse("div.trans-container").unwrap();
        let phone_selector = Selector::parse("div.per-phone").unwrap();
        let span_selector = Selector::parse("span").unwrap();
        let phonetic_selector = Selector::parse("span.phonetic").unwrap();
        let word_exp_selector = Selector::parse("li.word-exp").unwrap();
        let pos_selector = Selector::parse("span.pos").unwrap();
        let trans_selector = Selector::parse("span.trans").unwrap();

        if let Some(container) = document.select(&trans_container_selector).nth(0) {
            for phone_div in container.select(&phone_selector) {
                if let Some(label) = phone_div.select(&span_selector).next() {
                    let label_text = label.text().collect::<String>().trim().to_string();
                    if let Some(phonetic) = phone_div.select(&phonetic_selector).next() {
                        let phonetic_text = phonetic.text().collect::<String>().trim().to_string();
                        phonetics.push(format!("{} {}", label_text, phonetic_text));
                    }
                }
            }
        }

        if let Some(container) = document.select(&trans_container_selector).nth(1) {
            for exp in container.select(&word_exp_selector) {
                if let (Some(pos), Some(trans)) = (
                    exp.select(&pos_selector).next(),
                    exp.select(&trans_selector).next(),
                ) {
                    let pos_text = pos.text().collect::<String>().trim().to_string();
                    let trans_text = trans.text().collect::<String>().trim().to_string();
                    translations.push(format!("{}: {}", pos_text, trans_text));
                }
            }
        }
    }

    let output = if phonetics.is_empty() && translations.is_empty() {
        "No results.".to_string()
    } else {
        let phonetics_str = phonetics.join(" ");
        let translations_str = translations.join("\n");
        if phonetics_str.is_empty() {
            translations_str
        } else if translations_str.is_empty() {
            phonetics_str
        } else {
            format!("{}\n{}", phonetics_str, translations_str)
        }
    };
    output
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a word to translate");
        return;
    }
    println!("{}", get_translation(&args[1]));
}
