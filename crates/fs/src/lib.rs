#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub size: usize,
    pub start_block: usize,
    pub block_count: usize,
}

#[derive(Debug)]
pub enum FsError {
    AlreadyExists,
    NotFound,
    NoSpace,
}

#[derive(Debug)]
pub struct FileSystem {
    total_blocks: usize,
    block_size: usize,
    free_blocks: Vec<bool>,
    files: BTreeMap<String, FileEntry>,
    data: BTreeMap<String, Vec<u8>>,
}

impl FileSystem {
    pub fn new(total_blocks: usize, block_size: usize) -> Self {
        Self {
            total_blocks,
            block_size,
            free_blocks: alloc::vec![true; total_blocks],
            files: BTreeMap::new(),
            data: BTreeMap::new(),
        }
    }

    pub fn create_file(&mut self, name: &str, content: &[u8]) -> Result<(), FsError> {
        if self.files.contains_key(name) {
            return Err(FsError::AlreadyExists);
        }

        let needed_blocks = self.blocks_needed(content.len());
        let start_block = self.find_contiguous_free(needed_blocks)?;
        self.mark_blocks(start_block, needed_blocks, false);

        let entry = FileEntry {
            name: String::from(name),
            size: content.len(),
            start_block,
            block_count: needed_blocks,
        };

        self.files.insert(String::from(name), entry);
        let mut buf = Vec::with_capacity(content.len());
        buf.extend_from_slice(content);
        self.data.insert(String::from(name), buf);
        Ok(())
    }

    pub fn read_file(&self, name: &str) -> Result<&[u8], FsError> {
        let content = self.data.get(name).ok_or(FsError::NotFound)?;
        Ok(content.as_slice())
    }

    pub fn delete_file(&mut self, name: &str) -> Result<(), FsError> {
        let entry = self.files.remove(name).ok_or(FsError::NotFound)?;
        self.data.remove(name);
        self.mark_blocks(entry.start_block, entry.block_count, true);
        Ok(())
    }

    pub fn list_files(&self) -> Vec<&FileEntry> {
        self.files.values().collect()
    }

    pub fn used_blocks(&self) -> usize {
        self.total_blocks - self.free_blocks.iter().filter(|b| **b).count()
    }

    fn blocks_needed(&self, size: usize) -> usize {
        if size == 0 {
            0
        } else {
            (size + self.block_size - 1) / self.block_size
        }
    }

    fn find_contiguous_free(&self, blocks: usize) -> Result<usize, FsError> {
        if blocks == 0 {
            return Ok(0);
        }

        let mut count = 0;
        let mut start = 0;

        for (i, free) in self.free_blocks.iter().enumerate() {
            if *free {
                if count == 0 {
                    start = i;
                }
                count += 1;
                if count == blocks {
                    return Ok(start);
                }
            } else {
                count = 0;
            }
        }

        Err(FsError::NoSpace)
    }

    fn mark_blocks(&mut self, start: usize, blocks: usize, free: bool) {
        for i in start..start + blocks {
            if i < self.free_blocks.len() {
                self.free_blocks[i] = free;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FileSystem;

    #[test]
    fn create_and_read_file() {
        let mut fs = FileSystem::new(10000, 4096);

        fs.create_file("a.txt", b"hello").unwrap();
        let data = fs.read_file("a.txt").unwrap();
        assert_eq!(data, b"hello");

        let entry = fs.files.get("a.txt").unwrap();
        assert_eq!(entry.size, 5);
        assert_eq!(entry.block_count, 1);
        assert_eq!(fs.used_blocks(), 1);
    }
}
