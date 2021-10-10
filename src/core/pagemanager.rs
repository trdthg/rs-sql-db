#![allow(dead_code, unused_variables, unused)]

use std::cell::RefCell;
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::{size_of, take};
use std::os::windows::prelude::FileExt;
use std::rc::{Rc, Weak};

use super::super::tree::bplustree::*;

const PAGE_SIZE: usize = 152;
const HEADER_SIZE: usize = size_of::<FileHeader>() + size_of::<PageHeader>();

#[derive(Debug, Clone)]
struct FileHeader {
    file_page_offset: usize, // 整个表中的第几个页
    file_page_next: usize,   // 下一个页的位置
}
impl FileHeader {
    pub fn new(file_page_offset: usize, file_page_next: usize) -> Self {
        FileHeader {
            file_page_offset,
            file_page_next,
        }
    }
}

#[derive(Debug, Clone)]
struct PageHeader {
    page_heap_top: usize,    // 首个数据的位置,
    page_n_heap: usize,      // 堆中的记录数,
    page_last_insert: usize, // 最后插入的位置,
    page_leval: usize,       // 表示当前页在索引树的位置, 就是第几层, 0表示叶节点, 向上递增,
    page_index_id: usize,    // 索引id, 表示当前页属于那个索引, (好像是指father)
}
impl PageHeader {
    pub fn new(
        page_heap_top: usize,
        page_n_heap: usize,
        page_last_insert: usize,
        page_leval: usize,
        page_index_id: usize,
    ) -> Self {
        PageHeader {
            page_heap_top,
            page_n_heap,
            page_last_insert,
            page_leval,
            page_index_id,
        }
    }
}

#[derive(Debug, Clone)]
struct RowData {
    next: usize,
    id: usize,
    data: Vec<u8>,
}
#[derive(Debug, Clone)]
struct RowIndex {
    next: usize,
    id: usize,
    pos: usize, // 对应页节点的page_id
}

#[derive(Debug, Clone)]
struct DataRecord {
    row: Vec<RowData>,
}
#[derive(Debug, Clone)]

struct IndexRecord {
    row: Vec<RowIndex>,
}
#[derive(Debug, Clone)]
struct IndexPage {
    fileheader: FileHeader,
    pageheader: PageHeader,
    indexrecord: IndexRecord,
}
impl IndexPage {
    pub fn to_vec_u8(&self) -> Vec<u8> {
        let mut s: Vec<u8> = Vec::new();
        let (fileheader, pageheader, indexrecord) =
            (&self.fileheader, &self.pageheader, &self.indexrecord);
        for i in [
            fileheader.file_page_offset,
            fileheader.file_page_next,
            pageheader.page_heap_top,
            pageheader.page_n_heap,
            pageheader.page_last_insert,
            pageheader.page_leval,
            pageheader.page_index_id,
        ] {
            let tmp = i.to_ne_bytes();
            s.append(&mut tmp.to_vec());
        }

        for row in &indexrecord.row {
            s.append(&mut row.next.to_ne_bytes().to_vec());
            s.append(&mut row.id.to_ne_bytes().to_vec());
            s.append(&mut row.pos.to_ne_bytes().to_vec());
        }
        s
    }
}
#[derive(Debug, Clone)]
struct DataPage {
    fileheader: FileHeader,
    pageheader: PageHeader,
    datarecord: DataRecord,
}
impl DataPage {
    pub fn to_vec_u8(&self) -> Vec<u8> {
        let mut s: Vec<u8> = Vec::new();
        let fileheader = &self.fileheader;
        let pageheader = &self.pageheader;
        let datarecord = &self.datarecord;
        for i in [
            fileheader.file_page_offset,
            fileheader.file_page_next,
            pageheader.page_heap_top,
            pageheader.page_n_heap,
            pageheader.page_last_insert,
            pageheader.page_leval,
            pageheader.page_index_id,
        ] {
            let tmp = i.to_ne_bytes();
            s.append(&mut tmp.to_vec());
        }

        for row in &datarecord.row {
            // s.push_str(&format!("{}{}", row.next, row.data));
            s.append(&mut row.next.to_ne_bytes().to_vec());
            s.append(&mut row.id.to_ne_bytes().to_vec());
            s.append(&mut row.data.clone());
        }
        s
    }

    pub fn from_leaf_node(leaf: Rc<RefCell<LeafNode>>) -> Self {
        let ids = leaf.borrow().ids.clone();
        let len = ids.len();

        let file_page_offset = 0;
        let start_offset = file_page_offset * PAGE_SIZE;
        // 创建一个数据块
        let fileheader = FileHeader::new(0, 1);
        let start_row_offset = start_offset + HEADER_SIZE;
        let mut pageheader = PageHeader::new(start_row_offset, len, HEADER_SIZE, 0, 0);

        let mut next_offset = start_row_offset;
        let datarecord = {
            let mut datarecord = DataRecord { row: vec![] };
            for tuple in ids {
                next_offset += size_of::<usize>() * 2 + tuple.data.as_bytes().len();
                datarecord.row.push(RowData {
                    next: next_offset,
                    id: tuple.id,
                    data: tuple.data.as_bytes().to_vec(),
                });
            }
            datarecord
        };

        pageheader.page_last_insert = next_offset;
        // pageheader.page_last_insert = next_offset;
        // 写入
        let indexpage = DataPage {
            fileheader,
            pageheader,
            datarecord,
        };
        indexpage
    }

    pub fn push(&mut self, id: usize, data: Vec<u8>) {
        let mut start_row_offset = self.pageheader.page_heap_top;
        let next_offset = self.pageheader.page_last_insert;
        let new_len = size_of::<usize>() * 2 + data.len();
        self.datarecord.row.push(RowData {
            next: (next_offset + new_len),
            id,
            data: data,
        });
        self.datarecord.row.sort_by_key(|row| row.id);
        self.datarecord
            .row
            .iter_mut()
            .enumerate()
            .for_each(|(i, row)| {
                row.next = start_row_offset + size_of::<usize>() * 2 + row.data.len();
                start_row_offset += size_of::<usize>() * 2 + row.data.len();
            });
        // 重置某些属性
        self.pageheader.page_last_insert = start_row_offset;
        self.pageheader.page_n_heap += 1;
    }
}

#[derive(Debug, Clone)]
enum PageType {
    Index(IndexPage),
    Data(DataPage),
}

pub(crate) struct PageManager {
    f: File,
    max_page_id: usize,
    root_page_id: usize,
}
impl PageManager {
    pub fn create(file_name: &str) -> Self {
        let f = File::create(file_name).unwrap();
        f.seek_write(&[b'0'], 64 * PAGE_SIZE as u64).unwrap();
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_name)
            .unwrap();
        PageManager {
            f,
            max_page_id: 0,
            root_page_id: 0,
        }
    }

    pub fn read_file(filename: &str) -> Self {
        if let Ok(f) = OpenOptions::new().read(true).write(true).open(filename) {
            let max_page_id = Self::get_max_page_id(&f);
            PageManager {
                f,
                max_page_id,
                root_page_id: 0,
            }
        } else {
            Self::create(filename)
        }
    }

    pub fn from_tree(tree: &BPlusTree) -> Self {
        let name = tree.name.as_str();
        let file_name = format!("{}.db", name);
        let f = File::create(file_name.as_str()).unwrap();
        f.seek_write(&[b'0'], 64 * PAGE_SIZE as u64).unwrap();

        let node = &tree.root;
        match tree.root.clone() {
            Some(LinkType::Leaf(leaf)) => {
                // TODO root是根节点
                let datapage = DataPage::from_leaf_node(leaf);
                let mut s: Vec<u8> = datapage.to_vec_u8();

                f.seek_write(&s, 0).unwrap();
                let f = File::open(file_name.clone()).unwrap();
            }
            Some(LinkType::Branch(node)) => {
                // TODO
                // * 得到了堆

                // 记录branch节点
                let mut vec = Vec::new();
                // 记录leaf节点
                let mut vec2 = Vec::new();
                // 初始化
                //          node, father_page_id, depth
                vec.push((node.clone(), 0usize, 1usize, 0usize));
                let mut page_id = 0;
                // 准备写入的文件
                while vec.len() != 0 {
                    // ! =============
                    let a = vec.pop().unwrap();
                    let ids = a.0.borrow().ids.clone();
                    let len = ids.len();

                    // ! ------------- 创建一个字符流
                    let file_page_offset = page_id;
                    let start_offset = file_page_offset * PAGE_SIZE;
                    // ? PART I => file_header
                    let fileheader = FileHeader::new(file_page_offset, page_id + 1);
                    let start_row_offset = start_offset + HEADER_SIZE;
                    // ? PART II => page_header
                    let mut pageheader =
                        PageHeader::new(start_row_offset, len, HEADER_SIZE, a.2, a.1);

                    let mut next_offset = start_row_offset;
                    let indexrecord = {
                        let mut userrecord = IndexRecord { row: vec![] };
                        for tuple in &ids {
                            next_offset += size_of::<usize>() * 3;
                            match tuple.id {
                                // ? PART III => user_record
                                Some(id) => userrecord.row.push(RowIndex {
                                    next: next_offset,
                                    id: id,
                                    pos: 1789,
                                }),
                                None => userrecord.row.push(RowIndex {
                                    next: next_offset,
                                    id: 0,
                                    pos: 1789,
                                }),
                            }
                        }
                        userrecord
                    };

                    pageheader.page_last_insert = next_offset;

                    let indexpage = IndexPage {
                        fileheader,
                        pageheader,
                        indexrecord,
                    };
                    let mut s: Vec<u8> = indexpage.to_vec_u8();
                    f.seek_write(&s, start_offset as u64).unwrap();
                    let child_nth: u64 = a.3 as u64;
                    let father_nth: u64 = a.1 as u64;
                    f.seek_write(
                        &page_id.to_ne_bytes()[..],
                        father_nth * PAGE_SIZE as u64 + HEADER_SIZE as u64 + 24 * child_nth + 16,
                    );
                    // ! ------------- 准备后继节点
                    let mut tmp_i = 0;
                    for tuple in a.0.borrow().ids.clone() {
                        match tuple.link {
                            // 插入当前的page_id就是child的father_page_id
                            LinkType::Branch(branch) => {
                                vec.insert(0, (branch.clone(), page_id, a.2 + 1, tmp_i));
                            }
                            LinkType::Leaf(leaf) => {
                                vec2.insert(0, (leaf, page_id, a.2 + 1, tmp_i));
                            }
                            _ => {}
                        }
                        tmp_i += 1;
                    }
                    println!("page_id: {} father_id: {}", page_id, a.1);
                    page_id += 1;
                }
                while vec2.len() != 0 {
                    let a = vec2.pop().unwrap();

                    let ids = a.0.borrow().ids.clone();
                    let len = ids.len();

                    let file_page_offset = page_id;
                    let start_offset = file_page_offset * PAGE_SIZE;
                    // 创建一个数据块
                    let fileheader = FileHeader::new(file_page_offset, page_id + 1);
                    let start_row_offset = start_offset + HEADER_SIZE;
                    let mut pageheader =
                        PageHeader::new(start_row_offset, len, HEADER_SIZE, 0, a.1);

                    let mut next_offset = start_row_offset;
                    let datarecord = {
                        let mut datarecord = DataRecord { row: vec![] };
                        for tuple in ids {
                            next_offset += size_of::<usize>() * 2 + tuple.data.as_bytes().len();
                            datarecord.row.push(RowData {
                                next: next_offset,
                                id: tuple.id,
                                data: tuple.data.as_bytes().to_vec(),
                            });
                        }
                        datarecord
                    };
                    pageheader.page_last_insert = next_offset;

                    let datapage = DataPage {
                        fileheader,
                        pageheader,
                        datarecord,
                    };
                    let mut s = datapage.to_vec_u8();
                    // pageheader.page_last_insert = next_offset;
                    // 写入
                    f.seek_write(&s, start_offset.try_into().unwrap()).unwrap();
                    let child_nth: u64 = a.3.try_into().unwrap();
                    let father_nth: u64 = a.1.try_into().unwrap();
                    f.seek_write(
                        &page_id.to_ne_bytes()[..],
                        father_nth * PAGE_SIZE as u64 + HEADER_SIZE as u64 + 24 * child_nth + 16,
                    );

                    println!("page_id: {} father_id: {} leaf=true", page_id, a.1);
                    page_id += 1;
                }
            }
            None => {
                // TODO root为空, 新建一个区64个页, 每页16kb
            }
            _ => {}
        }
        Self::read_file(file_name.as_str())
    }

    pub fn show(&self) {
        let f = &self.f;
        for i in 0..=self.max_page_id as u64 {
            println!("-------------------------------");
            let start_offset = i * PAGE_SIZE as u64;
            let mut buf = [0; 8];
            let mut sbuf = [0; 1024];

            // 读取header
            let mut vec = vec![];
            for i in 0..7 {
                match f.seek_read(&mut buf, start_offset + i * 8) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("{}", "表为空");
                        return;
                    }
                }
                print!("{:?} ", usize::from_ne_bytes(buf));
                vec.push(usize::from_ne_bytes(buf));
            }
            println!("");
            let level = vec[5];
            // 判断节点类型, 读取record
            if level > 0 {
                // 是branch节点, 有next, id, pos,
                let node_n: u64 = vec[3].try_into().unwrap();
                for i in 0..(node_n) {
                    f.seek_read(&mut buf, start_offset + HEADER_SIZE as u64 + i * 24 + 8 * 0)
                        .unwrap();
                    print!("{:?} ", usize::from_ne_bytes(buf));
                    f.seek_read(&mut buf, start_offset + HEADER_SIZE as u64 + i * 24 + 8 * 1)
                        .unwrap();
                    print!("{:?} ", usize::from_ne_bytes(buf));
                    f.seek_read(&mut buf, start_offset + HEADER_SIZE as u64 + i * 24 + 8 * 2)
                        .unwrap();
                    println!("{:?} ", usize::from_ne_bytes(buf));
                }
            } else {
                // 是叶节点, 需要分离出next, id, data
                let mut row_start: u64 = start_offset + HEADER_SIZE as u64;
                for i in 0..vec[3] {
                    // 将next读入buf
                    f.seek_read(&mut buf, row_start);
                    let row_next: usize = usize::from_ne_bytes(buf);
                    let row_next_u64: u64 = u64::from_ne_bytes(buf);
                    let row_start_usize: usize = row_start.try_into().unwrap();
                    let len: usize = (row_next - row_start_usize);
                    // 将id读入buf
                    f.seek_read(&mut buf, row_start + 8 * 1);
                    let id = usize::from_ne_bytes(buf);
                    // 将data读入sbuf
                    f.seek_read(&mut sbuf, row_start + 8 * 2);
                    let s = String::from_utf8_lossy(&sbuf[0..len - 16]).to_string();

                    println!("{} {} {}", row_start, id, s);

                    row_start = row_next_u64;
                }
            }
        }
    }

    fn get_max_page_id(f: &File) -> usize {
        // 读取两页
        let mut max_page_id = 0;
        for i in 0..64 {
            println!("-------------------------------");
            let start_offset = i * PAGE_SIZE as u64;
            let mut buf = [0; 8];
            let mut sbuf = [0; 1024];

            // 读取前7个数字
            let mut vec = vec![];
            for i in 0..7 {
                f.seek_read(&mut buf, start_offset + i * 8).unwrap();
                print!("{:?} ", usize::from_ne_bytes(buf));
                vec.push(usize::from_ne_bytes(buf));
            }

            if vec[1] == 0 {
                break;
            }
            max_page_id += 1;
        }
        max_page_id - 1
    }

    fn get_page(&self, i: u64) -> Option<PageType> {
        let f = &self.f;
        let start_offset = i * PAGE_SIZE as u64;
        let mut buf = [0; 8];
        let mut sbuf = [0; 1024];
        let mut vec = vec![];
        for i in 0..7 {
            match f.seek_read(&mut buf, start_offset + i * 8) {
                Ok(_) => {}
                Err(e) => {
                    return None;
                    break;
                }
            }
            print!("{:?} ", usize::from_ne_bytes(buf));
            vec.push(usize::from_ne_bytes(buf));
        }
        if vec[1] == 0 {
            return None;
        }
        let fileheader: FileHeader = FileHeader::new(vec[0], vec[1]);
        let pageheader: PageHeader = PageHeader::new(vec[2], vec[3], vec[4], vec[5], vec[6]);
        let level = vec[5];
        if level > 0 {
            let mut indexrecord = IndexRecord { row: vec![] };
            let node_n: u64 = vec[3].try_into().unwrap();
            for i in 0..(node_n) {
                f.seek_read(&mut buf, start_offset + HEADER_SIZE as u64 + i * 24 + 8 * 0)
                    .unwrap();
                let next = usize::from_ne_bytes(buf);
                f.seek_read(&mut buf, start_offset + HEADER_SIZE as u64 + i * 24 + 8 * 1)
                    .unwrap();
                let id = usize::from_ne_bytes(buf);
                f.seek_read(&mut buf, start_offset + HEADER_SIZE as u64 + i * 24 + 8 * 2)
                    .unwrap();
                let pos = usize::from_ne_bytes(buf);
                indexrecord.row.push(RowIndex { next, id, pos });
            }
            let indexpage = IndexPage {
                fileheader,
                pageheader,
                indexrecord,
            };
            return Some(PageType::Index(indexpage));
        } else {
            let mut datarecord: DataRecord = DataRecord { row: vec![] };
            let mut row_start: u64 = start_offset + HEADER_SIZE as u64;
            for i in 0..vec[3] {
                f.seek_read(&mut buf, row_start);
                let row_next: usize = usize::from_ne_bytes(buf);
                let row_next_u64: u64 = u64::from_ne_bytes(buf);
                let row_start_usize: usize = row_start.try_into().unwrap();
                let len: usize = (row_next - row_start_usize);
                f.seek_read(&mut buf, row_start + 8 * 1);
                let id = usize::from_ne_bytes(buf);
                f.seek_read(&mut sbuf, row_start + 8 * 2);
                // let s = String::from_utf8_lossy(&sbuf[0..len - 16]).to_string();
                datarecord.row.push(RowData {
                    next: row_next,
                    id,
                    data: sbuf[0..len - 16].to_vec(),
                });
                row_start = row_next_u64;
            }
            let datapage = DataPage {
                fileheader,
                pageheader,
                datarecord,
            };
            return Some(PageType::Data(datapage));
        }
    }

    pub fn select_recursive(&self, page_id: usize, id: usize) -> Option<Vec<u8>> {
        let page_id: u64 = page_id.try_into().unwrap();
        let node = Self::get_page(self, page_id);
        match node {
            Some(PageType::Data(node)) => {
                for row in node.datarecord.row {
                    if id == row.id {
                        return Some(row.data);
                    }
                }
                return None;
            }
            Some(PageType::Index(node)) => {
                let ids: Vec<usize> = node
                    .indexrecord
                    .row
                    .iter()
                    .filter(|row| row.id != 0)
                    .map(|row| row.id)
                    .collect();
                println!("{:?} {}", ids, id);
                let pos = ids.binary_search(&id).unwrap_or_else(|x| x);
                let page_id = node.indexrecord.row[pos].pos;
                return Self::select_recursive(self, page_id, id);
            }
            None => None,
        }
    }

    pub fn insert(&mut self, id: usize, data: Vec<u8>) {
        let mut page_id: usize = self.root_page_id;

        // TODO 判断是否为空
        match Self::get_page(&self, page_id as u64) {
            Some(PageType::Data(mut node)) => {
                // ! 直接插入一条datarecord
                node.push(id, data);
                if node.pageheader.page_last_insert <= (page_id + 1) * PAGE_SIZE {
                    let s = node.to_vec_u8();
                    println!("{:#?}", node);
                    self.f.seek_write(
                        &s,
                        node.fileheader.file_page_offset as u64 * PAGE_SIZE as u64,
                    );
                } else {
                    // ! 分裂页
                    Self::split_page(self, PageType::Data(node));
                }
            }
            Some(PageType::Index(node)) => {
                // TODO Find insert node
                let mut vec = vec![];
                let mut page_id = self.root_page_id;
                while let Some(PageType::Index(node)) = Self::get_page(&self, page_id as u64) {
                    let ids: Vec<usize> = node.indexrecord.row.iter().map(|row| row.id).collect();
                    let pos = ids.binary_search(&id).unwrap_or_else(|x| x);
                    vec.push(page_id);
                    page_id = node.indexrecord.row[pos - 1].pos;
                }
                if let Some(PageType::Data(mut node)) = Self::get_page(&self, page_id as u64) {
                    node.push(id, data);
                    if node.pageheader.page_last_insert <= (page_id + 1) * PAGE_SIZE as usize {
                        let s = node.to_vec_u8();
                        println!("{:#?}", node);
                        self.f.seek_write(
                            &s,
                            node.fileheader.file_page_offset as u64 * PAGE_SIZE as u64,
                        );
                    } else {
                        // ! 分裂页
                        match Self::split_page(self, PageType::Data(node)) {
                            Some(PageType::Index(tmp_node)) => {
                                let mut tmp_node = tmp_node;
                                while let Some(PageType::Index(index_node)) =
                                    Self::split_page(self, PageType::Index(tmp_node.clone()))
                                {
                                    tmp_node = index_node;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            None => {
                let next = HEADER_SIZE + 24 + data.len();
                let datapage = DataPage {
                    fileheader: FileHeader::new(0, 1),
                    pageheader: PageHeader::new(HEADER_SIZE, 1, next, 0, 0),
                    datarecord: DataRecord {
                        row: vec![RowData {
                            next: next,
                            id,
                            data: data,
                        }],
                    },
                };
                let s = datapage.to_vec_u8();
                let obh = self.f.seek_write(&s, 0).unwrap();
                self.root_page_id = 0;
                self.max_page_id = 0;
                return;
            }
        }
    }

    fn split_page(&mut self, node: PageType) -> Option<PageType> {
        match node {
            PageType::Data(mut node) => {
                // ! 初始化
                let is_new_root = if node.fileheader.file_page_offset == self.root_page_id {
                    true
                } else {
                    false
                };
                let mut right = node.clone();
                let len = node.datarecord.row.len();
                let max_page_id = self.max_page_id;
                // ! 左节点
                node.datarecord.row = node.datarecord.row[0..(len / 2)].to_vec();
                let mut start_row_offset = node.pageheader.page_heap_top;
                node.datarecord
                    .row
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, row)| {
                        start_row_offset += size_of::<usize>() * 2 + row.data.len();
                        row.next = start_row_offset;
                    });
                node.pageheader.page_last_insert = start_row_offset;
                node.pageheader.page_n_heap = len / 2;
                let s_left_offset = node.fileheader.file_page_offset as u64 * PAGE_SIZE as u64;
                // ! 右节点
                let start_page_offset = (max_page_id + 1) * PAGE_SIZE;
                let mut start_row_offset = start_page_offset + HEADER_SIZE;
                right.datarecord.row = right.datarecord.row[(len / 2)..].to_vec();
                right
                    .datarecord
                    .row
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, row)| {
                        row.next = start_row_offset + size_of::<usize>() * 2 + row.data.len();
                        start_row_offset += size_of::<usize>() * 2 + row.data.len();
                    });
                let right_n = {
                    if len % 2 == 1 {
                        len / 2 + 1
                    } else {
                        len / 2
                    }
                };
                right.fileheader = FileHeader::new(max_page_id + 1, max_page_id + 2);
                right.pageheader = PageHeader::new(
                    start_page_offset + HEADER_SIZE,
                    right_n,
                    start_row_offset,
                    node.pageheader.page_leval,
                    node.pageheader.page_index_id,
                );
                let s_right_offset = start_page_offset as u64;
                // ! 判断是否为根页进行分离
                if is_new_root {
                    // TODO 自己是根节点, 需要创建新的根节点
                    let start_page_offset = (max_page_id + 2) * PAGE_SIZE;
                    let indexpage = IndexPage {
                        fileheader: FileHeader::new(max_page_id + 2, max_page_id + 3),
                        pageheader: PageHeader::new(
                            start_page_offset + HEADER_SIZE,
                            2,
                            start_page_offset + HEADER_SIZE + 24 * 2,
                            1,
                            max_page_id + 2,
                        ),
                        indexrecord: IndexRecord {
                            row: vec![
                                RowIndex {
                                    next: start_page_offset + HEADER_SIZE + 24,
                                    id: 0,
                                    pos: node.fileheader.file_page_offset,
                                },
                                RowIndex {
                                    next: start_page_offset + HEADER_SIZE + 24 * 2,
                                    id: right.datarecord.row[0].id,
                                    pos: right.fileheader.file_page_offset,
                                },
                            ],
                        },
                    };
                    let s_top = indexpage.to_vec_u8();
                    let obh = self
                        .f
                        .seek_write(
                            &s_top,
                            indexpage.fileheader.file_page_offset as u64 * PAGE_SIZE as u64,
                        )
                        .unwrap();

                    node.pageheader.page_index_id = indexpage.fileheader.file_page_offset;
                    right.pageheader.page_index_id = indexpage.fileheader.file_page_offset;
                    let s_left = node.to_vec_u8();
                    self.f.seek_write(&s_left, s_left_offset);
                    let s_right = right.to_vec_u8();
                    self.f.seek_write(&s_right, s_right_offset);

                    self.root_page_id = indexpage.fileheader.file_page_offset;
                    self.max_page_id += 2;

                    return None;
                } else {
                    let s_left = node.to_vec_u8();
                    self.f.seek_write(&s_left, s_left_offset);
                    let s_right = right.to_vec_u8();
                    self.f.seek_write(&s_right, s_right_offset);
                    self.max_page_id += 1;

                    if let Some(PageType::Index(mut new_top)) =
                        Self::get_page(&self, node.pageheader.page_index_id as u64)
                    {
                        let mut start_row_offset =
                            new_top.fileheader.file_page_offset * PAGE_SIZE + HEADER_SIZE;
                        // ! 获取中间值作为id
                        let id = right.datarecord.row[0].id;
                        new_top.indexrecord.row.push(RowIndex {
                            next: new_top.pageheader.page_last_insert + 8 * 3,
                            id,
                            pos: max_page_id + 1,
                        });
                        new_top.indexrecord.row.sort_by_key(|row| row.id);
                        new_top
                            .indexrecord
                            .row
                            .iter_mut()
                            .enumerate()
                            .for_each(|(i, row)| {
                                start_row_offset += size_of::<usize>() * 3;
                                row.next = start_row_offset;
                            });
                        new_top.pageheader.page_n_heap += 1;
                        new_top.pageheader.page_last_insert = start_row_offset;
                        let s = new_top.to_vec_u8();
                        println!("{:#?}", new_top);

                        // TODO 向上递归, 是否是新的根爷, 重置左右节点的父节点
                        if new_top.pageheader.page_last_insert
                            <= (new_top.fileheader.file_page_offset + 1) * PAGE_SIZE
                        {
                            self.f.seek_write(
                                &s,
                                new_top.fileheader.file_page_offset as u64 * PAGE_SIZE as u64,
                            );
                        } else {
                            return Some(PageType::Index(new_top));
                        }
                    }
                    None
                }
            }
            PageType::Index(mut node) => {
                let is_new_root = if node.pageheader.page_index_id == self.root_page_id {
                    true
                } else {
                    false
                };
                let mut right = node.clone();
                let max_page_id = self.max_page_id;
                let len = node.pageheader.page_n_heap;
                // ! 左节点
                let mut start_row_offset = node.pageheader.page_heap_top;
                node.indexrecord.row = node.indexrecord.row[0..(len / 2)].to_vec();
                node.indexrecord
                    .row
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, row)| {
                        start_row_offset += size_of::<usize>() * 3;
                        row.next = start_row_offset;
                    });
                node.pageheader.page_n_heap = len / 2;
                node.pageheader.page_last_insert = start_row_offset;
                let s_left_offset = node.fileheader.file_page_offset as u64 * PAGE_SIZE as u64;
                // ! 右节点
                let start_page_offset = (max_page_id + 1) * PAGE_SIZE;
                let mut start_row_offset = start_page_offset + HEADER_SIZE;
                right.indexrecord.row = right.indexrecord.row[(len / 2)..].to_vec();
                right
                    .indexrecord
                    .row
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, row)| {
                        row.next = start_row_offset + size_of::<usize>() * 3;
                        start_row_offset += size_of::<usize>() * 3;
                    });
                let right_n = {
                    if len % 2 == 1 {
                        len / 2 + 1
                    } else {
                        len / 2
                    }
                };
                right.fileheader = FileHeader::new(max_page_id + 1, max_page_id + 2);
                right.pageheader = PageHeader::new(
                    start_page_offset + HEADER_SIZE,
                    right_n,
                    start_row_offset,
                    node.pageheader.page_leval,
                    node.pageheader.page_index_id,
                );
                let s_right_offset = start_page_offset as u64;
                // TODO 判断是否需要新的根页
                if is_new_root {
                    // TODO 自己是根节点, 需要创建新的根节点
                    let start_page_offset = (max_page_id + 2) * PAGE_SIZE;
                    let indexpage = IndexPage {
                        fileheader: FileHeader::new(max_page_id + 2, max_page_id + 3),
                        pageheader: PageHeader::new(
                            start_page_offset + HEADER_SIZE,
                            2,
                            start_page_offset + HEADER_SIZE + 24 * 2,
                            1,
                            max_page_id + 2,
                        ),
                        indexrecord: IndexRecord {
                            row: vec![
                                RowIndex {
                                    next: start_page_offset + HEADER_SIZE + 24,
                                    id: 0,
                                    pos: node.fileheader.file_page_offset,
                                },
                                RowIndex {
                                    next: start_page_offset + HEADER_SIZE + 24 * 2,
                                    id: right.indexrecord.row[0].id,
                                    pos: right.fileheader.file_page_offset,
                                },
                            ],
                        },
                    };
                    let s_top = indexpage.to_vec_u8();
                    let obh = self
                        .f
                        .seek_write(
                            &s_top,
                            indexpage.fileheader.file_page_offset as u64 * PAGE_SIZE as u64,
                        )
                        .unwrap();

                    node.pageheader.page_index_id = indexpage.fileheader.file_page_offset;
                    right.pageheader.page_index_id = indexpage.fileheader.file_page_offset;
                    let s_left = node.to_vec_u8();
                    self.f.seek_write(&s_left, s_left_offset);
                    let s_right = right.to_vec_u8();
                    self.f.seek_write(&s_right, s_right_offset);

                    self.root_page_id = indexpage.fileheader.file_page_offset;
                    self.max_page_id += 2;

                    return None;
                } else {
                    let s_left = node.to_vec_u8();
                    self.f.seek_write(&s_left, s_left_offset);
                    let s_right = right.to_vec_u8();
                    self.f.seek_write(&s_right, s_right_offset);
                    self.max_page_id += 1;

                    let id = right.indexrecord.row[0].id;
                    if let Some(PageType::Index(mut node)) =
                        Self::get_page(&self, node.pageheader.page_index_id as u64)
                    {
                        let mut start_row_offset =
                            node.fileheader.file_page_offset * PAGE_SIZE + HEADER_SIZE;
                        node.indexrecord.row.push(RowIndex {
                            next: node.pageheader.page_last_insert + 8 * 3,
                            id,
                            pos: self.max_page_id + 1,
                        });
                        node.indexrecord.row.sort_by_key(|row| row.id);
                        node.indexrecord
                            .row
                            .iter_mut()
                            .enumerate()
                            .for_each(|(i, row)| {
                                start_row_offset += size_of::<usize>() * 3;
                                row.next = start_row_offset;
                            });
                        node.pageheader.page_n_heap += 1;

                        let s = node.to_vec_u8();
                        println!("{:#?}", node);

                        // TODO 是否需要像上递归
                        if len + node.pageheader.page_last_insert
                            <= (node.fileheader.file_page_offset + 1) * PAGE_SIZE
                        {
                            self.f.seek_write(
                                &s,
                                node.fileheader.file_page_offset as u64 * PAGE_SIZE as u64,
                            );
                        } else {
                            return Some(PageType::Index(node));
                        }
                    }
                }
                None
            }
        }
    }

    fn append_data(&self, node: PageType, page_id: usize, id: usize, data: &str) -> bool {
        if let Some(PageType::Data(node)) = Self::get_page(&self, page_id as u64) {
            let next_offset = node.pageheader.page_last_insert;
            let new_len = size_of::<usize>() * 2 + data.as_bytes().len();
            if new_len + next_offset <= (page_id + 1) * PAGE_SIZE {
                // * 追加一条datarecord
                let mut s: Vec<u8> = vec![];
                s.append(&mut (next_offset + new_len).to_ne_bytes().to_vec());
                s.append(&mut id.to_ne_bytes().to_vec());
                s.append(&mut data.as_bytes().to_vec());
                self.f.seek_write(&s[..], next_offset as u64).unwrap();
                // 修改node_n
                let right_n = node.pageheader.page_n_heap + 1;
                self.f.seek_write(
                    &right_n.to_ne_bytes(),
                    node.fileheader.file_page_offset as u64 * PAGE_SIZE as u64 + 24,
                );
                return true;
            } else {
                // ? 分裂页
                // 若不用递归,
                // father.node_n / 2, leaf.node_n / 2, new_leaf
                return false;
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{io::BufReader, mem::size_of};

    #[test]
    fn g() {
        let mut PageManager = PageManager::create("student.db");
        PageManager.insert(1, "sss".as_bytes().to_vec());
        PageManager.insert(2, "sss".as_bytes().to_vec());
        PageManager.insert(3, "sss".as_bytes().to_vec());
        PageManager.insert(4, "sss".as_bytes().to_vec());
        PageManager.insert(5, "sss".as_bytes().to_vec());
        PageManager.insert(6, "sss".as_bytes().to_vec());
        PageManager.insert(7, "sss".as_bytes().to_vec());
        PageManager.insert(8, "sss".as_bytes().to_vec());
        PageManager.insert(9, "sss".as_bytes().to_vec());
        PageManager.insert(10, "sss".as_bytes().to_vec());
        PageManager.insert(11, "sss".as_bytes().to_vec());
        PageManager.insert(12, "sss".as_bytes().to_vec());
        PageManager.insert(13, "sss".as_bytes().to_vec());
        PageManager.insert(2, "sss".as_bytes().to_vec());
        PageManager.insert(2, "sss".as_bytes().to_vec());
        PageManager.insert(2, "sss".as_bytes().to_vec());
        // PageManager.insert(14, "sss");
        println!("\n==============================");
        println!(
            "root_page_id: {} max_page_id: {}",
            PageManager.root_page_id, PageManager.max_page_id
        );
        PageManager.show();

        // let f = OpenOptions::new().read(true).open("student.db").unwrap();
        // let mut buf = [0; 8];
        // for i in 0..7 {
        //     f.seek_read(&mut buf, 0 + i * 8);
        //     println!("{}", usize::from_ne_bytes(buf));
        // }
    }

    #[test]
    fn f() {
        let mut tree = BPlusTree::new("user");
        tree.insert(1, "sss");
        let mut db = PageManager::from_tree(&tree);
        db.insert(2, "sss".as_bytes().to_vec());
        // db.insert(3, "sss");
        // db.insert(4, "sss");
        // db.insert(5, "sss");
        db.show();
    }

    #[test]
    fn d() {
        let mut tree = BPlusTree::new("user");
        for i in 1..=21 {
            tree.insert(i, "sss");
        }
        // println!("{:#?}", tree);
        let mut db = PageManager::from_tree(&tree);
        // db.show();
        db.insert(13, "a13".as_bytes().to_vec());
        db.insert(13, "a13".as_bytes().to_vec());
        db.insert(13, "a13".as_bytes().to_vec());
        db.insert(13, "a13".as_bytes().to_vec());

        db.insert(22, "新插入的数据".as_bytes().to_vec());
        db.insert(23, "a10".as_bytes().to_vec());
        db.insert(24, "a11".as_bytes().to_vec());
        db.insert(25, "a12".as_bytes().to_vec());
        db.insert(27, "a11".as_bytes().to_vec());
        db.insert(28, "a12".as_bytes().to_vec());
        db.insert(25, "a13".as_bytes().to_vec());
        db.show();
        let a = db.max_page_id;
        println!("{}", a);
    }

    #[test]
    fn e() {
        let manager = PageManager::read_file("assets/user.db");
        let page = manager.get_page(0);
        println!("{:#?}", page);
        let data = manager.select_recursive(0, 19);
        println!("{:?}", data);
    }

    #[test]
    fn c() {
        // "ssssss"
        let a = 0b00000010;
        println!("{}", a);
        let b = 12345678910123456789usize.to_ne_bytes();
        println!("{:?}", b);
        let c = usize::from_ne_bytes(b);
        println!("{:?}", c);
        let d = String::from_utf8_lossy(&b);
        println!("{}", d);

        let mut s = "ssssssssss".as_bytes().to_vec();
        let mut num = 123usize.to_ne_bytes().to_vec();
        let mut vec = Vec::new();
        vec.append(&mut s);
        vec.append(&mut num);
        let s = String::from_utf8_lossy(&vec);
        println!("{}", s);
    }

    #[test]
    fn read_partly() {
        let mut f = BufReader::new(File::open("assets/db1.txt").unwrap());

        let before = f.stream_position().unwrap();
        let s = &mut String::new();
        let after = f.stream_position().unwrap();

        println!("The first line was {} bytes long", after - before);
    }

    #[test]
    fn a() {
        use std::mem::size_of;
        println!("usize: {}", size_of::<usize>());
        println!("bplustree: {}", size_of::<BPlusTree>());
        println!("linktype: {}", size_of::<LinkType>());
        println!("branchnode: {}", size_of::<BranchNode>());
        println!("leafnode: {}", size_of::<LeafNode>());
        println!("datanode: {}", size_of::<DataNode>());
        println!("father: {}", size_of::<Option<Weak<RefCell<BranchNode>>>>());
    }

    #[test]
    fn aa() {
        for i in 0..=0 {
            println!("{}", i);
        }
    }
}
