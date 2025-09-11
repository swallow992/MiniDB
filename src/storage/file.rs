//! File system management
//!
//! This module provides file system operations for database storage.
//! It manages database files and provides atomic I/O operations.

use crate::storage::page::{Page, PageId, PageType, PAGE_SIZE};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;

/// File identifier type
pub type FileId = u32;

/// File manager for database storage
pub struct FileManager {
    /// Base directory for database files
    base_dir: PathBuf,
    /// Open files cache
    files: Arc<RwLock<HashMap<String, Arc<Mutex<DatabaseFile>>>>>,
    /// Next file ID for auto-generation
    next_file_id: Arc<Mutex<FileId>>,
}

/// Database file handle
#[derive(Debug)]
pub struct DatabaseFile {
    /// File path
    path: PathBuf,
    /// File handle
    file: File,
    /// File size in pages
    page_count: u32,
    /// File ID
    file_id: FileId,
}

/// File system errors
#[derive(Error, Debug)]
pub enum FileError {
    #[error("File not found: {path}")]
    NotFound { path: String },
    
    #[error("File already exists: {path}")]
    AlreadyExists { path: String },
    
    #[error("Invalid file format: {reason}")]
    InvalidFormat { reason: String },
    
    #[error("Invalid page ID: {page_id} (max: {max_pages})")]
    InvalidPageId { page_id: PageId, max_pages: u32 },
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Lock error: file is locked")]
    LockError,
}

impl FileManager {
    /// Create a new file manager
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self, FileError> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // Create base directory if it doesn't exist
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir)?;
        }
        
        Ok(Self {
            base_dir,
            files: Arc::new(RwLock::new(HashMap::new())),
            next_file_id: Arc::new(Mutex::new(1)),
        })
    }
    
    /// Create a new database file
    pub fn create_file(&self, name: &str) -> Result<Arc<Mutex<DatabaseFile>>, FileError> {
        let file_path = self.base_dir.join(format!("{}.db", name));
        
        if file_path.exists() {
            return Err(FileError::AlreadyExists {
                path: file_path.to_string_lossy().to_string(),
            });
        }
        
        // Generate file ID
        let file_id = {
            let mut next_id = self.next_file_id.lock()
                .map_err(|_| FileError::LockError)?;
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        // Create file
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&file_path)?;
            
        let db_file = DatabaseFile {
            path: file_path.clone(),
            file,
            page_count: 0,
            file_id,
        };
        
        let db_file_arc = Arc::new(Mutex::new(db_file));
        
        // Cache the file
        {
            let mut files = self.files.write()
                .map_err(|_| FileError::LockError)?;
            files.insert(name.to_string(), db_file_arc.clone());
        }
        
        Ok(db_file_arc)
    }
    
    /// Open an existing database file
    pub fn open_file(&self, name: &str) -> Result<Arc<Mutex<DatabaseFile>>, FileError> {
        // Check if file is already open
        {
            let files = self.files.read()
                .map_err(|_| FileError::LockError)?;
            if let Some(file) = files.get(name) {
                return Ok(file.clone());
            }
        }
        
        let file_path = self.base_dir.join(format!("{}.db", name));
        
        if !file_path.exists() {
            return Err(FileError::NotFound {
                path: file_path.to_string_lossy().to_string(),
            });
        }
        
        // Generate file ID
        let file_id = {
            let mut next_id = self.next_file_id.lock()
                .map_err(|_| FileError::LockError)?;
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        // Open file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&file_path)?;
            
        // Calculate page count
        let file_size = file.metadata()?.len();
        let page_count = (file_size / PAGE_SIZE as u64) as u32;
        
        let db_file = DatabaseFile {
            path: file_path.clone(),
            file,
            page_count,
            file_id,
        };
        
        let db_file_arc = Arc::new(Mutex::new(db_file));
        
        // Cache the file
        {
            let mut files = self.files.write()
                .map_err(|_| FileError::LockError)?;
            files.insert(name.to_string(), db_file_arc.clone());
        }
        
        Ok(db_file_arc)
    }
    
    /// Close a file and remove from cache
    pub fn close_file(&self, name: &str) -> Result<(), FileError> {
        let mut files = self.files.write()
            .map_err(|_| FileError::LockError)?;
        files.remove(name);
        Ok(())
    }
    
    /// Delete a database file
    pub fn delete_file(&self, name: &str) -> Result<(), FileError> {
        // Close file first
        self.close_file(name)?;
        
        let file_path = self.base_dir.join(format!("{}.db", name));
        
        if file_path.exists() {
            std::fs::remove_file(&file_path)?;
        }
        
        Ok(())
    }
    
    /// List all database files
    pub fn list_files(&self) -> Result<Vec<String>, FileError> {
        let mut files = Vec::new();
        
        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "db" {
                    if let Some(name) = path.file_stem() {
                        files.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(files)
    }
}

impl DatabaseFile {
    /// Get file ID
    pub fn file_id(&self) -> FileId {
        self.file_id
    }
    
    /// Get file path
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Get number of pages in file
    pub fn page_count(&self) -> u32 {
        self.page_count
    }
    
    /// Allocate a new page and return its ID
    pub fn allocate_page(&mut self) -> Result<PageId, FileError> {
        let page_id = self.page_count;
        
        // Extend file size
        self.file.seek(SeekFrom::Start((page_id as u64 + 1) * PAGE_SIZE as u64))?;
        self.file.write(&[0])?; // Write one byte to extend file
        self.file.flush()?;
        
        self.page_count += 1;
        
        Ok(page_id)
    }
    
    /// Read a page from file
    pub fn read_page(&mut self, page_id: PageId) -> Result<Page, FileError> {
        if page_id >= self.page_count {
            return Err(FileError::InvalidPageId {
                page_id,
                max_pages: self.page_count,
            });
        }
        
        // Seek to page position
        self.file.seek(SeekFrom::Start(page_id as u64 * PAGE_SIZE as u64))?;
        
        // Read page data
        let mut buffer = vec![0u8; PAGE_SIZE];
        self.file.read_exact(&mut buffer)?;
        
        // Parse page from bytes
        Page::from_bytes(page_id, buffer)
            .map_err(|e| FileError::InvalidFormat { 
                reason: format!("Failed to parse page {}: {}", page_id, e) 
            })
    }
    
    /// Write a page to file
    pub fn write_page(&mut self, page: &mut Page) -> Result<(), FileError> {
        let page_id = page.page_id();
        
        if page_id >= self.page_count {
            return Err(FileError::InvalidPageId {
                page_id,
                max_pages: self.page_count,
            });
        }
        
        // Seek to page position
        self.file.seek(SeekFrom::Start(page_id as u64 * PAGE_SIZE as u64))?;
        
        // Serialize page to bytes
        let page_bytes = page.to_bytes()
            .map_err(|e| FileError::InvalidFormat { 
                reason: format!("Failed to serialize page {}: {}", page_id, e) 
            })?;
        
        // Write page data
        self.file.write_all(page_bytes)?;
        self.file.flush()?;
        
        // Mark page as clean
        page.mark_clean();
        
        Ok(())
    }
    
    /// Sync all changes to disk
    pub fn sync(&mut self) -> Result<(), FileError> {
        self.file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_file_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let _fm = FileManager::new(temp_dir.path()).unwrap();
        
        // Base directory should exist
        assert!(temp_dir.path().exists());
    }
    
    #[test]
    fn test_create_and_open_file() {
        let temp_dir = TempDir::new().unwrap();
        let fm = FileManager::new(temp_dir.path()).unwrap();
        
        // Create a new file
        let file = fm.create_file("test").unwrap();
        
        // Should be able to open the same file
        let file2 = fm.open_file("test").unwrap();
        
        // Should return the same file instance (from cache)
        assert!(Arc::ptr_eq(&file, &file2));
    }
    
    #[test]
    fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let fm = FileManager::new(temp_dir.path()).unwrap();
        
        // Create file
        let file_arc = fm.create_file("test").unwrap();
        
        // Allocate and write a page
        {
            let mut file = file_arc.lock().unwrap();
            
            let page_id = file.allocate_page().unwrap();
            assert_eq!(page_id, 0);
            assert_eq!(file.page_count(), 1);
            
            let mut page = Page::new(page_id, PageType::Data);
            page.insert_record(b"test data").unwrap();
            
            file.write_page(&mut page).unwrap();
        }
        
        // Read the page back
        {
            let mut file = file_arc.lock().unwrap();
            let page = file.read_page(0).unwrap();
            
            let record = page.get_record(0).unwrap();
            assert_eq!(record, b"test data");
        }
    }
    
    #[test]
    fn test_file_listing() {
        let temp_dir = TempDir::new().unwrap();
        let fm = FileManager::new(temp_dir.path()).unwrap();
        
        // Create some files
        fm.create_file("file1").unwrap();
        fm.create_file("file2").unwrap();
        
        let files = fm.list_files().unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1".to_string()));
        assert!(files.contains(&"file2".to_string()));
    }
    
    #[test]
    fn test_file_deletion() {
        let temp_dir = TempDir::new().unwrap();
        let fm = FileManager::new(temp_dir.path()).unwrap();
        
        // Create and delete file
        fm.create_file("test").unwrap();
        assert!(fm.list_files().unwrap().contains(&"test".to_string()));
        
        fm.delete_file("test").unwrap();
        assert!(!fm.list_files().unwrap().contains(&"test".to_string()));
    }
}
