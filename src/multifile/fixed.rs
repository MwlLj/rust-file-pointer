use crate::{Result, Error, Code};
use super::delete::stack;

use serde_derive::{Serialize, Deserialize};

use std::path::Path;
use std::fs;
use std::io::SeekFrom;
use std::io::prelude::*;

fn to_vec<T: serde::Serialize>(t: &T) -> Result<Vec<u8>> {
    let s = match bincode::serialize(t) {
        Ok(c) => c,
        Err(err) => {
            return Err(Error{
                code: Some(Code::SerdeError(Some(err.to_string())))
            });
        }
    };
    Ok(s)
}

fn new_u8_vec_with_size(size: usize) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity(size);
    for i in 0..size {
        v.push(0);
    }
    v
}

/*
** 块
*/
pub struct Block {
    path: String,
    start_pos: usize,
    length: usize,
    file: fs::File
}

/*
** 为 usize 新增方法
*/
trait ToVec {
    fn to_vec(&self) -> Result<Vec<u8>>;
}

impl ToVec for usize {
    fn to_vec(&self) -> Result<Vec<u8>> {
        to_vec(self)
    }
}

#[derive(Default, Serialize, Deserialize)]
struct BlockHeader {
    /*
    ** 业务的header长度
    */
    header_size: usize
}

impl BlockHeader {
    fn to_vec(&self) -> Result<Vec<u8>> {
        to_vec(self)
    }

    fn new(header_size: usize) -> Self {
        Self {
            header_size: header_size
        }
    }
}

lazy_static!{
    static ref BLOCK_HEADER_LENGTH: usize = BlockHeader::new(0).to_vec().unwrap().len();
}

impl Block {
    /*
    ** 更新header
    */
    pub fn update_header<Header: serde::Serialize>(&self, header: Header) {
        let block_header = match self.get_block_header(&mut self.file) {
            Ok(h) => h,
            Err(err) => {
                return Err(err);
            }
        };
        let header_vec = match to_vec(&header) {
            Ok(v) => v,
            Err(err) => {
                return Err(err);
            }
        };
    }
}

impl Block {
    fn get_block_header(file: &mut fs::File) -> Result<BlockHeader> {
        if let Err(err) = file.seek(SeekFrom::Start(0)) {
            return Err(Error{
                code: Some(Code::FileSeekError(Some(err.to_string())))
            });
        };
        let mut content: Vec<u8> = Vec::new();
        if let Err(err) = file.take(*BLOCK_HEADER_LENGTH as u64).read_to_end(&mut content) {
            return Err(Error{
                code: Some(Code::FileReadError(Some(err.to_string())))
            });
        };
        let header = match bincode::deserialize(&content) {
            Ok(h) => h,
            Err(err) => {
                return Err(Error{
                    code: Some(Code::DeserdeError(Some(err.to_string())))
                });
            }
        };
        Ok(header)
    }
}

impl Block {
    fn from_delete_stack_pos(pos: stack::Pos, file: fs::File) -> Self {
        Self {
            path: pos.path,
            start_pos: pos.start_pos,
            length: pos.length,
            file: file
        }
    }

    fn new(path: String, start_pos: usize, length: usize, file: fs::File) -> Self {
        Self {
            path: path,
            start_pos: start_pos,
            length: length,
            file: file
        }
    }
}

/*
** 固定大小
*/
pub struct Fixed {
    fixed_size: usize,
    delete_record: stack::Delete,
    file: fs::File,
    name: String,
    file_path: String
}

impl Fixed {
    /*
    ** 在文件中创建一个块
    */
    pub fn new_block(&mut self) -> Result<Block> {
        let file_clone = match self.file.try_clone() {
            Some(f) => f,
            None => {
                return Err(Error{
                    code: Some(Code::FileTryCloneError(Some(String::from("file try clone error"))))
                });
            }
        };
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
                return Ok(Block::from_delete_stack_pos(pos, file_clone))
            }
            None => {
                /*
                ** 不存在可用位置 => 从文件尾部创建新的block
                */
                let file_size = match self.get_file_size() {
                    Ok(l) => l,
                    Err(err) => {
                        return Err(err);
                    }
                };
                /*
                ** 写入初始化数据
                */
                if let Err(err) = self.file.seek(SeekFrom::End(0)) {
                    return Err(Error{
                        code: Some(Code::FileSeekError(Some(err.to_string())))
                    });
                };
                /*
                ** block size + fixed size
                */
                if let Err(err) = self.file.write(new_u8_vec_with_size(*BLOCK_HEADER_LENGTH + self.fixed_size).as_slice()) {
                    return Err(Error{
                        code: Some(Code::FileWriteError(Some(err.to_string())))
                    });
                };
                return Ok(Block::new(self.file_path.clone(), file_size, self.fixed_size, file_clone));
            }
        }
        Err(Error{
            code: Some(Code::NewError(Some(String::from("delete and new block all error"))))
        })
    }
}

impl Fixed {
    pub fn new<P: AsRef<Path>>(name: &str, fixed_size: usize, path: P) -> Result<Self> {
        /*
        ** 打开文件
        */
        let file_path = path.as_ref().join(&name);
        let file_path_name = match file_path.to_str() {
            Some(p) => p.to_string(),
            None => {
                return Err(Error{
                    code: Some(Code::PathToStrError(Some(String::from("path to_str is none"))))
                });
            }
        };
        let f = match fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(file_path) {
            Ok(f) => f,
            Err(err) => {
                return Err(Error{
                    code: Some(Code::OpenFileError(Some(err.to_string())))
                })
            }
        };
        /*
        ** 使用 name 拼接 delete record name
        */
        let mut delete_record_name = String::new();
        delete_record_name.push_str(name);
        delete_record_name.push_str("_delete.rd");
        let delete_record = match stack::Delete::new(path.as_ref().join(&delete_record_name)) {
            Ok(d) => d,
            Err(err) => {
                return Err(err);
            }
        };
        let fixed = Self {
            fixed_size: fixed_size,
            delete_record: delete_record,
            file: f,
            name: name.to_string(),
            file_path: file_path_name
        };
        Ok(fixed)
    }
}

impl Fixed {
    fn get_file_size(&self) -> Result<usize> {
        let metadata = match self.file.metadata() {
            Ok(l) => l,
            Err(err) => {
                return Err(Error{
                    code: Some(Code::FileMetadataError(Some(err.to_string())))
                });
            }
        };
        Ok(metadata.len() as usize)
    }
}
