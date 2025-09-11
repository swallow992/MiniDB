use minidb::storage::file::FileManager;
use minidb::storage::page::{Page, PageType};
use tempfile::TempDir;

fn main() {
    let temp_dir = TempDir::new().unwrap();
    let fm = FileManager::new(temp_dir.path()).unwrap();
    let file_arc = fm.create_file("test").unwrap();

    // Create a page and insert data
    let mut page = Page::new(0, PageType::Data);
    let slot_id = page.insert_record(b"test data").unwrap();
    println!("Created page with slot {}", slot_id);
    println!("Page has {} slots", page.slot_count());

    // Write to file
    {
        let mut file = file_arc.lock().unwrap();
        let page_id = file.allocate_page().unwrap();
        println!("Allocated page ID: {}", page_id);

        file.write_page(&mut page).unwrap();
        println!("Wrote page to file");
    }

    // Read from file
    {
        let mut file = file_arc.lock().unwrap();
        let loaded_page = file.read_page(0).unwrap();
        println!("Loaded page has {} slots", loaded_page.slot_count());
        println!("Loaded page slot IDs: {:?}", loaded_page.slot_ids());

        if loaded_page.slot_count() > 0 {
            let record = loaded_page.get_record(0).unwrap();
            println!("Record: {:?}", String::from_utf8_lossy(record));
        }
    }
}
