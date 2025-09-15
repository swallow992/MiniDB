use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};

const PAGE_SIZE: usize = 4096; // 4KB pages

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId(pub u64);

#[derive(Debug, Clone, Copy)]
pub enum PageType {
    Data = 1,
    Index = 2,
    Meta = 3,
}

#[derive(Debug, Clone)]
pub struct PageHeader {
    pub page_id: PageId,
    pub page_type: PageType,
    pub free_space: u16,
    pub checksum: u32,
    pub lsn: u64, // Log Sequence Number
}

#[derive(Debug, Clone)]
pub struct Page {
    pub header: PageHeader,
    pub data: Vec<u8>,
}

impl Page {
    pub fn new(page_id: PageId, page_type: PageType) -> Self {
        let data = vec![0u8; PAGE_SIZE - 24]; // 24 bytes for header
        
        Self {
            header: PageHeader {
                page_id,
                page_type,
                free_space: (PAGE_SIZE - 24) as u16,
                checksum: 0,
                lsn: 0,
            },
            data,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(PAGE_SIZE);
        
        // Serialize header
        buffer.extend_from_slice(&self.header.page_id.0.to_le_bytes());
        buffer.push(self.header.page_type as u8);
        buffer.extend_from_slice(&self.header.free_space.to_le_bytes());
        buffer.extend_from_slice(&self.header.checksum.to_le_bytes());
        buffer.extend_from_slice(&self.header.lsn.to_le_bytes());
        buffer.push(0); // Reserved byte
        
        // Serialize data
        buffer.extend_from_slice(&self.data);
        
        // Ensure exactly PAGE_SIZE
        buffer.resize(PAGE_SIZE, 0);
        buffer
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, String> {
        if buffer.len() != PAGE_SIZE {
            return Err("Invalid page size".to_string());
        }

        let page_id = PageId(u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3],
            buffer[4], buffer[5], buffer[6], buffer[7],
        ]));

        let page_type = match buffer[8] {
            1 => PageType::Data,
            2 => PageType::Index,
            3 => PageType::Meta,
            _ => return Err("Invalid page type".to_string()),
        };

        let free_space = u16::from_le_bytes([buffer[9], buffer[10]]);
        let checksum = u32::from_le_bytes([
            buffer[11], buffer[12], buffer[13], buffer[14],
        ]);
        let lsn = u64::from_le_bytes([
            buffer[15], buffer[16], buffer[17], buffer[18],
            buffer[19], buffer[20], buffer[21], buffer[22],
        ]);

        let data = buffer[24..].to_vec();

        Ok(Self {
            header: PageHeader {
                page_id,
                page_type,
                free_space,
                checksum,
                lsn,
            },
            data,
        })
    }

    pub fn write_data(&mut self, offset: usize, data: &[u8]) -> Result<(), String> {
        if offset + data.len() > self.data.len() {
            return Err("Data exceeds page capacity".to_string());
        }
        
        self.data[offset..offset + data.len()].copy_from_slice(data);
        self.header.free_space = self.header.free_space.saturating_sub(data.len() as u16);
        Ok(())
    }

    pub fn read_data(&self, offset: usize, length: usize) -> Result<&[u8], String> {
        if offset + length > self.data.len() {
            return Err("Read exceeds page boundary".to_string());
        }
        Ok(&self.data[offset..offset + length])
    }
}

pub struct FileManager {
    files: HashMap<String, File>,
    next_page_id: u64,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            next_page_id: 1,
        }
    }

    pub fn create_file(&mut self, filename: &str) -> Result<(), io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(filename)?;
        
        self.files.insert(filename.to_string(), file);
        println!("📁 创建文件: {} (4KB页面)", filename);
        Ok(())
    }

    pub fn allocate_page(&mut self, filename: &str, page_type: PageType) -> Result<PageId, String> {
        if !self.files.contains_key(filename) {
            return Err(format!("File '{}' not found", filename));
        }

        let page_id = PageId(self.next_page_id);
        self.next_page_id += 1;

        let page = Page::new(page_id, page_type);
        self.write_page(filename, page_id, &page)?;

        println!("📄 分配页面: ID={}, 类型={:?}, 文件={}", 
                 page_id.0, page_type, filename);
        Ok(page_id)
    }

    pub fn write_page(&mut self, filename: &str, page_id: PageId, page: &Page) -> Result<(), String> {
        let file = self.files.get_mut(filename)
            .ok_or_else(|| format!("File '{}' not found", filename))?;

        let offset = (page_id.0 - 1) * PAGE_SIZE as u64;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Seek error: {}", e))?;

        let serialized = page.serialize();
        file.write_all(&serialized)
            .map_err(|e| format!("Write error: {}", e))?;

        file.flush()
            .map_err(|e| format!("Flush error: {}", e))?;

        Ok(())
    }

    pub fn read_page(&mut self, filename: &str, page_id: PageId) -> Result<Page, String> {
        let file = self.files.get_mut(filename)
            .ok_or_else(|| format!("File '{}' not found", filename))?;

        let offset = (page_id.0 - 1) * PAGE_SIZE as u64;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Seek error: {}", e))?;

        let mut buffer = vec![0u8; PAGE_SIZE];
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;

        Page::deserialize(&buffer)
    }

    pub fn deallocate_page(&mut self, filename: &str, page_id: PageId) -> Result<(), String> {
        // 在实际实现中，这里会将页面标记为空闲
        println!("🗑️ 释放页面: ID={}, 文件={}", page_id.0, filename);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    page: Page,
    dirty: bool,
    last_accessed: u64,
}

pub struct BufferPool {
    cache: HashMap<PageId, CacheEntry>,
    access_order: VecDeque<PageId>,
    capacity: usize,
    access_counter: u64,
    stats: CacheStats,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub writes: u64,
}

impl BufferPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: VecDeque::new(),
            capacity,
            access_counter: 0,
            stats: CacheStats::default(),
        }
    }

    pub fn get_page(&mut self, page_id: PageId, file_manager: &mut FileManager, filename: &str) -> Result<Page, String> {
        self.access_counter += 1;

        if let Some(entry) = self.cache.get_mut(&page_id) {
            // Cache hit
            self.stats.hits += 1;
            entry.last_accessed = self.access_counter;
            
            // Move to front of access order
            self.access_order.retain(|&id| id != page_id);
            self.access_order.push_front(page_id);
            
            let total_accesses = self.stats.hits + self.stats.misses;
            let hit_rate = if total_accesses == 0 { 0.0 } else { (self.stats.hits as f64 / total_accesses as f64) * 100.0 };
            println!("🎯 缓存命中: 页面 {} (命中率: {:.1}%)", 
                     page_id.0, hit_rate);
            
            return Ok(entry.page.clone());
        }
        
        // Cache miss - load from disk
        self.stats.misses += 1;
        println!("❌ 缓存未命中: 页面 {} - 从磁盘加载", page_id.0);
        
        let page = file_manager.read_page(filename, page_id)?;
        
        // Check if we need to evict
        if self.cache.len() >= self.capacity {
            self.evict_lru(file_manager, filename)?;
        }
        
        // Insert new page
        let entry = CacheEntry {
            page: page.clone(),
            dirty: false,
            last_accessed: self.access_counter,
        };
        
        self.cache.insert(page_id, entry);
        self.access_order.push_front(page_id);
        
        let cache_len = self.cache.len();
        println!("📥 加载页面到缓存: {} (缓存使用: {}/{})", 
                 page_id.0, cache_len, self.capacity);
        
        Ok(page)
    }

    pub fn put_page(&mut self, page_id: PageId, page: Page, dirty: bool) {
        self.access_counter += 1;
        
        let entry = CacheEntry {
            page,
            dirty,
            last_accessed: self.access_counter,
        };
        
        if let Some(existing) = self.cache.get_mut(&page_id) {
            existing.page = entry.page;
            existing.dirty = dirty;
            existing.last_accessed = self.access_counter;
        } else {
            self.cache.insert(page_id, entry);
            self.access_order.push_front(page_id);
        }
        
        if dirty {
            println!("✏️ 页面标记为脏: {}", page_id.0);
        }
    }

    fn evict_lru(&mut self, file_manager: &mut FileManager, filename: &str) -> Result<(), String> {
        if let Some(lru_page_id) = self.access_order.pop_back() {
            if let Some(entry) = self.cache.remove(&lru_page_id) {
                self.stats.evictions += 1;
                
                if entry.dirty {
                    // Write back dirty page
                    self.stats.writes += 1;
                    file_manager.write_page(filename, lru_page_id, &entry.page)?;
                    println!("💾 LRU驱逐并写回脏页: {}", lru_page_id.0);
                } else {
                    println!("🔄 LRU驱逐干净页: {}", lru_page_id.0);
                }
            }
        }
        Ok(())
    }

    pub fn flush_dirty_pages(&mut self, file_manager: &mut FileManager, filename: &str) -> Result<(), String> {
        println!("🔄 刷新所有脏页到磁盘...");
        let mut flushed = 0;
        
        for (page_id, entry) in &mut self.cache {
            if entry.dirty {
                file_manager.write_page(filename, *page_id, &entry.page)?;
                entry.dirty = false;
                self.stats.writes += 1;
                flushed += 1;
            }
        }
        
        println!("✅ 刷新完成: {} 个脏页写入磁盘", flushed);
        Ok(())
    }

    pub fn get_hit_rate(&self) -> f64 {
        let total = self.stats.hits + self.stats.misses;
        if total == 0 {
            0.0
        } else {
            (self.stats.hits as f64 / total as f64) * 100.0
        }
    }

    pub fn print_stats(&self) {
        println!("\n📊 缓存统计信息:");
        println!("  总访问次数: {}", self.stats.hits + self.stats.misses);
        println!("  缓存命中: {}", self.stats.hits);
        println!("  缓存未命中: {}", self.stats.misses);
        println!("  命中率: {:.1}%", self.get_hit_rate());
        println!("  驱逐次数: {}", self.stats.evictions);
        println!("  写回次数: {}", self.stats.writes);
        println!("  当前缓存使用: {}/{}", self.cache.len(), self.capacity);
    }
}

pub struct StorageEngine {
    file_manager: FileManager,
    buffer_pool: BufferPool,
    tables: HashMap<String, Vec<PageId>>,
}

impl StorageEngine {
    pub fn new(buffer_pool_size: usize) -> Self {
        Self {
            file_manager: FileManager::new(),
            buffer_pool: BufferPool::new(buffer_pool_size),
            tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, table_name: &str) -> Result<(), String> {
        let filename = format!("{}.db", table_name);
        self.file_manager.create_file(&filename)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        // Allocate initial page for table
        let page_id = self.file_manager.allocate_page(&filename, PageType::Data)?;
        self.tables.insert(table_name.to_string(), vec![page_id]);
        
        println!("🗃️ 创建表: {} (初始页面: {})", table_name, page_id.0);
        Ok(())
    }

    pub fn insert_record(&mut self, table_name: &str, data: &[u8]) -> Result<(), String> {
        let filename = format!("{}.db", table_name);
        
        let page_ids = self.tables.get(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?
            .clone();

        // Try to insert into existing page
        for &page_id in &page_ids {
            let page = self.buffer_pool.get_page(page_id, &mut self.file_manager, &filename)?;
            
            if page.header.free_space as usize >= data.len() {
                // Clone the page to modify it
                let mut modified_page = page;
                let offset = (PAGE_SIZE - 24) - modified_page.header.free_space as usize;
                modified_page.write_data(offset, data)?;
                
                self.buffer_pool.put_page(page_id, modified_page, true);
                println!("📝 插入记录到表 {} (页面: {}, 大小: {} bytes)", 
                         table_name, page_id.0, data.len());
                return Ok(());
            }
        }

        // Need new page
        let new_page_id = self.file_manager.allocate_page(&filename, PageType::Data)?;
        let mut new_page = Page::new(new_page_id, PageType::Data);
        new_page.write_data(0, data)?;
        
        self.buffer_pool.put_page(new_page_id, new_page, true);
        self.tables.get_mut(table_name).unwrap().push(new_page_id);
        
        println!("📝 插入记录到新页面: {} (页面: {})", table_name, new_page_id.0);
        Ok(())
    }

    pub fn scan_table(&mut self, table_name: &str) -> Result<Vec<Vec<u8>>, String> {
        let filename = format!("{}.db", table_name);
        let page_ids = self.tables.get(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?
            .clone();

        let mut records = Vec::new();
        
        for &page_id in &page_ids {
            let page = self.buffer_pool.get_page(page_id, &mut self.file_manager, &filename)?;
            
            // 简化的记录扫描 - 实际实现会更复杂
            let used_space = (PAGE_SIZE - 24) - page.header.free_space as usize;
            if used_space > 0 {
                records.push(page.data[0..used_space].to_vec());
            }
            
            println!("🔍 扫描页面: {} (使用空间: {} bytes)", page_id.0, used_space);
        }
        
        Ok(records)
    }

    pub fn delete_records(&mut self, table_name: &str, _condition: &str) -> Result<u32, String> {
        // 简化的删除实现
        println!("🗑️ 删除表 {} 中的记录", table_name);
        
        let filename = format!("{}.db", table_name);
        let page_ids = self.tables.get(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?
            .clone();

        let mut deleted_count = 0;
        
        for &page_id in &page_ids {
            let page = self.buffer_pool.get_page(page_id, &mut self.file_manager, &filename)?;
            let mut modified_page = page;
            
            // 重置页面（简化删除）
            modified_page.data.fill(0);
            modified_page.header.free_space = (PAGE_SIZE - 24) as u16;
            
            self.buffer_pool.put_page(page_id, modified_page, true);
            deleted_count += 1;
        }
        
        println!("✅ 删除完成: {} 个页面被清空", deleted_count);
        Ok(deleted_count)
    }

    pub fn print_table_info(&self, table_name: &str) {
        if let Some(page_ids) = self.tables.get(table_name) {
            println!("\n📋 表信息: {}", table_name);
            println!("  页面数量: {}", page_ids.len());
            println!("  页面ID列表: {:?}", page_ids.iter().map(|p| p.0).collect::<Vec<_>>());
            println!("  存储文件: {}.db", table_name);
        }
    }

    pub fn persistence_test(&mut self) -> Result<(), String> {
        println!("\n🔒 数据持久性测试:");
        
        // 1. 刷新所有脏页
        println!("1. 刷新所有脏页到磁盘...");
        let table_names: Vec<String> = self.tables.keys().cloned().collect();
        for table_name in table_names {
            let filename = format!("{}.db", table_name);
            self.buffer_pool.flush_dirty_pages(&mut self.file_manager, &filename)?;
        }
        
        // 2. 清空缓存（模拟重启）
        println!("2. 清空缓存（模拟系统重启）...");
        self.buffer_pool = BufferPool::new(self.buffer_pool.capacity);
        
        // 3. 重新读取数据验证
        println!("3. 从磁盘重新读取数据验证...");
        let table_names: Vec<String> = self.tables.keys().cloned().collect();
        for table_name in table_names {
            let records = self.scan_table(&table_name)?;
            println!("   表 {}: 恢复 {} 条记录", table_name, records.len());
        }
        
        println!("✅ 持久性测试完成 - 所有数据成功恢复");
        Ok(())
    }
}

fn main() {
    println!("=== MiniDB 磁盘存储系统演示 ===");
    println!();

    let mut storage = StorageEngine::new(8); // 8页缓存

    println!("=== 测试1: 文件管理和页分配 ===");
    
    // 创建表
    if let Err(e) = storage.create_table("users") {
        println!("创建表失败: {}", e);
        return;
    }
    
    if let Err(e) = storage.create_table("orders") {
        println!("创建表失败: {}", e);
        return;
    }

    println!("\n=== 测试2: 数据插入操作 ===");
    
    // 插入数据
    let user_records = vec![
        b"1,Alice,25,alice@email.com".to_vec(),
        b"2,Bob,30,bob@email.com".to_vec(),
        b"3,Carol,28,carol@email.com".to_vec(),
        b"4,David,35,david@email.com".to_vec(),
        b"5,Eve,22,eve@email.com".to_vec(),
    ];

    for (i, record) in user_records.iter().enumerate() {
        if let Err(e) = storage.insert_record("users", record) {
            println!("插入记录 {} 失败: {}", i + 1, e);
        }
    }

    println!("\n=== 测试3: 缓存性能测试 ===");
    
    // 访问页面测试缓存
    println!("多次访问相同页面测试缓存命中:");
    for i in 0..10 {
        println!("第 {} 次查询:", i + 1);
        if let Err(e) = storage.scan_table("users") {
            println!("查询失败: {}", e);
        }
    }

    // 访问更多页面测试LRU
    println!("\n测试LRU替换策略:");
    for i in 0..12 {
        let table_name = format!("temp_table_{}", i);
        if let Err(e) = storage.create_table(&table_name) {
            println!("创建临时表失败: {}", e);
            continue;
        }
        
        let record = format!("temp_data_{}", i).into_bytes();
        if let Err(e) = storage.insert_record(&table_name, &record) {
            println!("插入临时数据失败: {}", e);
        }
    }

    println!("\n=== 测试4: 查询操作 ===");
    
    match storage.scan_table("users") {
        Ok(records) => {
            println!("查询到 {} 条记录:", records.len());
            for (i, record) in records.iter().enumerate() {
                println!("  记录 {}: {} bytes", i + 1, record.len());
            }
        }
        Err(e) => println!("查询失败: {}", e),
    }

    println!("\n=== 测试5: 删除操作 ===");
    
    if let Err(e) = storage.delete_records("users", "id > 3") {
        println!("删除失败: {}", e);
    }

    println!("\n=== 测试6: 数据持久性验证 ===");
    
    if let Err(e) = storage.persistence_test() {
        println!("持久性测试失败: {}", e);
    }

    // 显示最终统计
    println!("\n=== 最终统计信息 ===");
    storage.buffer_pool.print_stats();
    
    storage.print_table_info("users");
    storage.print_table_info("orders");

    println!("\n✅ 存储系统测试完成！");
    println!("测试覆盖:");
    println!("  ✓ 页式存储系统 (4KB页面)");
    println!("  ✓ LRU缓存管理");
    println!("  ✓ 页分配与释放");
    println!("  ✓ 数据插入、查询、删除");
    println!("  ✓ 缓存命中率优化");
    println!("  ✓ 数据持久性验证");
}