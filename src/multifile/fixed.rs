use crate::{Result, Error, Code};
use super::delete::stack;

use std::path::Path;

pub struct Block {
    path: String,
    start_pos: usize,
    length: usize
}

/*
** 固定大小
*/
pub struct Fixed {
    fixed_size: usize,
    delete_record: stack::Delete
}

impl Fixed {
    /*
    ** 在文件中创建一个块
    */
    pub fn new_block<Header: serde::Serialize, Body: serde::Serialize>(&mut self, header: Option<Header>, body: Option<Body>) -> Result<Block> {
        let mut header_vec: Vec<u8> = Vec::new();
        match header {
            Some(h) => {
                header_vec = match bincode::serialize(&h) {
                    Ok(c) => c,
                    Err(err) => {
                        return Err(Error{
                            code: Some(Code::SerdeError(Some(err.to_string())))
                        });
                    }
                };
            },
            None => {
            }
        }
        let mut body_vec: Vec<u8> = Vec::new();
        match body {
            Some(b) => {
                body_vec = match bincode::serialize(&b) {
                    Ok(v) => v,
                    Err(err) => {
                        return Err(Error{
                            code: Some(Code::SerdeError(Some(err.to_string())))
                        });
                    }
                };
            },
            None => {
            }
        }
        if header_vec.len() + body_vec.len() > self.fixed_size {
            return Err(Error{
                code: Some(Code::LimitError(Some(String::from("header len + body len > fixed size"))))
            });
        }
        /*
        ** 从删除的栈顶获取可用位置
        */
        let p = match self.delete_record.pop() {
            Ok(p) => p,
            Err(err) => {
                return Err(err);
            }
        };
        match p {
            Some(pos) => {
                /*
                ** 存在可用位置 => 直接使用pos作为block返回
                */
            }
            None => {
                /*
                ** 不存在可用位置 => 从文件尾部创建新的block
                */
            }
        }
        Err(Error{
            code: Some(Code::NotImplement(None))
        })
    }
}

impl Fixed {
    pub fn new<P: AsRef<Path>>(fixed_size: usize, path: P) -> Result<Self> {
        let delete_record = match stack::Delete::new(path.as_ref().join("delete_record")) {
            Ok(d) => d,
            Err(err) => {
                return Err(err);
            }
        };
        let fixed = Self {
            fixed_size: fixed_size,
            delete_record: delete_record
        };
        Ok(fixed)
    }
}
