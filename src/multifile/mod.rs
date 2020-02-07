use serde::{Deserialize, Serialize};

use super::{Result, Error, Code};

pub struct CMultiFile {
}

pub struct CConnect {
}

impl CMultiFile {
    pub fn open(&self, name: &str) -> Result<CConnect> {
        Err(Error{
            code: Some(Code::NotImplement(None))
        })
    }
}

impl CConnect {
    /*
    ** 在文件中创建一个块
    */
    pub fn new_block<Header: Serialize>(&self, header: &Header) -> Result<()> {
        let mut fh = match bincode::serialize(header) {
            Ok(c) => c,
            Err(err) => {
                return Err(Error{
                    code: Some(Code::SerdeError(Some(err.to_string())))
                });
            }
        };
        Err(Error{
            code: Some(Code::NotImplement(None))
        })
    }
}

impl CMultiFile {
    pub fn new() -> CMultiFile {
        let f = CMultiFile{
        };
        f
    }
}

pub mod delete;
