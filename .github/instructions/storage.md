# 存储系统模块指令

## 模块职责
实现数据库的底层存储管理，包括页式存储模型、缓存管理器和文件系统接口，为上层查询引擎提供高效的数据访问。

## 开发指导

### 页式存储 (page.rs)
- **页面大小**: 标准4KB或8KB页面，支持配置
- **页面类型**: 数据页、索引页、元数据页、空闲页
- **页面格式**: 统一的页头格式，支持校验和
- **并发控制**: 页级锁定机制，支持读写锁

```rust
// 推荐的页面结构
pub const PAGE_SIZE: usize = 4096;

#[repr(C)]
pub struct PageHeader {
    pub page_id: u32,
    pub page_type: PageType,
    pub checksum: u32,
    pub lsn: u64,          // 日志序列号
    pub free_space: u16,   // 可用空间大小
    pub slot_count: u16,   // 记录槽数量
}

#[derive(Debug, Clone, Copy)]
pub enum PageType {
    Data = 1,
    Index = 2,
    Metadata = 3,
    Free = 4,
}

pub struct Page {
    data: [u8; PAGE_SIZE],
    is_dirty: bool,
    pin_count: AtomicU32,
}

impl Page {
    pub fn new(page_id: u32, page_type: PageType) -> Self;
    pub fn read_slot(&self, slot_id: u16) -> Option<&[u8]>;
    pub fn write_slot(&mut self, slot_id: u16, data: &[u8]) -> Result<(), PageError>;
    pub fn allocate_slot(&mut self, size: usize) -> Option<u16>;
    pub fn free_slot(&mut self, slot_id: u16);
    pub fn verify_checksum(&self) -> bool;
    pub fn update_checksum(&mut self);
}
```

### 缓存管理器 (buffer.rs)
- **替换策略**: 实现LRU和Clock算法
- **脏页管理**: 延迟写入和批量刷新
- **预读机制**: 支持顺序和随机预读
- **统计信息**: 命中率、I/O统计

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};

pub struct BufferPool {
    frames: Vec<Arc<RwLock<Frame>>>,
    page_table: HashMap<u32, usize>,  // page_id -> frame_index
    replacer: Box<dyn Replacer>,
    free_frames: Vec<usize>,
    stats: BufferStats,
}

pub struct Frame {
    page: Option<Page>,
    page_id: u32,
    pin_count: u32,
    is_dirty: bool,
    access_time: Instant,
}

pub trait Replacer: Send + Sync {
    fn victim(&mut self) -> Option<usize>;
    fn pin(&mut self, frame_id: usize);
    fn unpin(&mut self, frame_id: usize);
}

pub struct LRUReplacer {
    lru_list: LinkedList<usize>,
    node_map: HashMap<usize, *mut LRUNode>,
}

impl BufferPool {
    pub fn new(pool_size: usize, replacer: Box<dyn Replacer>) -> Self;
    pub fn get_page(&self, page_id: u32) -> Result<Arc<RwLock<Frame>>, BufferError>;
    pub fn new_page(&self) -> Result<(u32, Arc<RwLock<Frame>>), BufferError>;
    pub fn unpin_page(&self, page_id: u32, is_dirty: bool) -> bool;
    pub fn flush_page(&self, page_id: u32) -> Result<(), BufferError>;
    pub fn flush_all_pages(&self) -> Result<(), BufferError>;
    pub fn delete_page(&self, page_id: u32) -> Result<(), BufferError>;
}
```

### 文件系统接口 (file.rs)
- **文件管理**: 数据文件和日志文件的创建、打开、关闭
- **I/O接口**: 同步和异步I/O操作
- **文件扩展**: 自动文件扩展和空间回收
- **错误处理**: 完善的I/O错误处理和恢复

```rust
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Read, Write};
use std::path::{Path, PathBuf};

pub struct DatabaseFile {
    file: File,
    file_path: PathBuf,
    file_id: u32,
    page_count: u32,
    is_readonly: bool,
}

pub struct FileManager {
    data_dir: PathBuf,
    files: HashMap<u32, DatabaseFile>,
    next_file_id: u32,
}

impl DatabaseFile {
    pub fn open<P: AsRef<Path>>(path: P, file_id: u32) -> Result<Self, FileError>;
    pub fn create<P: AsRef<Path>>(path: P, file_id: u32) -> Result<Self, FileError>;
    
    pub fn read_page(&mut self, page_id: u32, buffer: &mut [u8]) -> Result<(), FileError>;
    pub fn write_page(&mut self, page_id: u32, buffer: &[u8]) -> Result<(), FileError>;
    
    pub fn allocate_page(&mut self) -> Result<u32, FileError>;
    pub fn deallocate_page(&mut self, page_id: u32) -> Result<(), FileError>;
    
    pub fn sync(&mut self) -> Result<(), FileError>;
    pub fn get_file_size(&self) -> u64;
}

impl FileManager {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self;
    pub fn open_file(&mut self, filename: &str) -> Result<u32, FileError>;
    pub fn create_file(&mut self, filename: &str) -> Result<u32, FileError>;
    pub fn get_file(&mut self, file_id: u32) -> Option<&mut DatabaseFile>;
    pub fn close_file(&mut self, file_id: u32) -> Result<(), FileError>;
}
```

## 错误处理

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("页面错误: {message}")]
    PageError { message: String },
    
    #[error("缓冲池错误: {message}")]
    BufferError { message: String },
    
    #[error("文件错误: {message}")]
    FileError { message: String },
    
    #[error("I/O错误: {source}")]
    IoError { #[from] source: std::io::Error },
    
    #[error("页面校验失败: page_id={page_id}")]
    ChecksumError { page_id: u32 },
}
```

## 性能优化指导

### 内存对齐和布局
```rust
// 使用内存对齐优化缓存性能
#[repr(align(64))]  // 缓存行对齐
pub struct Frame {
    // 热点数据放在前面
    page_id: u32,
    pin_count: AtomicU32,
    is_dirty: AtomicBool,
    // 冷数据放在后面
    page_data: [u8; PAGE_SIZE],
}
```

### 并发优化
```rust
// 使用读写锁减少锁竞争
pub struct BufferPool {
    // 使用分段锁减少锁粒度
    page_table_locks: Vec<RwLock<HashMap<u32, usize>>>,
    // 使用原子操作避免锁
    stats: AtomicBufferStats,
}
```

### I/O优化
```rust
impl FileManager {
    // 批量I/O操作
    pub fn batch_read_pages(&mut self, requests: &[(u32, u32)]) -> Result<Vec<Vec<u8>>, FileError>;
    pub fn batch_write_pages(&mut self, requests: &[(u32, &[u8])]) -> Result<(), FileError>;
    
    // 异步I/O支持
    pub async fn async_read_page(&mut self, page_id: u32) -> Result<Vec<u8>, FileError>;
    pub async fn async_write_page(&mut self, page_id: u32, data: &[u8]) -> Result<(), FileError>;
}
```

## 测试指导

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_page_operations() {
        let mut page = Page::new(1, PageType::Data);
        let data = b"test data";
        
        let slot_id = page.allocate_slot(data.len()).unwrap();
        page.write_slot(slot_id, data).unwrap();
        
        let read_data = page.read_slot(slot_id).unwrap();
        assert_eq!(read_data, data);
        
        page.free_slot(slot_id);
        assert!(page.read_slot(slot_id).is_none());
    }

    #[test]
    fn test_buffer_pool_concurrency() {
        let pool = BufferPool::new(10, Box::new(LRUReplacer::new(10)));
        
        // 并发访问测试
        let handles: Vec<_> = (0..4).map(|_| {
            let pool = Arc::clone(&pool);
            thread::spawn(move || {
                for i in 0..100 {
                    let frame = pool.get_page(i % 20).unwrap();
                    // 模拟页面操作
                    thread::sleep(Duration::from_millis(1));
                    pool.unpin_page(i % 20, i % 5 == 0);
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
```

### 性能测试
```rust
#[cfg(test)]
mod bench {
    use super::*;
    use criterion::{black_box, Criterion, criterion_group, criterion_main};

    fn bench_page_operations(c: &mut Criterion) {
        c.bench_function("page_slot_allocation", |b| {
            b.iter(|| {
                let mut page = Page::new(1, PageType::Data);
                for i in 0..100 {
                    let slot = page.allocate_slot(black_box(64));
                    black_box(slot);
                }
            })
        });
    }

    criterion_group!(benches, bench_page_operations);
    criterion_main!(benches);
}
```

## 调试和监控

### 统计信息收集
```rust
#[derive(Debug, Default)]
pub struct BufferStats {
    pub total_requests: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub pages_read: AtomicU64,
    pub pages_written: AtomicU64,
    pub evictions: AtomicU64,
}

impl BufferStats {
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let total = self.total_requests.load(Ordering::Relaxed) as f64;
        if total > 0.0 { hits / total } else { 0.0 }
    }
}
```

### 调试工具
```rust
impl BufferPool {
    pub fn dump_stats(&self) -> String {
        format!(
            "BufferPool Stats:\n\
             Hit Ratio: {:.2}%\n\
             Total Requests: {}\n\
             Cache Hits: {}\n\
             Pages Written: {}",
            self.stats.hit_ratio() * 100.0,
            self.stats.total_requests.load(Ordering::Relaxed),
            self.stats.cache_hits.load(Ordering::Relaxed),
            self.stats.pages_written.load(Ordering::Relaxed),
        )
    }
    
    pub fn dump_buffer_state(&self) -> String {
        // 输出缓冲池当前状态，用于调试
    }
}
```

这些指导原则确保存储系统模块具有高性能、高可靠性和良好的可维护性。
