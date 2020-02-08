#[macro_use]
extern crate lazy_static;

#[derive(Debug)]
pub enum Code {
    NotImplement(Option<String>),
    SerdeError(Option<String>),
    DeserdeError(Option<String>),
    OpenFileError(Option<String>),
    FileMetadataError(Option<String>),
    FileSeekError(Option<String>),
    FileWriteError(Option<String>),
    FileReadError(Option<String>),
    CreateDirError(Option<String>),
    LimitError(Option<String>)
}

#[derive(Debug)]
pub struct Error {
    pub code: Option<Code>
}

type Result<T> = std::result::Result<T, Error>;

pub mod multifile;
pub mod singlefile;
