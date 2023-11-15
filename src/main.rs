use std::env;
use std::io;
use std::process;

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Char(char),
    SpecialChar(char),
    UnionGroup(Vec<Re>),
    PositiveGroup(Vec<char>),
    NegativeGroup(Vec<char>),
    Start,
    End,
    ZeroOne,
    OneMore,
    ZeroOneGroup(Box<Token>),
    OneMoreGroup(Box<Token>),
    Any,
}

#[derive(Debug, PartialEq, Clone)]
struct Re(Vec<Token>);

struct Parser {
    chars: Vec<char>,
    current: usize,
}

impl Parser {
    fn new(input: &str) -> Parser {
        Parser {
            current: 0,
            chars: input.chars().collect(),
        }
    }

    fn peek(&self) -> char {
        self.chars[self.current]
    }

    fn get(&mut self) -> char {
        let ch = self.chars[self.current];
        self.current += 1;
        return ch;
    }

    fn parse_union_group_item(&mut self) -> Re {
        println!("begin union group item");
        let mut tokens = Vec::new();
        while self.peek() != '|' && self.peek() != ')' {
            tokens.push(self.parse_token());
        }

        if self.peek() == '|' {
            self.get();
        }

        println!("finished union group item");
        Re(tokens)
    }

    fn parse_union_group(&mut self) -> Token {
        println!("parsing union group");
        let mut group = Vec::new();
        while self.peek() != ')' {
            let group_item = self.parse_union_group_item();
            group.push(group_item);
        }
        self.get();
        return Token::UnionGroup(group);
    }

    fn parse_positive_token(&mut self) -> Token {
        println!("parsing positive/negative token");
        let mut group = Vec::new();
        if self.peek() == '^' {
            self.get();
            while self.peek() != ']' {
                group.push(self.get());
            }
            self.get();
            return Token::NegativeGroup(group);
        }

        while self.peek() != ']' {
            group.push(self.get());
        }
        self.get();
        return Token::PositiveGroup(group);
    }

    fn parse_token(&mut self) -> Token {
        match self.get() {
            '$' if self.current == self.chars.len() => Token::End,
            '^' if self.current == 1 => Token::Start,
            '\\' => match self.get() {
                ch => Token::SpecialChar(ch),
            },
            '(' => self.parse_union_group(),
            '[' => self.parse_positive_token(),
            '.' => Token::Any,
            '+' => Token::OneMore,
            '?' => Token::ZeroOne,
            ch => Token::Char(ch),
        }
    }

    fn parse(&mut self) -> Re {
        let mut tokens = Vec::new();

        while self.current < self.chars.len() {
            let token = self.parse_token();
            if token == Token::ZeroOne {
                let last = tokens.pop().unwrap();
                tokens.push(Token::ZeroOneGroup(Box::new(last)));
            } else if token == Token::OneMore {
                let last = tokens.pop().unwrap();
                tokens.push(Token::OneMoreGroup(Box::new(last)));
            } else {
                tokens.push(token);
            }
        }

        Re(tokens)
    }
}

struct Matcher<'a> {
    input: &'a [char],
    re: Re,
}

impl<'a> Matcher<'a> {
    fn match_re_and_input(&self, mut inp_ind: usize) -> (bool, usize) {
        let mut token_ind = 0;

        loop {
            if token_ind == self.re.0.len() {
                return (true, inp_ind);
            }

            if inp_ind == self.input.len() {
                return (false, inp_ind);
            }

            if (inp_ind == self.input.len()) || (token_ind == self.re.0.len()) {
                println!("one");
                return (false, 0);
            }

            let input_ch = &self.input[inp_ind];
            let token = &self.re.0[token_ind];

            match token {
                Token::Any => {
                    inp_ind += 1;
                    token_ind += 1;
                    continue;
                }
                Token::Char(ch) => {
                    if input_ch == ch {
                        inp_ind += 1;
                        token_ind += 1;
                        continue;
                    } else {
                        return (false, 0);
                    }
                }
                Token::NegativeGroup(group) => {
                    if group.contains(input_ch) {
                        return (false, 0);
                    } else {
                        inp_ind += 1;
                        token_ind += 1;
                        continue;
                    }
                }
                Token::PositiveGroup(group) => {
                    if group.contains(input_ch) {
                        inp_ind += 1;
                        token_ind += 1;
                        continue;
                    } else {
                        return (false, 0);
                    }
                }
                Token::ZeroOneGroup(token) => {
                    let (matched, ind) =
                        Matcher::mtch(&self.input[inp_ind..], Re(vec![*token.clone()]));
                    if matched {
                        inp_ind += ind;
                        token_ind += 1;
                        continue;
                    } else {
                        // inp_ind = input_ind;
                        token_ind += 1;
                        continue;
                    }
                }
                Token::OneMoreGroup(token) => {
                    // one
                    let (matched, ind) =
                        Matcher::mtch(&self.input[inp_ind..], Re(vec![*token.clone()]));

                    if matched == false {
                        return (false, 0);
                    } else {
                        inp_ind += ind;
                    }

                    // more
                    loop {
                        let (matched, ind) =
                            Matcher::mtch(&self.input[inp_ind..], Re(vec![*token.clone()]));
                        if matched {
                            inp_ind += ind;
                            continue;
                        } else {
                            token_ind += 1;
                            break;
                        }
                    }
                }
                Token::SpecialChar(ch) => match ch {
                    'w' => {
                        if input_ch.is_alphanumeric() {
                            inp_ind += 1;
                            token_ind += 1;
                            continue;
                        } else {
                            return (false, 0);
                        }
                    }
                    'd' => {
                        if input_ch.is_numeric() {
                            inp_ind += 1;
                            token_ind += 1;
                            continue;
                        } else {
                            return (false, 0);
                        }
                    }
                    ch => {
                        if input_ch == ch {
                            inp_ind += 1;
                            token_ind += 1;
                            continue;
                        } else {
                            return (false, 0);
                        }
                    }
                },
                Token::UnionGroup(groups) => {
                    let matched = groups.iter().any(|group| {
                        let (matched, ind) = Matcher::mtch(&self.input[inp_ind..], group.clone());
                        if matched {
                            inp_ind += ind;
                            token_ind += 1;
                        };
                        return matched;
                    });

                    if !matched {
                        return (false, 0);
                    }
                }
                _ => todo!(),
            }
        }
    }

    fn mtch(input: &[char], re: Re) -> (bool, usize) {
        let matcher = Matcher {
            input: input,
            re: re,
        };

        let (matched, ind) = matcher.match_re_and_input(0);
        if matched {
            return (matched, ind);
        }

        return (false, 0);
    }

    fn mtch_main(input: &[char], re: Re) -> (bool, usize) {
        let mut matcher = Matcher {
            input: input,
            re: re,
        };

        let mut end = input.len();
        if matcher.re.0[0] == Token::Start {
            end = 1;
            matcher.re.0.remove(0); 
        }


        let mut match_end = false;
        if (matcher.re.0.len() > 0) && (matcher.re.0[matcher.re.0.len() - 1] == Token::End) {
            match_end = true;
            matcher.re.0.pop();
        }

        for i in 0..end {
            let (matched, ind) = matcher.match_re_and_input(i);
            if matched {
                if match_end {
                    if ind == input.len() {
                        return (matched, ind);
                    } else {
                        return (false, 0)
                    }
                } 
                return (matched, ind);
            }
        }

        return (false, 0);
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let mut parser = Parser::new(&pattern);
    let re = parser.parse();
    let chars = input_line.chars().collect::<Vec<char>>();
    let (matched, _ind) = Matcher::mtch_main(&chars, re);
    return matched;
}

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // Uncomment this block to pass the first stage
    if match_pattern(&input_line, &pattern) {
        println!("Matched!");
        process::exit(0)
    } else {
        println!("No Matched!");
        process::exit(1)
    }
}
