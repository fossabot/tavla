#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    /// A piece of un-emphasised speech.
    Normal(&'a str),
    /// A piece of emphasised speech.
    Emphasised(&'a str),
    // A pause with specified duration
    Pause(PauseDuration),
}

#[derive(Debug, PartialEq, Eq)]
pub enum PauseDuration {
    Sentence,
    Paragraph,
    Seconds(u32),
}

impl PauseDuration {
    fn lengthen(&self) -> Self {
        match self {
            PauseDuration::Sentence => PauseDuration::Paragraph,
            PauseDuration::Paragraph => PauseDuration::Seconds(1),
            PauseDuration::Seconds(secs) => PauseDuration::Seconds(2 * secs),
        }
    }
}

pub struct Tokenizer<'a> {
    rest: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Tokenizer<'a> {
        Tokenizer { rest: source }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.rest;
        let mut chars = self.rest.char_indices();

        let (first_consumed, first_char) = chars.next()?;
        let mut token = match first_char {
            '_' => Token::Emphasised(&rest[1..1]),
            '.' => Token::Pause(PauseDuration::Sentence),
            _ => Token::Normal(&rest[0..1]),
        };

        let mut last_consumed = first_consumed;
        let mut retain_last = false;
        while let Some((idx, character)) = chars.next() {
            last_consumed = idx;
            token = match character {
                '_' => {
                    match token {
                        Token::Emphasised(_) => break, // Swallow the underscore
                        _ => {
                            // Handle the underscore on next call and return pause or normal
                            retain_last = true;
                            break;
                        }
                    }
                }
                '.' => {
                    match token {
                        // Extend length of pause by doubling it
                        Token::Pause(duration) => Token::Pause(duration.lengthen()),
                        // Handle pause on next call
                        _ => {
                            retain_last = true;
                            break;
                        }
                    }
                }
                _ => match token {
                    Token::Emphasised(_) => Token::Emphasised(&rest[1..=idx]),
                    Token::Normal(_) => Token::Normal(&rest[0..=idx]),
                    Token::Pause(_) => {
                        retain_last = true;
                        break;
                    }
                },
            }
        }

        let consumed_until = if retain_last {
            last_consumed
        } else {
            chars.next().map(|(idx, _)| idx).unwrap_or(self.rest.len())
        };
        self.rest = &self.rest[consumed_until..];

        Some(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unemphasised_string() {
        let mut tokenizer = Tokenizer::new("holodrio");
        assert_eq!(
            tokenizer.next().expect("Expected a single token"),
            Token::Normal("holodrio")
        );
        assert_eq!(tokenizer.next(), None);
        assert!(tokenizer.next().is_none());
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn emphasised_string() {
        let mut tokenizer = Tokenizer::new("holodrio _there_");
        assert_eq!(tokenizer.next(), Some(Token::Normal("holodrio ")));
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("there")));
        assert_eq!(tokenizer.next(), None);
        assert!(tokenizer.next().is_none());
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn mixed_string() {
        let mut tokenizer = Tokenizer::new("plain ... _emph_. ..");
        assert_eq!(tokenizer.next(), Some(Token::Normal("plain ")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Seconds(1)))
        );
        assert_eq!(tokenizer.next(), Some(Token::Normal(" ")));
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("emph")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Sentence))
        );
        assert_eq!(tokenizer.next(), Some(Token::Normal(" ")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Paragraph))
        );
        assert_eq!(tokenizer.next(), None);
        assert_eq!(tokenizer.next(), None);
    }
}
