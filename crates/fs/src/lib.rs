use std::collections::HashMap;

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
    files: HashMap<String, FileEntry>,
    data: HashMap<String, Vec<u8>>,
}

impl FileSystem {
    pub fn new(total_blocks: usize, block_size: usize) -> Self {
        Self {
            total_blocks,
            block_size,
            free_blocks: vec![true; total_blocks],
            files: HashMap::new(),
            data: HashMap::new(),
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
            name: name.to_string(),
            size: content.len(),
            start_block,
            block_count: needed_blocks,
        };

        self.files.insert(name.to_string(), entry);
        self.data.insert(name.to_string(), content.to_vec());
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
