use std::cell::{RefCell};
use std::fs::File;

pub struct FSOParsingError {
	
}

pub struct FSOParserState {
	file: RefCell<File>
}

pub trait FSOParser<'a> {
	
}

impl FSOParser<'_> for FSOParserState {
	
}

pub trait FSOTable<'parser, Parser: FSOParser<'parser>> {
	fn parse(state: &'parser Parser) -> Result<Self, FSOParsingError> where Self: Sized;
	fn dump(&self);
}

impl<'a, Parser: FSOParser<'a>> FSOTable<'a, Parser> for String {
	fn parse(_state: &Parser) -> Result<Self, FSOParsingError> {
		Ok("".to_string())
	}

	fn dump(&self) { }
}