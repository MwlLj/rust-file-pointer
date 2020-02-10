use crate::{Result, Error, Code};

use std::path;
use std::fs;

pub struct MultiFile {
    root: String
}

impl MultiFile {
    pub fn open_fixed(&self, name: &str, fixed_name: &str, fixed_size: usize) -> Result<fixed::Fixed> {
        /*
        ** 1. 检测 self.root 中是否存在 name 为名称的目录
        **  不存在 => 创建
        */
        let root_path = path::Path::new(&self.root);
        let name_path = root_path.join(name);
        if name_path.exists() {
            /*
            ** name目录存在
            */
        } else {
            /*
            ** name目录不存在
            */
            if let Err(err) = fs::create_dir_all(name_path.clone()) {
                return Err(Error{
                    code: Some(Code::CreateDirError(Some(err.to_string())))
                });
            };
        }
        let fixed = match fixed::Fixed::new(fixed_name, fixed_size, name_path) {
            Ok(f) => f,
            Err(err) => {
                return Err(err);
            }
        };
        Ok(fixed)
    }
}

impl MultiFile {
    pub fn new(root: String) -> MultiFile {
        let f = MultiFile{
            root: root
        };
        f
    }
}

pub mod delete;
pub mod fixed;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[ignore]
    fn multi_file_open_fixed_test() {
        let multi_file = MultiFile::new(String::from("run_test"));
        match multi_file.open_fixed("test.db", "user_index", 64) {
            Ok(f) => f,
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        };
    }

    #[test]
    #[ignore]
    fn fixed_new_block_test() {
        let multi_file = MultiFile::new(String::from("run_test"));
        let mut fixed = match multi_file.open_fixed("test.db", "user_index", 64) {
            Ok(f) => f,
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        };
        let block = match fixed.new_block() {
            Ok(b) => b,
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        };
    }
}
