use std::cell::{RefCell};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;
use regex::Regex;

#[derive(Debug)]
pub struct FSOParsingError {
	
}

impl Display for FSOParsingError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {	
		write!(f, "")
	}
}
impl Error for FSOParsingError{ }

pub trait FSOParser<'a> {
	fn get(&self) -> &str;

	fn line(&self) -> usize;

	fn consume(&self, count: usize);
	
	//Returns (comments, version_string) in this whitespace. Will break immediately after a version string
	fn consume_whitespace(&self, stop_on_newline: bool) -> (Option<String>, Option<String>) {
		thread_local! { static VERSION_SYNTAX: Regex = Regex::new(r"\A;;FSO\x20\d+(?:\x2E\d+)+;;").unwrap(); }
		
		let mut comments: Option<String> = None;
		let mut version: Option<String> = None;
		let mut linebreak_since_comment = true;

		loop {
			self.consume_whitespace_strict(&[]);
			let current = self.get();
			
			let mut add_to_comment = "".to_string();
			
			let mut current_char : Peekable<Chars> = current.chars().peekable();
			match current_char.next() {
				Some('\n') if stop_on_newline => { break; }
				Some('\n') => {
					linebreak_since_comment = true;
					self.consume(1);
					continue;
				}
				Some(';') => { 
					//Comment or Version
					if VERSION_SYNTAX.with(|regex| regex.is_match(current)) {
						//Version
						self.consume(2);
						version = Some(format!(";;{};;", self.read_until_target(";;", true)));
						break;
					}
					else {
						add_to_comment = format!("{}", self.read_until_target("\n", true));
						linebreak_since_comment = true;
					}
				}
				Some('/') if current_char.peek().is_some_and(|c| *c == '/')=> {
					//Comment
					add_to_comment = format!("{}", self.read_until_target("\n", true));
					linebreak_since_comment = true;
				}
				(Some(start @ '!') | Some(start @ '/')) if current_char.peek().is_some_and(|c| *c == '*') => {
					//Mutliline comment
					current_char.next();
					self.consume(2);
					let mut target = "*".to_string();
					target.push(start);
					add_to_comment = format!("{}*{}*{}", start, self.read_until_target(target.as_str(), true), start);
				}
				_ => { break; }
			}
			if comments.is_none() {
				comments = Some(String::new());
			}
			if let Some(comment) = &mut comments {
				*comment = format!("{}{}{}", comment, if linebreak_since_comment { "" } else { "\n" }, add_to_comment);
				linebreak_since_comment = false;
			}
		}
		
		return (comments, version)
	}
	
	fn consume_whitespace_strict<const N: usize>(&self, also_consume: &[char;N]) {
		let current = self.get();
		let whitespaces = current.chars().take_while(|c| (*c != '\n' && c.is_whitespace()) || also_consume.contains(c)).fold(0, |sum, c| sum + c.len_utf8());
		self.consume(whitespaces);
	}
	
	fn read_until_target(&self, target: &str, consume_target: bool) -> &str {
		let current = self.get();
		let content_size = current.find(target).unwrap_or_else(|| current.len());
		self.consume(content_size + if consume_target { target.len() } else { 0 });
		&current[..content_size]
	}
	
	//Notably, this also does not include post-line comments!
	fn read_until_last_whitespace_of_line(&self) -> &str {
		let current = self.get();
		let mut last_non_whitespace = 0usize;
		
		for c in current.chars() {
			if c == '\n' || c == ';' {
				break;
			}
			if !c.is_whitespace() {
				last_non_whitespace += c.len_utf8();
			}
		}
		
		self.consume(last_non_whitespace);
		return &current[..last_non_whitespace]
	}
}

#[derive(Default)]
struct FSOParserState {
	pos: usize,
	line: usize
}

pub struct FSOTableFileParser {

	original: String,
	state: RefCell<FSOParserState>
}
impl FSOTableFileParser {
	pub fn new(path: &Path) -> Result<Self, impl Error>{
		let mut s = String::new();
		
		let mut file = match File::open(&path) {
			Ok(file) => { file }
			Err(_) => { return Err(FSOParsingError { }) }
		};

		match file.read_to_string(&mut s) {
			Ok(_) => {  }
			Err(_) => { return Err(FSOParsingError { }) }
		};

		let parser = FSOTableFileParser {
			original: s,
			state: RefCell::new(FSOParserState::default())
		};
		
		Ok(parser)
	}
}

impl FSOParser<'_> for FSOTableFileParser {
	fn get(&self) -> &str {
		let start = self.state.borrow().pos;
		&self.original[start..]
	}
	
	fn line(&self) -> usize {
		self.state.borrow().line
	}

	fn consume(&self, count: usize) {
		let newlines = self.get()[..count].chars().filter(|c| *c == '\n').count();
		
		let mut state = self.state.borrow_mut();
		state.pos += count;
		state.line += newlines;
	}
}

pub trait FSOTable<'parser, Parser: FSOParser<'parser>> {
	fn parse(state: &'parser Parser) -> Result<Self, FSOParsingError> where Self: Sized;
	fn dump(&self);
}

impl<'a, Parser: FSOParser<'a>> FSOTable<'a, Parser> for String {
	fn parse(state: &Parser) -> Result<Self, FSOParsingError> {
		let mut to_consume: usize = 0;
		let to_parse = state.get();
		
		state.consume(to_consume);
		Ok("".to_string())
	}

	fn dump(&self) { }
}