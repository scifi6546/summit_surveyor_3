use super::Grid;
use std::marker::PhantomData;
#[derive(PartialEq, Debug)]
pub enum Context {
    XDimension,
    YDimension,
    MaxHeight,
    Height,
}
#[derive(PartialEq, Debug)]
pub enum ParseError {
    InvalidMagicNumber(String),
    EmptyFile,
    InvalidNumber { context: Context, error: String },
    MissingXDimension,
    MissingYDimension,
    MissingMaxHeight,
    MissingDatapoint,
}
pub fn terrain_from_pgm<S>(data: String) -> Result<Grid<u32, S>, ParseError> {
    let mut iter = SkipWhitespace::new(data.as_str());
    if let Some(magic_number) = iter.next() {
        if magic_number != "P2" {
            return Err(ParseError::InvalidMagicNumber(magic_number));
        }
    } else {
        return Err(ParseError::EmptyFile);
    }
    let x_dimension_string = if let Some(s) = iter.next() {
        s
    } else {
        return Err(ParseError::MissingXDimension);
    };
    let y_dimension_string = if let Some(s) = iter.next() {
        s
    } else {
        return Err(ParseError::MissingYDimension);
    };
    let dim_x: usize = if let Ok(x) = x_dimension_string.parse() {
        x
    } else {
        return Err(ParseError::InvalidNumber {
            context: Context::XDimension,
            error: x_dimension_string,
        });
    };
    let dim_y: usize = if let Ok(y) = y_dimension_string.parse() {
        y
    } else {
        return Err(ParseError::InvalidNumber {
            context: Context::YDimension,
            error: y_dimension_string,
        });
    };
    let max_height_string = if let Some(s) = iter.next() {
        s
    } else {
        return Err(ParseError::MissingMaxHeight);
    };
    let _max_height: usize = if let Ok(h) = max_height_string.parse() {
        h
    } else {
        return Err(ParseError::InvalidNumber {
            context: Context::MaxHeight,
            error: max_height_string,
        });
    };
    let mut data = vec![];
    data.reserve(dim_x * dim_y);
    for _x in 0..dim_x {
        for _y in 0..dim_y {
            let height_string = if let Some(s) = iter.next() {
                s
            } else {
                return Err(ParseError::MissingDatapoint);
            };
            let height: u32 = if let Ok(i) = height_string.parse() {
                i
            } else {
                return Err(ParseError::InvalidNumber {
                    context: Context::Height,
                    error: height_string,
                });
            };
            data.push(height);
        }
    }
    Ok(Grid {
        data,
        dim_x,
        dim_y,
        special_marker: PhantomData,
    })
}
///Iterator over whitespace skips comments and whitespace characters
struct SkipWhitespace<'a> {
    iter: std::iter::Peekable<std::str::Chars<'a>>,
}
impl<'a> SkipWhitespace<'a> {
    pub fn new(data: &'a str) -> Self {
        SkipWhitespace {
            iter: data.chars().peekable(),
        }
    }
}
impl<'a> SkipWhitespace<'a> {
    fn is_white_space(c: &char) -> bool {
        c == &'\n' || c == &' ' || c == &'\t'
    }
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.iter.peek() {
            if Self::is_white_space(c) {
                self.iter.next();
            } else {
                break;
            }
        }
    }
    fn skip_comment(&mut self) {
        if let Some(c) = self.iter.peek() {
            if c == &'#' {
                loop {
                    if let Some(c) = self.iter.peek() {
                        if c == &'\n' {
                            return;
                        } else {
                            self.iter.next();
                        }
                    }
                }
            }
        }
    }
    fn is_next_skippable(&mut self) -> bool {
        if let Some(c) = self.iter.peek() {
            Self::is_white_space(c) || c == &'#'
        } else {
            false
        }
    }
}
impl<'a> Iterator for SkipWhitespace<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        loop {
            if self.is_next_skippable() {
                self.skip_comment();
                self.skip_whitespace();
            } else {
                break;
            }
        }
        let mut string_out = String::new();
        loop {
            if self.is_next_skippable() {
                break;
            } else if let Some(c) = self.iter.next() {
                string_out.push(c);
            } else {
                break;
            }
        }
        if string_out.is_empty() {
            None
        } else {
            Some(string_out)
        }
    }
}
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_iterator() {
        let s = "s\ns2\n s3\n#do not read\ns4";
        let s_v: Vec<String> = SkipWhitespace::new(s).collect();
        assert_eq!(s_v, vec!["s", "s2", "s3", "s4"]);
    }
    #[test]
    fn test_iterator_spaces() {
        let s = "s1\n    s2";
        let s_v: Vec<String> = SkipWhitespace::new(s).collect();
        assert_eq!(s_v, vec!["s1", "s2"]);
    }
    #[test]
    fn test_iterator_pgm() {
        let s = "P2
    1 1
    10000
    10
            ";
        let s_v: Vec<String> = SkipWhitespace::new(s).collect();
        assert_eq!(s_v, vec!["P2", "1", "1", "10000", "10"]);
    }
    #[test]
    fn test_pgm_comment() {
        let s = "P2
    1 1
    #hello!
    10000
    10000
            ";
        let s_v: Vec<String> = SkipWhitespace::new(s).collect();
        assert_eq!(s_v, vec!["P2", "1", "1", "10000", "10000"]);
    }

    #[test]
    fn basic_terrain() {
        let terrain: Grid<u32, u8> = terrain_from_pgm(
            "P2
    1 1
    10000
    10000
            "
            .to_string(),
        )
        .expect("failed to parse");
        assert_eq!(terrain, Grid::from_val((1, 1), 10_000));
    }
    #[test]
    fn comment() {
        let terrain: Grid<u32, u8> = terrain_from_pgm(
            "P2
    1 1
    #hello!
    10000
    10000
            "
            .to_string(),
        )
        .expect("failed to parse");
        assert_eq!(terrain, Grid::from_val((1, 1), 10_000));
    }
}
