use std::fmt::Display;
use std::iter::FromIterator;
use std::str::FromStr;

#[derive(Debug)]
pub struct InvalidNotationError(pub String);

impl Display for InvalidNotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum Side {
    White(String),
    Black(String),
}

impl Side {
    fn mov_ref(&self) -> &str {
        match self {
            Side::White(mov) | Side::Black(mov) => mov,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pgn(Vec<String>);

impl Pgn {
    pub fn moves(&self) -> &Vec<String> {
        &self.0
    }
}

impl Display for Pgn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buff = String::new();
        for i in self.moves() {
            buff.push_str(&format!("{} ", i));
        }
        buff.pop();
        write!(f, "{buff}")
    }
}

impl Pgn {
    fn validate_move(mov: Side) -> Result<String, String> {
        match mov {
            Side::White(str) if str == "O-O" => Ok("e1g1".into()),
            Side::White(str) if str == "O-O-O" => Ok("e1c1".into()),

            Side::Black(str) if str == "O-O" => Ok("e8g8".into()),
            Side::Black(str) if str == "O-O-O" => Ok("e8c8".into()),

            Side::White(str) | Side::Black(str)
                if ["o-o", "o-o-o", "0-0", "0-0-0"].iter().any(|a| a == &str) =>
            {
                Err(format!("expected O-O or O-O-O, got {str}"))
            }
            _ => Self::validate_mov_chars(mov.mov_ref()),
        }
    }

    fn validate_mov_chars(mov: &str) -> Result<String, String> {
        let sanitized = Self::sanitize_move(mov.to_string());
        Self::validate_sanitized_move(&sanitized)?;
        Ok(sanitized)
    }

    fn sanitize_move(mut mov: String) -> String {
        if mov.chars().next().is_some_and(|c| c.is_uppercase()) {
            mov.remove(0);
        }
        mov.retain(|c| !"x+#=-".contains(c));
        mov
    }

    fn validate_sanitized_move(mov: &str) -> Result<(), String> {
        if mov.len() < 4 || mov.len() > 5 {
            return Err(format!("expected {mov} to have length of 4 or 5"));
        }

        let errors = mov.chars().enumerate().map(|(idx, character)| {
            if (idx == 0 || idx == 2) && !Self::is_valid_file(character) {
                return Err(format!(
                    "first and third char must be any character between a-h, but got {character}"
                ));
            }
            if (idx == 1 || idx == 3) && !Self::is_valid_rank(character) {
                return Err(format!(
                    "second and fourth char must be any digit between 1-9, but got {character}"
                ));
            }
            if idx == 4 {
                let promotion = character.to_ascii_lowercase();
                if !matches!(promotion, 'q' | 'r' | 'b' | 'n') {
                    return Err(format!(
                        "fifth char must be one of q/r/b/n, but got {promotion}"
                    ));
                }
            }

            Ok(())
        }).filter_map(Result::err).fold(String::new(), |mut acc, err| {
            if !acc.is_empty() {
                acc.push('\n');
            }
            acc.push_str(&err);
            acc
        });

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_valid_file(c: char) -> bool {
        matches!(c, 'a'..='h')
    }

    fn is_valid_rank(c: char) -> bool {
        c.to_digit(10)
            .is_some_and(|digit| (1..=10).contains(&digit))
    }
}

impl FromIterator<String> for Pgn {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Pgn(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a str> for Pgn {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Pgn(iter.into_iter().map(|s| s.to_string()).collect())
    }
}

impl FromStr for Pgn {
    type Err = InvalidNotationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (moves, errors) = s
            .split_whitespace()
            .filter(|s| {
                if *s == "..." {
                    return false;
                }

                if s.ends_with('.') {
                    let maybe_num = s.strip_suffix('.').unwrap();
                    if maybe_num.parse::<u64>().is_ok() {
                        return false;
                    }
                }

                true
            })
            .enumerate()
            .map(|(i, raw_move)| {
                let raw_move = raw_move.to_string();
                let raw_move = if i % 2 == 0 {
                    Side::White(raw_move)
                } else {
                    Side::Black(raw_move)
                };

                Pgn::validate_move(raw_move)
                    .map_err(|e| InvalidNotationError(format!("{e}\nmove num:{}", i + 1)))
            })
            .fold((Vec::new(), String::new()), |mut acc, result| {
                match result {
                    Ok(mov) => acc.0.push(mov),
                    Err(e) => {
                        acc.1.push_str(&e.to_string());
                        acc.1.push('\n');
                    }
                }
                acc
            });

        if errors.is_empty() {
            Ok(Self(moves))
        } else {
            Err(InvalidNotationError(errors))
        }
    }
}
