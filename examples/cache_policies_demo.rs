/// Cache Policies Demo
/// 
/// This demo showcases the multiple cache replacement policies
/// supported by MiniDB's buffer pool: LRU, Clock, and LFU.

use minidb::storage::buffer::{BufferPool, CachePolicyType};
use minidb::storage::file::FileManager;
use minidb::storage::page::PageType;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MiniDB 多缓存策略演示 ===\n");

    let temp_dir = TempDir::new()?;
    let fm = FileManager::new(temp_dir.path())?;
    
    // Test different cache policies
    test_cache_policy("LRU", CachePolicyType::LRU, &fm)?;
    test_cache_policy("Clock", CachePolicyType::Clock, &fm)?;
    test_cache_policy("LFU", CachePolicyType::LFU, &fm)?;

    println!("=== 多缓存策略演示完成 ===");
    
    Ok(())
}

fn test_cache_policy(
    policy_name: &str,
    policy_type: CachePolicyType,
    fm: &FileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 测试 {} 缓存策略:", policy_name);
    
    // Create buffer pool with specified policy
    let pool = BufferPool::with_policy(5, policy_type);
    
    // Verify policy is set correctly
    let actual_policy = pool.cache_policy_name()?;
    println!("   ✅ 策略设置: {}", actual_policy);
    
    // Create test file
    let file_name = format!("test_{}", policy_name.to_lowercase());
    let file = fm.create_file(&file_name)?;
    
    // Create some pages to test cache behavior
    let mut frame_ids = Vec::new();
    for i in 0..3 {
        let (frame_id, page_arc) = pool.new_page(file.clone(), PageType::Data)?;
        
        // Add some data to the page
        {
            let mut page = page_arc.lock().unwrap();
            let data = format!("Test data for page {} with {} policy", i, policy_name);
            page.insert_record(data.as_bytes()).ok(); // Ignore errors for demo
        }
        
        frame_ids.push(frame_id);
        pool.unpin_page(frame_id, true)?;
        
        println!("   📄 创建页面 {} (Frame ID: {})", i, frame_id);
    }
    
    // Get buffer statistics
    let stats = pool.get_stats()?;
    println!("   📈 缓存统计:");
    println!("      - 池大小: {}", stats.pool_size);
    println!("      - 使用帧数: {}", stats.used_frames);
    println!("      - 固定页数: {}", stats.pinned_pages);
    println!("      - 脏页数: {}", stats.dirty_pages);
    
    // Test cache access patterns
    if policy_name == "LFU" {
        println!("   🔄 测试 LFU 访问频率模式:");
        // Access first page multiple times to increase frequency
        for i in 0..3 {
            let _ = pool.fetch_page(file.clone(), 0)?;
            pool.unpin_page(frame_ids[0], false)?;
            println!("      - 第 {} 次访问页面 0", i + 1);
        }
    }
    
    // Force eviction by creating more pages than buffer size
    if stats.used_frames >= 3 {
        println!("   🔄 创建更多页面以触发页面替换...");
        for i in 3..6 {
            let (frame_id, _) = pool.new_page(file.clone(), PageType::Data)?;
            pool.unpin_page(frame_id, true)?;
            println!("      - 创建页面 {} (可能触发替换)", i);
        }
    }
    
    let final_stats = pool.get_stats()?;
    println!("   📊 最终统计: 使用帧数 {}/{}", final_stats.used_frames, final_stats.pool_size);
    println!();
    
    Ok(())
}