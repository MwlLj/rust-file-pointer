/*
** 使用栈, 保存删除信息
*/
use crate::multifile::{Result, Error, Code};

// use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

use std::fs;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::Path;

lazy_static!{
    static ref TAIL_LENGTH: usize = Tail::new(0).to_vec().unwrap().len();
    static ref FILE_HEADER_LENGTH: usize = FileHeader::new(0).to_vec().unwrap().len();
}

pub struct Delete {
    file: fs::File
}

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

#[derive(Default, Deserialize, Serialize)]
pub struct Pos {
    pub path: String,
    pub start_pos: usize,
    pub length: usize
}

impl Pos {
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        to_vec(self)
    }

    pub fn new(path: String, start_pos: usize, length: usize) -> Pos {
        let pos = Pos{
            path: path,
            start_pos: start_pos,
            length: length
        };
        pos
    }
}

#[derive(Default, Deserialize, Serialize)]
struct Tail {
    length: usize
}

impl Tail {
    fn to_vec(&self) -> Result<Vec<u8>> {
        to_vec(self)
    }

    fn new(length: usize) -> Tail {
        let tail = Tail{
            length: length
        };
        tail
    }
}

#[derive(Default, Deserialize, Serialize)]
struct Body {
    pos: Pos
}

impl Body {
    fn to_vec(&self) -> Result<Vec<u8>> {
        let mut pos_vec = match self.pos.to_vec() {
            Ok(v) => v,
            Err(err) => {
                return Err(err);
            }
        };
        let tail = Tail::new(pos_vec.len());
        let mut tail_vec = match tail.to_vec() {
            Ok(v) => v,
            Err(err) => {
                return Err(err);
            }
        };
        pos_vec.append(&mut tail_vec);
        Ok(pos_vec)
    }

    fn new(pos: Pos) -> Body {
        let body = Body{
            pos: pos
        };
        body
    }
}

#[derive(Default, Deserialize, Serialize)]
struct FileHeader {
    stack_top_pos: usize
}

impl FileHeader {
    fn to_vec(&self) -> Result<Vec<u8>> {
        to_vec(self)
    }

    fn new(stack_top_pos: usize) -> FileHeader {
        let file_header = FileHeader{
            stack_top_pos: stack_top_pos
        };
        file_header
    }
}

impl Delete {
    /*
    ** 将传入的位置放到栈顶
    */
    pub fn push(&mut self, pos: Pos) -> Result<()> {
        let body = Body::new(pos);
        let body_vec = match body.to_vec() {
            Ok(v) => v,
            Err(err) => {
                return Err(err);
            }
        };
        /*
        ** 获取文件头
        */
        let file_header = match Delete::get_file_header(&mut self.file) {
            Ok(h) => h,
            Err(err) => {
                return Err(err);
            }
        };
        /*
        ** 将文件指针指向文件头指定的位置
        */
        if let Err(err) = self.file.seek(SeekFrom::Start(file_header.stack_top_pos as u64)) {
            return Err(Error{
                code: Some(Code::FileSeekError(Some(err.to_string())))
            });
        };
        if let Err(err) = self.file.write(body_vec.as_slice()) {
            return Err(Error{
                code: Some(Code::FileWriteError(Some(err.to_string())))
            });
        };
        /*
        ** 更新文件头
        */
        if let Err(err) = self.update_file_header(FileHeader::new(file_header.stack_top_pos + body_vec.len())) {
            return Err(err);
        };
        Ok(())
    }

    /*
    ** 将栈顶的位置移除
    */
    pub fn pop(&mut self) -> Result<Option<Pos>> {
        /*
        ** 获取文件头
        */
        let file_header = match Delete::get_file_header(&mut self.file) {
            Ok(h) => h,
            Err(err) => {
                return Err(err);
            }
        };
        /*
        ** 判断栈是否为空
        */
        if file_header.stack_top_pos == *FILE_HEADER_LENGTH {
            return Ok(None);
        }
        /*
        ** 获取栈顶Tail
        */
        if let Err(err) = self.file.seek(SeekFrom::Start((file_header.stack_top_pos - *TAIL_LENGTH) as u64)) {
            return Err(Error{
                code: Some(Code::FileSeekError(Some(err.to_string())))
            });
        };
        let tail = match Delete::deserde_tail(&mut self.file) {
            Ok(t) => t,
            Err(err) => {
                return Err(err);
            }
        };
        /*
        ** 获取栈顶Pos
        */
        if let Err(err) = self.file.seek(SeekFrom::Start((file_header.stack_top_pos - *TAIL_LENGTH - tail.length) as u64)) {
            return Err(Error{
                code: Some(Code::FileSeekError(Some(err.to_string())))
            });
        };
        let pos = match Delete::deserde_pos(&mut self.file, tail.length) {
            Ok(p) => p,
            Err(err) => {
                return Err(err);
            }
        };
        /*
        ** 更新文件头
        */
        if let Err(err) = self.update_file_header(FileHeader::new(file_header.stack_top_pos - *TAIL_LENGTH - tail.length)) {
            return Err(err);
        };
        Ok(Some(pos))
    }
}

impl Delete {
    fn deserde_pos(file: &mut fs::File, length: usize) -> Result<Pos> {
        let mut content: Vec<u8> = Vec::new();
        if let Err(err) = file.take(length as u64).read_to_end(&mut content) {
            return Err(Error{
                code: Some(Code::FileReadError(Some(err.to_string())))
            });
        };
        let pos = match bincode::deserialize(&content) {
            Ok(p) => p,
            Err(err) => {
                return Err(Error{
                    code: Some(Code::DeserdeError(Some(err.to_string())))
                });
            }
        };
        Ok(pos)
    }

    fn deserde_tail(file: &mut fs::File) -> Result<Tail> {
        let mut content: Vec<u8> = Vec::new();
        if let Err(err) = file.take(*TAIL_LENGTH as u64).read_to_end(&mut content) {
            return Err(Error{
                code: Some(Code::FileReadError(Some(err.to_string())))
            });
        };
        let tail = match bincode::deserialize(&content) {
            Ok(t) => t,
            Err(err) => {
                return Err(Error{
                    code: Some(Code::DeserdeError(Some(err.to_string())))
                });
            }
        };
        Ok(tail)
    }

    fn get_file_header(file: &mut fs::File) -> Result<FileHeader> {
        if let Err(err) = file.seek(SeekFrom::Start(0)) {
            return Err(Error{
                code: Some(Code::FileSeekError(Some(err.to_string())))
            });
        };
        let mut content: Vec<u8> = Vec::new();
        if let Err(err) = file.take(*FILE_HEADER_LENGTH as u64).read_to_end(&mut content) {
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

    fn update_file_header(&mut self, file_hedaer: FileHeader) -> Result<()> {
        if let Err(err) = self.file.seek(SeekFrom::Start(0)) {
            return Err(Error{
                code: Some(Code::FileSeekError(Some(err.to_string())))
            });
        };
        let file_header_vec = match file_hedaer.to_vec() {
            Ok(v) => v,
            Err(err) => {
                return Err(err);
            }
        };
        if let Err(err) = self.file.write(file_header_vec.as_slice()) {
            return Err(Error{
                code: Some(Code::FileWriteError(Some(err.to_string())))
            });
        };
        Ok(())
    }

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

impl Delete {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Delete> {
        /*
        ** 打开文件
        **  1. 如果文件不存在, 写入尾指针到文件头
        **  2. 如果文件存在, 直接打开
        */
        let f = match fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path) {
            Ok(f) => f,
            Err(err) => {
                return Err(Error{
                    code: Some(Code::OpenFileError(Some(err.to_string())))
                })
            }
        };
        let mut delete = Delete{
            file: f
        };
        match delete.get_file_size() {
            Ok(size) => {
                if size == 0 {
                    /*
                    ** 文件内容为空, 需要添加文件头
                    */
                    let file_header_vec = match FileHeader::new(*FILE_HEADER_LENGTH).to_vec() {
                        Ok(v) => v,
                        Err(err) => {
                            return Err(err);
                        }
                    };
                    if let Err(err) = delete.file.write(file_header_vec.as_slice()) {
                        return Err(Error{
                            code: Some(Code::FileWriteError(Some(err.to_string())))
                        });
                    };
                }
            },
            Err(err) => {
                return Err(err);
            }
        }
        Ok(delete)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[ignore]
    fn fixed_push_test() {
        let mut delete = match Delete::new("delete_record") {
            Ok(f) => f,
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        };
        delete.push(Pos::new(String::from("."), 2, 5));
    }

    #[test]
    #[ignore]
    fn fixed_pop_test() {
        let mut delete = match Delete::new("delete_record") {
            Ok(f) => f,
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        };
        match delete.pop() {
            Ok(p) => {
                match p {
                    Some(pos) => {
                        println!("path: {}, start_pos: {}, length: {}", pos.path, pos.start_pos, pos.length);
                    },
                    None => {
                        println!("stack is empty");
                        return;
                    }
                }
            },
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        }
    }
}
