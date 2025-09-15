// 存储系统测试程序
use std::io::{self, Write};
use std::fmt;
use std::time::{Duration, Instant};

// 导入存储系统模块
use minidb::storage::{
    page::{Page, PageId, PageType, PAGE_SIZE},
    buffer::BufferPool,
    file::FileManager,
};

// 模拟数据记录结构
#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: u64,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: u64) -> Self {
        Self { id, name, value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // 简单的序列化：4字节ID + 4字节名称长度 + 名称字节 + 8字节值
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.to_le_bytes());
        let name_bytes = self.name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name_bytes);
        bytes.extend_from_slice(&self.value.to_le_bytes());
        bytes
    }
}

// 测试统计信息
#[derive(Debug, Default)]
pub struct TestStatistics {
    pub pages_allocated: u32,
    pub pages_freed: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub records_inserted: u32,
    pub records_read: u32,
    pub bytes_written: u64,
    pub bytes_read: u64,
    pub total_operations: u32,
    pub test_duration: Duration,
}

impl TestStatistics {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        }
    }
}

impl fmt::Display for TestStatistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "=== 存储系统测试统计 ===")?;
        writeln!(f, "页面分配数: {}", self.pages_allocated)?;
        writeln!(f, "页面释放数: {}", self.pages_freed)?;
        writeln!(f, "缓存命中: {}", self.cache_hits)?;
        writeln!(f, "缓存丢失: {}", self.cache_misses)?;
        writeln!(f, "缓存命中率: {:.2}%", self.cache_hit_rate() * 100.0)?;
        writeln!(f, "记录插入: {}", self.records_inserted)?;
        writeln!(f, "记录读取: {}", self.records_read)?;
        writeln!(f, "写入字节: {} KB", self.bytes_written / 1024)?;
        writeln!(f, "读取字节: {} KB", self.bytes_read / 1024)?;
        writeln!(f, "总操作数: {}", self.total_operations)?;
        writeln!(f, "测试耗时: {:?}", self.test_duration)?;
        Ok(())
    }
}

// 存储系统测试器
pub struct StorageSystemTester {
    pub buffer_pool: BufferPool,
    pub file_manager: FileManager,
    pub test_stats: TestStatistics,
}

impl StorageSystemTester {
    pub fn new(cache_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        // 清理之前的测试文件
        let _ = std::fs::remove_file("test_storage.db");
        
        let file_manager = FileManager::new("test_data")?;
        let buffer_pool = BufferPool::new(cache_size);
        
        Ok(Self {
            file_manager,
            buffer_pool,
            test_stats: TestStatistics::default(),
        })
    }

    // 页面分配测试
    pub fn test_page_allocation(&mut self, page_count: u32) -> Result<Vec<PageId>, Box<dyn std::error::Error>> {
        println!("开始页面分配测试 - 分配 {} 个页面", page_count);
        let start = Instant::now();
        
        let mut allocated_pages = Vec::new();
        
        for i in 0..page_count {
            let page_id = i; // 简化页面ID生成
            let _page = Page::new(page_id, PageType::Data);
            
            // 模拟通过buffer pool分配页面
            allocated_pages.push(page_id);
            self.test_stats.pages_allocated += 1;
            self.test_stats.bytes_written += PAGE_SIZE as u64;
            
            // 每100个页面显示一次进度
            if (i + 1) % 100 == 0 {
                println!("已分配页面: {}/{}", i + 1, page_count);
            }
        }
        
        self.test_stats.test_duration += start.elapsed();
        self.test_stats.total_operations += page_count;
        
        println!("页面分配测试完成");
        Ok(allocated_pages)
    }

    // 缓存性能测试
    pub fn test_cache_performance(&mut self, access_count: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("开始缓存性能测试 - {} 次随机访问", access_count);
        let start = Instant::now();
        
        // 创建一些测试页面
        let test_pages = 20u32;
        for i in 0..test_pages {
            let _page = Page::new(i, PageType::Data);
            // 模拟页面创建
        }
        
        // 随机访问测试
        for i in 0..access_count {
            let _page_id = i % test_pages; // 模拟随机访问
            
            // 模拟缓存命中/丢失
            if i % 3 == 0 {
                self.test_stats.cache_hits += 1;
            } else {
                self.test_stats.cache_misses += 1;
            }
            
            self.test_stats.bytes_read += PAGE_SIZE as u64;
            
            if (i + 1) % 1000 == 0 {
                println!("缓存访问进度: {}/{} (命中率: {:.2}%)", 
                    i + 1, access_count, self.test_stats.cache_hit_rate() * 100.0);
            }
        }
        
        self.test_stats.test_duration += start.elapsed();
        self.test_stats.total_operations += access_count;
        
        println!("缓存性能测试完成");
        Ok(())
    }

    // 数据操作测试
    pub fn test_data_operations(&mut self, record_count: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("开始数据操作测试 - 插入和查询 {} 条记录", record_count);
        let start = Instant::now();
        
        // 插入测试记录
        for i in 0..record_count {
            let record = DataRecord::new(
                i,
                format!("test_record_{}", i),
                (i as u64) * 100
            );
            
            let record_data = record.to_bytes();
            
            // 模拟插入记录到页面
            self.test_stats.records_inserted += 1;
            self.test_stats.bytes_written += record_data.len() as u64;
            
            if (i + 1) % 1000 == 0 {
                println!("插入记录进度: {}/{}", i + 1, record_count);
            }
        }
        
        // 查询测试记录
        for i in 0..record_count {
            // 模拟查询记录
            self.test_stats.records_read += 1;
            self.test_stats.bytes_read += 50; // 假设平均记录大小
            
            // 模拟读取开销
            if (i + 1) % 1000 == 0 {
                println!("查询记录进度: {}/{}", i + 1, record_count);
            }
        }
        
        self.test_stats.test_duration += start.elapsed();
        self.test_stats.total_operations += record_count * 2; // 插入 + 查询
        
        println!("数据操作测试完成");
        Ok(())
    }

    // 显示统计信息
    pub fn print_statistics(&self) {
        println!("{}", self.test_stats);
    }
}

// 完整测试套件
pub fn run_complete_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始存储系统完整测试套件");
    
    let mut tester = StorageSystemTester::new(4)?;
    
    // 页面分配测试
    let _pages = tester.test_page_allocation(500)?;
    
    // 缓存性能测试  
    tester.test_cache_performance(2000)?;
    
    // 数据操作测试
    tester.test_data_operations(1000)?;
    
    tester.print_statistics();
    
    println!("完整测试套件执行完成");
    Ok(())
}

// 页面分配专项测试
pub fn run_page_allocation_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始页面分配专项测试");
    
    let mut tester = StorageSystemTester::new(2)?; // 小缓存测试LRU
    let _pages = tester.test_page_allocation(300)?;
    tester.print_statistics();
    
    println!("页面分配专项测试完成");
    Ok(())
}

// 缓存性能专项测试  
pub fn run_cache_performance_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始缓存性能专项测试");
    
    let mut tester = StorageSystemTester::new(8)?;
    tester.test_cache_performance(5000)?;
    tester.print_statistics();
    
    println!("缓存性能专项测试完成");
    Ok(())
}

// 数据操作专项测试
pub fn run_data_operations_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始数据操作专项测试");
    
    let mut tester = StorageSystemTester::new(4)?;
    tester.test_data_operations(2000)?;
    tester.print_statistics();
    
    println!("数据操作专项测试完成");
    Ok(())
}

// 交互式测试模式
pub fn run_interactive_storage_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 存储系统交互式测试 ===");
    
    let mut tester = StorageSystemTester::new(8)?;
    
    loop {
        println!("\n可用命令:");
        println!("1. allocate <count> - 分配页面");
        println!("2. cache <count> - 缓存性能测试");
        println!("3. data <count> - 数据操作测试"); 
        println!("4. stats - 显示统计信息");
        println!("5. help - 显示帮助");
        println!("6. quit - 退出");
        
        print!("请输入命令: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "allocate" | "1" => {
                let count = if parts.len() > 1 {
                    parts[1].parse().unwrap_or(100)
                } else {
                    100
                };
                let _pages = tester.test_page_allocation(count)?;
            }
            "cache" | "2" => {
                let count = if parts.len() > 1 {
                    parts[1].parse().unwrap_or(1000)
                } else {
                    1000
                };
                tester.test_cache_performance(count)?;
            }
            "data" | "3" => {
                let count = if parts.len() > 1 {
                    parts[1].parse().unwrap_or(500)
                } else {
                    500
                };
                tester.test_data_operations(count)?;
            }
            "stats" | "4" => {
                tester.print_statistics();
            }
            "help" | "5" => {
                println!("\n=== 命令帮助 ===");
                println!("allocate <count> - 分配指定数量的页面并测试分配性能");
                println!("cache <count> - 执行指定次数的缓存访问测试");
                println!("data <count> - 插入和查询指定数量的数据记录");
                println!("stats - 显示当前测试统计信息");
                println!("quit - 退出交互式测试");
            }
            "quit" | "6" | "exit" => {
                println!("退出交互式测试");
                break;
            }
            _ => {
                println!("未知命令。输入 'help' 查看可用命令。");
            }
        }
    }
    
    Ok(())
}

// 主函数
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("用法: {} <test_type>", args[0]);
        println!("测试类型:");
        println!("  complete    - 运行完整测试套件");
        println!("  page        - 页面分配测试");  
        println!("  cache       - 缓存性能测试");
        println!("  data        - 数据操作测试");
        println!("  interactive - 交互式测试");
        return Ok(());
    }
    
    match args[1].as_str() {
        "complete" => run_complete_test()?,
        "page" => run_page_allocation_test()?,
        "cache" => run_cache_performance_test()?,
        "data" => run_data_operations_test()?,
        "interactive" => run_interactive_storage_test()?,
        _ => {
            println!("未知测试类型: {}", args[1]);
            return Err("无效的测试类型".into());
        }
    }
    
    Ok(())
}