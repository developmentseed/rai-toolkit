use regex::Regex;
use super::diacritics;
use std::collections::HashMap;
use geocoder_abbreviations::{Token, TokenType};
use crate::Context;

#[derive(Debug, PartialEq, Clone)]
pub struct Tokens {
    tokens: HashMap<String, ParsedToken>
}

impl Tokens {
    pub fn new(tokens: HashMap<String, ParsedToken>) -> Self {
        Tokens {
            tokens: tokens
        }
    }

    pub fn generate(languages: Vec<String>) -> Self {
        let import: HashMap<String, Vec<Token>> = geocoder_abbreviations::config(languages).unwrap();
        let mut map: HashMap<String, ParsedToken> = HashMap::new();

        for language in import.keys() {
            for group in import.get(language).unwrap() {
                // if it's a simple, non regex token replacer
                if !group.regex {
                    for tk in &group.tokens {
                        map.insert(
                            diacritics(&tk.to_lowercase()),
                            ParsedToken::new(diacritics(&group.canonical.to_lowercase()), group.token_type.to_owned())
                        );
                    }
                }
            }
        }

        Tokens {
            tokens: map
        }
    }

    pub fn process(&self, text: &String, country: &String) -> Vec<Tokenized> {
        let tokens = self.tokenize(&text);

        let mut tokenized: Vec<Tokenized> = Vec::with_capacity(tokens.len());

        for token in &tokens {
            match self.tokens.get(token) {
                None => {
                    tokenized.push(Tokenized::new(token.to_owned(), None));
                },
                Some(t) => {
                    tokenized.push(Tokenized::new(t.canonical.to_owned(), t.token_type.to_owned()));
                }
            };
        }
        if country == &String::from("US") {
            tokenized = type_us_st(&tokens, tokenized);
        }

        tokenized
    }

    ///
    /// Remove all diacritics, punctuation non-space whitespace
    /// returning a vector of component tokens
    ///
    fn tokenize(&self, text: &String) -> Vec<String> {
        let text = text.trim();

        lazy_static! {
            static ref UP: Regex = Regex::new(r"[\^]+").unwrap();

            // collapse apostraphes, periods
            static ref PUNC: Regex = Regex::new(r"[\u2018\u2019\u02BC\u02BB\uFF07'\.]").unwrap();

            // all other ascii and unicode punctuation except '-' per
            // http://stackoverflow.com/questions/4328500 split terms
            static ref SPACEPUNC: Regex = Regex::new(r#"[\u2000-\u206F\u2E00-\u2E7F\\'!"$#%&()*+,./:;<=>?@\[\]^_`{|}~-]"#).unwrap();

            static ref SPACE: Regex = Regex::new(r"\s+").unwrap();

            static ref IGNORE: Regex = Regex::new(r"(\d+)-(\d+)[a-z]?").unwrap();
        }

        let mut normalized = diacritics(&text.to_lowercase());

        normalized = UP.replace_all(normalized.as_str(), "").to_string();
        normalized = PUNC.replace_all(normalized.as_str(), "").to_string();
        normalized = SPACEPUNC.replace_all(normalized.as_str(), " ").to_string();
        normalized = SPACE.replace_all(normalized.as_str(), " ").to_string();

        let tokens: Vec<String> = normalized.split(" ").map(|split| {
            String::from(split)
        }).filter(|token| {
            // Remove Empty Tokens (Double Space/Non Trimmed Input)
            if token.len() == 0 {
                false
            } else {
                true
            }
        }).collect();

        tokens
    }
}

/// Simplified struct from geocoder_abbreviations::Token
/// @TODO replace with geocoder_abbreviations::Token when additional traits are derived
#[derive(Debug, PartialEq, Clone)]
pub struct ParsedToken {
    canonical: String,
    token_type: Option<TokenType>
}

impl ParsedToken {
    pub fn new(canonical: String, token_type: Option<TokenType>) -> Self {
        ParsedToken {
            canonical,
            token_type
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Tokenized {
    pub token: String,
    pub token_type: Option<TokenType>
}

impl Tokenized {
    pub fn new(token: String, token_type: Option<TokenType>) -> Self {
        Tokenized {
            token,
            token_type
        }
    }
}

///
/// Change 'st' token_type to TokenType::Way ('Street')  or None ('Saint')
///
pub fn type_us_st(tokens: &Vec<String>, mut tokenized: Vec<Tokenized>) -> Vec<Tokenized> {
    // check if original name contained "st"
    // don't modify if "street" or "saint" has already been tokenized
    if tokens.contains(&String::from("st")) {

        let mut st_index = Vec::new();
        let mut way_tokens = false;
        for (i, tk) in tokenized.iter().enumerate() {
            if tk.token == String::from("st") {
                st_index.push(i);
            }
            // if there are non-st ways
            else if tk.token_type == Some(TokenType::Way) {
                way_tokens = true;
            }
        }
        // all but the last 'st' are likely not ways
        let last = st_index.pop().unwrap();
        for i in st_index {
            tokenized[i].token_type = None;
        }
        // if there are no other way tokens, st => street
        if !way_tokens {
            tokenized[last].token_type = Some(TokenType::Way);
        // if there are non-st way tokens, st => saint
        } else {
            tokenized[last].token_type = None;
        }
    }
    tokenized
}

pub fn tokenize_name(name: String, context: Context) -> Vec<Tokenized> {
    context.tokens.process(&name, &context.country)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenized_string(tokenized: Vec<Tokenized>) -> String {
        let tokens: Vec<String> = tokenized
            .into_iter()
            .map(|x| String::from(x.token))
            .collect();
        let token_string = String::from(tokens.join(" ").trim());
        token_string
    }

    #[test]
    fn test_remove_diacritics() {
        let tokens = Tokens::new(HashMap::new());

        // diacritics are removed from latin text
        assert_eq!(tokenized_string(tokens.process(&String::from("Hérê àrë søme wöřdš, including diacritics and puncatuation!"), &String::from(""))), String::from("here are some words including diacritics and puncatuation"));

        // nothing happens to latin text
        assert_eq!(tokenized_string(tokens.process(&String::from("Cranberries are low, creeping shrubs or vines up to 2 metres (7 ft)"), &String::from(""))), String::from("cranberries are low creeping shrubs or vines up to 2 metres 7 ft"));

        // nothing happens to Japanese text
        assert_eq!(tokenized_string(tokens.process(&String::from("堪《たま》らん！」と片息《かたいき》になつて、喚《わめ》"), &String::from(""))), String::from("堪《たま》らん！」と片息《かたいき》になつて、喚《わめ》"));

        // greek diacritics are removed and other characters stay the same
        assert_eq!(tokenized_string(tokens.process(&String::from("άΆέΈήΉίΊόΌύΎ αΑεΕηΗιΙοΟυΥ"), &String::from(""))), String::from("άάέέήήίίόόύύ ααεεηηιιοουυ"));

        // cyrillic diacritics are removed and other characters stay the same
        assert_eq!(tokenized_string(tokens.process(&String::from("ўЎёЁѐЀґҐйЙ уУеЕеЕгГиИ"), &String::from(""))), String::from("ўўёёѐѐґґйй ууееееггии"));
    }

    #[test]
    fn test_tokenize() {
        let tokens = Tokens::new(HashMap::new());

        assert_eq!(tokenized_string(tokens.process(&String::from(""), &String::from(""))), String::from(""));

        assert_eq!(tokenized_string(tokens.process(&String::from("foo"), &String::from(""))), String::from("foo"));
        assert_eq!(tokenized_string(tokens.process(&String::from(" foo bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo bar "), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo-bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo+bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo_bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo:bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo;bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo|bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo}bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo{bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo[bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo]bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo(bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo)bar"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo b.a.r"), &String::from(""))), String::from("foo bar"));
        assert_eq!(tokenized_string(tokens.process(&String::from("foo's bar"), &String::from(""))), String::from("foos bar"));

        assert_eq!(tokenized_string(tokens.process(&String::from("San José"), &String::from(""))), String::from("san jose"));
        assert_eq!(tokenized_string(tokens.process(&String::from("A Coruña"), &String::from(""))), String::from("a coruna"));
        assert_eq!(tokenized_string(tokens.process(&String::from("Chamonix-Mont-Blanc"), &String::from(""))), String::from("chamonix mont blanc"));
        assert_eq!(tokenized_string(tokens.process(&String::from("Rue d'Argout"), &String::from(""))), String::from("rue dargout"));
        assert_eq!(tokenized_string(tokens.process(&String::from("Hale’iwa Road"), &String::from(""))), String::from("haleiwa road"));
        assert_eq!(tokenized_string(tokens.process(&String::from("москва"), &String::from(""))), String::from("москва"));
        assert_eq!(tokenized_string(tokens.process(&String::from("京都市"), &String::from(""))), String::from("京都市"));
    }

    #[test]
    fn test_replacement_tokens() {
        let mut map: HashMap<String, ParsedToken> = HashMap::new();
        map.insert(String::from("barter"), ParsedToken::new(String::from("foo"), None));
        map.insert(String::from("saint"), ParsedToken::new(String::from("st"), None));
        map.insert(String::from("street"), ParsedToken::new(String::from("st"), Some(TokenType::Way)));

        let tokens = Tokens::new(map);

        assert_eq!(tokens.process(&String::from("Main Street"), &String::from("")),
            vec![
                Tokenized::new(String::from("main"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);

        assert_eq!(tokens.process(&String::from("Main St"), &String::from("")),
            vec![
                Tokenized::new(String::from("main"), None),
                Tokenized::new(String::from("st"), None)
            ]);

        assert_eq!(tokens.process(&String::from("foobarter"), &String::from("")),
            vec![Tokenized::new(String::from("foobarter"), None)]);

        assert_eq!(tokens.process(&String::from("foo barter"), &String::from("")),
            vec![
                Tokenized::new(String::from("foo"), None),
                Tokenized::new(String::from("foo"), None)
            ]);
    }

    #[test]
    fn test_generate_tokens() {
        let tokens = Tokens::generate(vec![String::from("en")]);

        assert_eq!(tokens.process(&String::from("New Jersey Av NW"), &String::from("US")),
            vec![
                Tokenized::new(String::from("new"), None),
                Tokenized::new(String::from("jersey"), None),
                Tokenized::new(String::from("av"), Some(TokenType::Way)),
                Tokenized::new(String::from("nw"), Some(TokenType::Cardinal))
            ]);

        assert_eq!(tokens.process(&String::from("New Jersey Ave NW"), &String::from("US")),
            vec![
                Tokenized::new(String::from("new"), None),
                Tokenized::new(String::from("jersey"), None),
                Tokenized::new(String::from("av"), Some(TokenType::Way)),
                Tokenized::new(String::from("nw"), Some(TokenType::Cardinal))
            ]);

        assert_eq!(tokens.process(&String::from("New Jersey Avenue Northwest"), &String::from("US")),
            vec![
                Tokenized::new(String::from("new"), None),
                Tokenized::new(String::from("jersey"), None),
                Tokenized::new(String::from("av"), Some(TokenType::Way)),
                Tokenized::new(String::from("nw"), Some(TokenType::Cardinal))
            ]);

        assert_eq!(tokens.process(&String::from("Saint Peter Street"), &String::from("US")),
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);

        assert_eq!(tokens.process(&String::from("St Peter St"), &String::from("US")),
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);
    }

    #[test]
    fn test_type_us_st() {
        assert_eq!(
            type_us_st(&vec![String::from("")],
            vec![Tokenized::new(String::from(""), None)]),
            vec![Tokenized::new(String::from(""), None)]
        );

        // main st
        assert_eq!(
            type_us_st(&vec![String::from("main"), String::from("st")],
            vec![
                Tokenized::new(String::from("main"), None),
                Tokenized::new(String::from("st"), None)
            ]),
            vec![
                Tokenized::new(String::from("main"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);
        assert_eq!(
            type_us_st(&vec![String::from("main"), String::from("st")],
            vec![
                Tokenized::new(String::from("main"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]),
            vec![
                Tokenized::new(String::from("main"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);

        // st peter st
        assert_eq!(
            type_us_st(&vec![String::from("st"), String::from("peter"), String::from("st")],
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("st"), None)
            ]),
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);
        assert_eq!(
            type_us_st(&vec![String::from("st"), String::from("peter"), String::from("st")],
            vec![
                Tokenized::new(String::from("st"), Some(TokenType::Way)),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]),
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);

        // st peter
        assert_eq!(
            type_us_st(&vec![String::from("st"), String::from("peter")],
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
            ]),
            vec![
                Tokenized::new(String::from("st"), Some(TokenType::Way)),
                Tokenized::new(String::from("peter"), None),
            ]);
        assert_eq!(
            type_us_st(&vec![String::from("st"), String::from("peter")],
            vec![
                Tokenized::new(String::from("st"), Some(TokenType::Way)),
                Tokenized::new(String::from("peter"), None),
            ]),
            vec![
                Tokenized::new(String::from("st"), Some(TokenType::Way)),
                Tokenized::new(String::from("peter"), None),
            ]);

        // st peter av
        assert_eq!(
            type_us_st(&vec![String::from("st"), String::from("peter"), String::from("av")],
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("av"), Some(TokenType::Way))
            ]),
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("av"), Some(TokenType::Way))
            ]);
        assert_eq!(
            type_us_st(&vec![String::from("st"), String::from("peter"), String::from("av")],
            vec![
                Tokenized::new(String::from("st"), Some(TokenType::Way)),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("av"), Some(TokenType::Way))
            ]),
            vec![
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("peter"), None),
                Tokenized::new(String::from("av"), Some(TokenType::Way))
            ]);

        // rue st francois st
        assert_eq!(
            type_us_st(&vec![String::from("rue"), String::from("st"), String::from("francois"), String::from("st")],
            vec![
                Tokenized::new(String::from("rue"), None),
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("francois"), None),
                Tokenized::new(String::from("st"), None)
            ]),
            vec![
                Tokenized::new(String::from("rue"), None),
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("francois"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);
        assert_eq!(
            type_us_st(&vec![String::from("rue"), String::from("st"), String::from("francois"), String::from("st")],
            vec![
                Tokenized::new(String::from("rue"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way)),
                Tokenized::new(String::from("francois"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]),
            vec![
                Tokenized::new(String::from("rue"), None),
                Tokenized::new(String::from("st"), None),
                Tokenized::new(String::from("francois"), None),
                Tokenized::new(String::from("st"), Some(TokenType::Way))
            ]);
    }
}
