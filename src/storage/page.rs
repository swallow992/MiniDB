//! 页面管理
//!
//! 此模块实现基于页面的存储系统，使用固定大小的页面。
//! 每个页面可以包含数据记录或索引条目。

use std::collections::HashMap;
use std::mem;
use thiserror::Error;

/// 页面标识符类型
pub type PageId = u32;

/// 页面内的槽标识符
pub type SlotId = u16;

/// 页面大小（字节）(8KB)
pub const PAGE_SIZE: usize = 8192;

/// 页头大小（字节）
pub const PAGE_HEADER_SIZE: usize = 64;

/// 每页最大数据大小
pub const MAX_PAGE_DATA_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE;

/// 页面类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageType {
    /// 包含记录的数据页
    Data = 1,
    /// 包含 B+ 树节点的索引页
    Index = 2,
    /// 包含元数据的元页
    Meta = 3,
}

/// 包含元数据的页头
#[derive(Debug, Clone)]
pub struct PageHeader {
    pub page_id: PageId,
    pub page_type: PageType,
    pub slot_count: SlotId,
    pub free_space_offset: u16,
    pub free_space_size: u16,
    pub next_page: Option<PageId>,
    pub prev_page: Option<PageId>,
    pub checksum: u32,
}

/// 槽目录条目
#[derive(Debug, Clone, Copy)]
pub struct SlotEntry {
    /// 从页面开头的偏移量
    pub offset: u16,
    /// 记录的长度
    pub length: u16,
}

/// 包含记录的固定大小页面
#[derive(Debug, Clone)]
pub struct Page {
    header: PageHeader,
    /// 原始页面数据
    data: Vec<u8>,
    /// 槽目录：将 slot_id 映射到记录位置
    slots: HashMap<SlotId, SlotEntry>,
    /// 脏标志：表示页面是否已被修改
    dirty: bool,
}

/// 页面相关错误
#[derive(Error, Debug)]
pub enum PageError {
    #[error("未找到页面 {0}")]
    NotFound(PageId),

    #[error("页面中未找到槽 {0}")]
    SlotNotFound(SlotId),

    #[error("页面空间不足（需要: {required}, 可用: {available}）")]
    InsufficientSpace { required: usize, available: usize },

    #[error("Record too large (size: {size}, max: {max})")]
    RecordTooLarge { size: usize, max: usize },

    #[error("Invalid page format: {0}")]
    InvalidFormat(String),

    #[error("Page checksum mismatch")]
    ChecksumMismatch,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl PageHeader {
    /// 创建新的页面头
    pub fn new(page_id: PageId, page_type: PageType) -> Self {
        Self {
            page_id,
            page_type,
            slot_count: 0,
            free_space_offset: PAGE_HEADER_SIZE as u16,
            free_space_size: MAX_PAGE_DATA_SIZE as u16,
            next_page: None,
            prev_page: None,
            checksum: 0,
        }
    }
}

impl Page {
    /// 创建新的空页面
    pub fn new(page_id: PageId, page_type: PageType) -> Self {
        let data = vec![0u8; PAGE_SIZE];

        let header = PageHeader::new(page_id, page_type);

        Self {
            header,
            data,
            slots: HashMap::new(),
            dirty: false,
        }
    }

    /// Create page from raw bytes
    pub fn from_bytes(page_id: PageId, bytes: Vec<u8>) -> Result<Self, PageError> {
        if bytes.len() != PAGE_SIZE {
            return Err(PageError::InvalidFormat(format!(
                "Invalid page size: {}, expected: {}",
                bytes.len(),
                PAGE_SIZE
            )));
        }

        // Parse header from bytes
        let header = Self::parse_header(&bytes)?;

        // Verify page ID matches
        if header.page_id != page_id {
            return Err(PageError::InvalidFormat(format!(
                "Page ID mismatch: expected {}, found {}",
                page_id, header.page_id
            )));
        }

        // Parse slot directory
        let slots = Self::parse_slots(&bytes, &header)?;

        Ok(Self {
            header,
            data: bytes,
            slots,
            dirty: false,
        })
    }

    /// Get page ID
    pub fn page_id(&self) -> PageId {
        self.header.page_id
    }

    /// Get page type
    pub fn page_type(&self) -> PageType {
        self.header.page_type
    }

    /// Check if page is dirty (modified)
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark page as clean
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get available free space
    pub fn free_space(&self) -> usize {
        self.header.free_space_size as usize
    }

    /// Get number of slots
    pub fn slot_count(&self) -> usize {
        self.header.slot_count as usize
    }

    /// Insert a record into the page
    pub fn insert_record(&mut self, data: &[u8]) -> Result<SlotId, PageError> {
        let record_size = data.len();

        // Check if record fits in page
        if record_size > MAX_PAGE_DATA_SIZE {
            return Err(PageError::RecordTooLarge {
                size: record_size,
                max: MAX_PAGE_DATA_SIZE,
            });
        }

        // Check available space (record + slot entry)
        let required_space = record_size + mem::size_of::<SlotEntry>();
        let available_space = self.header.free_space_size as usize;
        if available_space < required_space {
            return Err(PageError::InsufficientSpace {
                required: required_space,
                available: available_space,
            });
        }

        // Find next slot ID
        let slot_id = self.header.slot_count;

        // Calculate record offset (grow from end of page backwards)
        let record_offset = self.header.free_space_offset as usize
            + self.header.free_space_size as usize
            - record_size;

        // Copy record data
        self.data[record_offset..record_offset + record_size].copy_from_slice(data);

        // Create slot entry
        let slot_entry = SlotEntry {
            offset: record_offset as u16,
            length: record_size as u16,
        };

        // Update slot directory
        self.slots.insert(slot_id, slot_entry);

        // Update header
        self.header.slot_count += 1;
        self.header.free_space_size -= required_space as u16;

        // Mark as dirty
        self.dirty = true;

        // Update header and slots in data
        self.serialize_header()?;
        self.serialize_slots()?;

        Ok(slot_id)
    }

    /// Get a record by slot ID
    pub fn get_record(&self, slot_id: SlotId) -> Result<&[u8], PageError> {
        let slot_entry = self
            .slots
            .get(&slot_id)
            .ok_or(PageError::SlotNotFound(slot_id))?;

        let start = slot_entry.offset as usize;
        let end = start + slot_entry.length as usize;

        Ok(&self.data[start..end])
    }

    /// Update a record by slot ID
    pub fn update_record(&mut self, slot_id: SlotId, new_data: &[u8]) -> Result<(), PageError> {
        let slot_entry = self
            .slots
            .get(&slot_id)
            .ok_or(PageError::SlotNotFound(slot_id))?
            .clone();

        // Check if new data fits in existing slot
        if new_data.len() > slot_entry.length as usize {
            return Err(PageError::InsufficientSpace {
                required: new_data.len(),
                available: slot_entry.length as usize,
            });
        }

        let start = slot_entry.offset as usize;
        let end = start + new_data.len();

        // Update record data
        self.data[start..end].copy_from_slice(new_data);

        // If new data is smaller, clear remaining space
        if new_data.len() < slot_entry.length as usize {
            let clear_start = start + new_data.len();
            let clear_end = start + slot_entry.length as usize;
            self.data[clear_start..clear_end].fill(0);
        }

        self.dirty = true;
        Ok(())
    }

    /// Delete a record by slot ID
    pub fn delete_record(&mut self, slot_id: SlotId) -> Result<(), PageError> {
        let slot_entry = self
            .slots
            .remove(&slot_id)
            .ok_or(PageError::SlotNotFound(slot_id))?;

        // Clear record data
        let start = slot_entry.offset as usize;
        let end = start + slot_entry.length as usize;
        self.data[start..end].fill(0);

        // Update free space (simplified - doesn't compact)
        self.header.free_space_size += slot_entry.length;

        self.dirty = true;
        Ok(())
    }

    /// Get all slot IDs in the page
    pub fn slot_ids(&self) -> Vec<SlotId> {
        self.slots.keys().cloned().collect()
    }

    /// Serialize page to bytes for storage
    pub fn to_bytes(&mut self) -> Result<&[u8], PageError> {
        self.serialize_header()?;
        self.serialize_slots()?;
        Ok(&self.data)
    }

    /// Parse header from raw bytes
    fn parse_header(bytes: &[u8]) -> Result<PageHeader, PageError> {
        if bytes.len() < PAGE_HEADER_SIZE {
            return Err(PageError::InvalidFormat(
                "Page too small to contain header".to_string(),
            ));
        }

        // Simple parsing (in real implementation, use proper serialization)
        let page_id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let page_type_val = bytes[4];
        let page_type = match page_type_val {
            1 => PageType::Data,
            2 => PageType::Index,
            3 => PageType::Meta,
            _ => {
                return Err(PageError::InvalidFormat(format!(
                    "Invalid page type: {}",
                    page_type_val
                )))
            }
        };

        let slot_count = u16::from_le_bytes([bytes[8], bytes[9]]);
        let free_space_offset = u16::from_le_bytes([bytes[10], bytes[11]]);
        let free_space_size = u16::from_le_bytes([bytes[12], bytes[13]]);

        Ok(PageHeader {
            page_id,
            page_type,
            slot_count,
            free_space_offset,
            free_space_size,
            next_page: None, // Simplified
            prev_page: None, // Simplified
            checksum: 0,     // Simplified
        })
    }

    /// Parse slot directory from bytes
    fn parse_slots(
        bytes: &[u8],
        header: &PageHeader,
    ) -> Result<HashMap<SlotId, SlotEntry>, PageError> {
        let mut slots = HashMap::new();

        // Simplified slot parsing
        let slot_dir_start = PAGE_HEADER_SIZE;
        for i in 0..header.slot_count {
            let slot_offset = slot_dir_start + (i as usize * mem::size_of::<SlotEntry>());
            if slot_offset + mem::size_of::<SlotEntry>() > bytes.len() {
                break;
            }

            let offset = u16::from_le_bytes([bytes[slot_offset], bytes[slot_offset + 1]]);
            let length = u16::from_le_bytes([bytes[slot_offset + 2], bytes[slot_offset + 3]]);

            if offset != 0 && length != 0 {
                slots.insert(i, SlotEntry { offset, length });
            }
        }

        Ok(slots)
    }

    /// Serialize header to page data
    fn serialize_header(&mut self) -> Result<(), PageError> {
        // Serialize header to first bytes of page
        self.data[0..4].copy_from_slice(&self.header.page_id.to_le_bytes());
        self.data[4] = self.header.page_type as u8;
        // Fill padding bytes
        self.data[5] = 0;
        self.data[6] = 0;
        self.data[7] = 0;
        self.data[8..10].copy_from_slice(&self.header.slot_count.to_le_bytes());
        self.data[10..12].copy_from_slice(&self.header.free_space_offset.to_le_bytes());
        self.data[12..14].copy_from_slice(&self.header.free_space_size.to_le_bytes());

        Ok(())
    }

    /// Serialize slot directory to page data
    fn serialize_slots(&mut self) -> Result<(), PageError> {
        let slot_dir_start = PAGE_HEADER_SIZE;

        // Clear slot directory area first
        let slot_dir_size = self.header.slot_count as usize * mem::size_of::<SlotEntry>();
        if slot_dir_start + slot_dir_size <= self.data.len() {
            self.data[slot_dir_start..slot_dir_start + slot_dir_size].fill(0);
        }

        // Write active slots
        for (slot_id, slot_entry) in &self.slots {
            if *slot_id >= self.header.slot_count {
                continue; // Skip invalid slot IDs
            }

            let slot_offset = slot_dir_start + (*slot_id as usize * mem::size_of::<SlotEntry>());
            if slot_offset + mem::size_of::<SlotEntry>() > self.data.len() {
                continue;
            }

            self.data[slot_offset..slot_offset + 2]
                .copy_from_slice(&slot_entry.offset.to_le_bytes());
            self.data[slot_offset + 2..slot_offset + 4]
                .copy_from_slice(&slot_entry.length.to_le_bytes());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_creation() {
        let page = Page::new(1, PageType::Data);
        assert_eq!(page.page_id(), 1);
        assert_eq!(page.page_type(), PageType::Data);
        assert_eq!(page.slot_count(), 0);
        assert!(!page.is_dirty());
    }

    #[test]
    fn test_record_insertion() {
        let mut page = Page::new(1, PageType::Data);
        let data = b"Hello, World!";

        let slot_id = page.insert_record(data).unwrap();
        assert_eq!(slot_id, 0);
        assert_eq!(page.slot_count(), 1);
        assert!(page.is_dirty());

        let retrieved = page.get_record(slot_id).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_record_update() {
        let mut page = Page::new(1, PageType::Data);
        let data = b"Hello, World!";

        let slot_id = page.insert_record(data).unwrap();

        let new_data = b"Hello, Rust!";
        assert!(new_data.len() <= data.len()); // Must fit in existing slot

        page.update_record(slot_id, new_data).unwrap();

        let retrieved = page.get_record(slot_id).unwrap();
        assert_eq!(&retrieved[..new_data.len()], new_data);
    }

    #[test]
    fn test_record_deletion() {
        let mut page = Page::new(1, PageType::Data);
        let data = b"Hello, World!";

        let slot_id = page.insert_record(data).unwrap();
        assert_eq!(page.slot_count(), 1);

        page.delete_record(slot_id).unwrap();
        assert!(page.get_record(slot_id).is_err());
    }

    #[test]
    fn test_multiple_records() {
        let mut page = Page::new(1, PageType::Data);

        let records = vec![b"Record 1", b"Record 2", b"Record 3"];
        let mut slot_ids = Vec::new();

        for record in &records {
            let slot_id = page.insert_record(*record).unwrap();
            slot_ids.push(slot_id);
        }

        assert_eq!(page.slot_count(), 3);

        for (i, slot_id) in slot_ids.iter().enumerate() {
            let retrieved = page.get_record(*slot_id).unwrap();
            assert_eq!(retrieved, records[i]);
        }
    }

    #[test]
    fn test_insufficient_space() {
        let mut page = Page::new(1, PageType::Data);

        // Try to insert a record larger than page capacity
        let large_data = vec![0u8; MAX_PAGE_DATA_SIZE + 1];

        let result = page.insert_record(&large_data);
        assert!(matches!(result, Err(PageError::RecordTooLarge { .. })));
    }

    #[test]
    fn test_page_serialization() {
        let mut page = Page::new(1, PageType::Data);

        // Insert a record
        let slot_id = page.insert_record(b"test data").unwrap();
        assert_eq!(slot_id, 0);

        // Serialize to bytes
        let bytes = page.to_bytes().unwrap();
        assert_eq!(bytes.len(), PAGE_SIZE);

        // Create new page from bytes
        let loaded_page = Page::from_bytes(1, bytes.to_vec()).unwrap();

        // Verify the record is still there
        let record = loaded_page.get_record(slot_id).unwrap();
        assert_eq!(record, b"test data");
    }
}
