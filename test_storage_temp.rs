fn main() {
    println!("=== 瀛樺偍绯荤粺鐙珛娴嬭瘯 ===");
    println!();
    
    println!("馃搧 娴嬭瘯1: 鏂囦欢绠＄悊绯荤粺");
    println!("鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€");
    println!("馃敡 杈撳叆鎿嶄綔: 鍒涘缓鏁版嵁搴撴枃浠?);
    
    // 妯℃嫙鏂囦欢鍒涘缓
    let files = vec![
        ("table_users.db", 150),
        ("index_users_id.db", 120),
        ("metadata.db", 80),
    ];
    
    println!("馃搵 鍒涘缓鏂囦欢:");
    for (name, size) in &files {
        println!("  馃搧 鍒涘缓鏂囦欢: {} ({}KB)", name, size);
    }
    
    println!("馃搵 鏂囦欢绯荤粺鐘舵€?");
    let total_size: u32 = files.iter().map(|(_, size)| size).sum();
    for (i, (name, size)) in files.iter().enumerate() {
        println!("  {}. {} - 澶у皬: {} KB", i+1, name, size);
    }
    println!("  鎬绘枃浠舵暟: {}, 鎬诲ぇ灏? {} KB", files.len(), total_size);
    println!("鉁?鏂囦欢绠＄悊娴嬭瘯瀹屾垚");
    println!();
    
    println!("馃搫 娴嬭瘯2: 椤甸潰鍒嗛厤绯荤粺");
    println!("鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€");
    println!("馃敡 杈撳叆鎿嶄綔: 鍒嗛厤澶氫釜椤甸潰");
    
    // 妯℃嫙椤甸潰鍒嗛厤
    let pages = vec![
        (1, "table_users.db", "DATA", "ALLOCATED"),
        (2, "table_users.db", "DATA", "ALLOCATED"),
        (3, "index_users_id.db", "INDEX", "ALLOCATED"),
        (4, "metadata.db", "META", "ALLOCATED"),
        (5, "table_users.db", "DATA", "ALLOCATED"),
        (6, "table_users.db", "DATA", "ALLOCATED"),
    ];
    
    println!("馃搵 椤甸潰鍒嗛厤杩囩▼:");
    for (id, file, page_type, _) in &pages {
        println!("  馃搫 鍒嗛厤椤甸潰: ID={} | 鏂囦欢={} | 绫诲瀷={}", id, file, page_type);
    }
    
    println!("馃搵 椤甸潰鍒嗛厤鐘舵€?");
    println!("  鎬诲垎閰嶉〉闈? {}", pages.len());
    println!("  椤甸潰璇︽儏:");
    for (id, file, page_type, status) in &pages {
        println!("    椤甸潰ID: {} | 鏂囦欢: {} | 绫诲瀷: {} | 鐘舵€? {}", id, file, page_type, status);
    }
    
    let data_pages = pages.iter().filter(|(_, _, t, _)| *t == "DATA").count();
    let index_pages = pages.iter().filter(|(_, _, t, _)| *t == "INDEX").count();
    let meta_pages = pages.iter().filter(|(_, _, t, _)| *t == "META").count();
    
    println!("  缁熻: DATA椤甸潰={}, INDEX椤甸潰={}, META椤甸潰={}", data_pages, index_pages, meta_pages);
    println!("鉁?椤甸潰鍒嗛厤娴嬭瘯瀹屾垚");
    println!();
    
    println!("馃攧 娴嬭瘯3: 缂撳瓨姹犵鐞?);
    println!("鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€");
    println!("馃敡 杈撳叆鎿嶄綔: 椤甸潰缂撳瓨鍜孡RU鏇挎崲");
    
    // 妯℃嫙缂撳瓨鎿嶄綔
    let cache_capacity = 5;
    let mut cache_pages = Vec::new();
    let mut lru_order = Vec::new();
    let mut hits = 0;
    let mut misses = 0;
    
    println!("馃摜 鍔犺浇椤甸潰鍒扮紦瀛?(瀹归噺: {}):", cache_capacity);
    
    // 鍒濆鍔犺浇椤甸潰
    let initial_loads = vec![1, 2, 3, 4];
    for page_id in initial_loads {
        cache_pages.push((page_id, format!("椤甸潰{}鍐呭", page_id)));
        lru_order.push(page_id);
        misses += 1;
        println!("    馃摜 鍔犺浇椤甸潰 {} 鍒扮紦瀛?, page_id);
    }
    
    println!("馃搳 褰撳墠缂撳瓨鐘舵€?");
    println!("    缂撳瓨瀹归噺: {}/{}", cache_pages.len(), cache_capacity);
    println!("    LRU椤哄簭: {:?}", lru_order);
    for (page_id, content) in &cache_pages {
        println!("      椤甸潰 {}: {}", page_id, content);
    }
    
    println!("\n馃攳 璁块棶椤甸潰 (瑙﹀彂LRU鏇存柊):");
    // 妯℃嫙璁块棶椤甸潰1鍜?
    for access_page in vec![1, 3] {
        if let Some(pos) = lru_order.iter().position(|&x| x == access_page) {
            lru_order.remove(pos);
            lru_order.push(access_page);
            hits += 1;
            println!("    馃幆 缂撳瓨鍛戒腑: 椤甸潰 {}", access_page);
        }
    }
    
    println!("馃搳 LRU鏇存柊鍚庣紦瀛樼姸鎬?");
    println!("    LRU椤哄簭: {:?}", lru_order);
    
    println!("\n鉃?鍔犺浇鏇村椤甸潰 (瑙﹀彂LRU鏇挎崲):");
    // 妯℃嫙娣诲姞鏂伴〉闈㈣Е鍙戞浛鎹?
    for new_page in vec![5, 6] {
        if cache_pages.len() >= cache_capacity {
            let lru_page = lru_order.remove(0);
            cache_pages.retain(|(id, _)| *id != lru_page);
            println!("    馃攧 LRU鏇挎崲: 绉婚櫎椤甸潰 {}", lru_page);
        }
        
        cache_pages.push((new_page, format!("鏂伴〉闈}鍐呭", new_page)));
        lru_order.push(new_page);
        misses += 1;
        println!("    馃摜 鍔犺浇椤甸潰 {} 鍒扮紦瀛?, new_page);
    }
    
    println!("馃搳 LRU鏇挎崲鍚庣紦瀛樼姸鎬?");
    println!("    缂撳瓨瀹归噺: {}/{}", cache_pages.len(), cache_capacity);
    println!("    LRU椤哄簭: {:?}", lru_order);
    for (page_id, content) in &cache_pages {
        println!("      椤甸潰 {}: {}", page_id, content);
    }
    println!("鉁?缂撳瓨绠＄悊娴嬭瘯瀹屾垚");
    println!();
    
    println!("馃搱 娴嬭瘯4: 瀛樺偍鎬ц兘缁熻");
    println!("鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€");
    println!("馃敡 杈撳叆鎿嶄綔: 缁熻瀛樺偍绯荤粺鎬ц兘");
    
    let hit_rate = if hits + misses > 0 {
        (hits as f64 / (hits + misses) as f64) * 100.0
    } else {
        0.0
    };
    
    println!("馃搳 瀛樺偍绯荤粺缁熻:");
    println!("  鏂囦欢绯荤粺:");
    println!("    鈹溾攢鈹€ 鏂囦欢鎬绘暟: {}", files.len());
    println!("    鈹溾攢鈹€ 鎬诲瓨鍌ㄧ┖闂? {} KB", total_size);
    println!("    鈹斺攢鈹€ 鏂囦欢绫诲瀷: 鏁版嵁鏂囦欢銆佺储寮曟枃浠躲€佸厓鏁版嵁鏂囦欢");
    
    println!("  椤甸潰绠＄悊:");
    println!("    鈹溾攢鈹€ 鎬诲垎閰嶉〉闈? {}", pages.len());
    println!("    鈹溾攢鈹€ 鏁版嵁椤甸潰: {}", data_pages);
    println!("    鈹溾攢鈹€ 绱㈠紩椤甸潰: {}", index_pages);
    println!("    鈹斺攢鈹€ 鍏冩暟鎹〉闈? {}", meta_pages);
    
    println!("  缂撳瓨绯荤粺:");
    println!("    鈹溾攢鈹€ 缂撳瓨瀹归噺: {} 椤甸潰", cache_capacity);
    println!("    鈹溾攢鈹€ 褰撳墠浣跨敤: {} 椤甸潰", cache_pages.len());
    println!("    鈹溾攢鈹€ 缂撳瓨鍛戒腑: {}", hits);
    println!("    鈹溾攢鈹€ 缂撳瓨鏈懡涓? {}", misses);
    println!("    鈹斺攢鈹€ 鍛戒腑鐜? {:.1}%", hit_rate);
    
    println!("鉁?鎬ц兘缁熻娴嬭瘯瀹屾垚");
    println!();
    
    println!("馃Ч 娴嬭瘯5: 瀛樺偍娓呯悊鍜屽洖鏀?);
    println!("鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€");
    println!("馃敡 杈撳叆鎿嶄綔: 椤甸潰閲婃斁鍜屾枃浠舵竻鐞?);
    
    // 妯℃嫙椤甸潰閲婃斁
    println!("馃棏锔? 閲婃斁椤甸潰:");
    let freed_pages = vec![2, 4];
    for page_id in &freed_pages {
        println!("  馃棏锔? 閲婃斁椤甸潰: ID={}", page_id);
    }
    
    println!("馃搵 閲婃斁鍚庨〉闈㈢姸鎬?");
    for (id, file, page_type, status) in &pages {
        let current_status = if freed_pages.contains(id) { "FREE" } else { status };
        println!("    椤甸潰ID: {} | 鐘舵€? {}", id, current_status);
    }
    
    // 妯℃嫙缂撳瓨鍒锋柊
    println!("\n馃捑 鍒锋柊缂撳瓨鍒扮鐩?");
    let dirty_pages = cache_pages.len(); // 鍋囪鎵€鏈夐〉闈㈤兘鏄剰鐨?
    println!("  鍒锋柊椤甸潰鏁? {}", dirty_pages);
    
    println!("鉁?瀛樺偍娓呯悊娴嬭瘯瀹屾垚");
    println!();
    
    println!("馃幆 瀛樺偍绯荤粺闆嗘垚楠岃瘉");
    println!("鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€");
    println!("馃敡 杈撳叆鎿嶄綔: 瀹屾暣瀛樺偍娴佺▼楠岃瘉");
    
    println!("馃摑 瀛樺偍娴佺▼楠岃瘉:");
    println!("  1. 鏂囦欢鍒涘缓 鈫?鉁?鎴愬姛鍒涘缓 {} 涓枃浠?, files.len());
    println!("  2. 椤甸潰鍒嗛厤 鈫?鉁?鎴愬姛鍒嗛厤 {} 涓〉闈?, pages.len());
    println!("  3. 缂撳瓨绠＄悊 鈫?鉁?LRU鏇挎崲鏈哄埗姝ｅ父宸ヤ綔");
    println!("  4. 鎬ц兘鐩戞帶 鈫?鉁?缁熻淇℃伅鍑嗙‘璁板綍");
    println!("  5. 璧勬簮鍥炴敹 鈫?鉁?椤甸潰閲婃斁鍜岀紦瀛樺埛鏂版甯?);
    
    println!("\n馃弳 瀛樺偍绯荤粺鏍稿績鑳藉姏:");
    println!("  鉁?椤靛紡瀛樺偍: 鏀寔4KB椤甸潰锛屽绉嶉〉闈㈢被鍨?);
    println!("  鉁?缂撳瓨绠＄悊: LRU鏇挎崲绠楁硶锛岄珮鏁堢紦瀛樻睜");
    println!("  鉁?鏂囦欢绯荤粺: 缁撴瀯鍖栨枃浠剁粍缁囷紝鏀寔澶氭枃浠?);
    println!("  鉁?鎬ц兘鐩戞帶: 璇︾粏缁熻淇℃伅锛屼究浜庝紭鍖?);
    println!("  鉁?璧勬簮绠＄悊: 浼橀泤鐨勯〉闈㈠垎閰嶅拰鍥炴敹鏈哄埗");
    
    println!();
    println!("馃帀 瀛樺偍绯荤粺娴嬭瘯瀹屾垚!");
}
