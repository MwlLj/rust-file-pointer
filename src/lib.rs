#[derive(Debug)]
pub enum Code {
    NotImplement(Option<String>),
    SerdeError(Option<String>)
}

#[derive(Debug)]
pub struct Error {
    pub code: Option<Code>
}

type Result<T> = std::result::Result<T, Error>;

pub mod multifile;
pub mod singlefile;
