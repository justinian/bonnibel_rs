use std::io;
use std::path::PathBuf;
use std::result;

use failure::Fail;

pub type Result<T> =
	result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq, Fail)]
pub enum Error {
    #[fail(display = "opening file {:?}", _0)]
    OpenFile(PathBuf),

    #[fail(display = "parsing config file")]
    ParseConfig,

    #[fail(display = "finding path")]
    PathError,

    #[fail(display = "parsing templates")]
    TemplateParsing,

    #[fail(display = "rendering templates")]
    TemplateRendering,

    #[fail(display = "no module named {}", _0)]
    NoSuchModule(String),
}
