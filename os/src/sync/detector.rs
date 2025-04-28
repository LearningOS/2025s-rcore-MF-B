use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

use crate::task::current_task;

use super::UPSafeCell;

/// 线程Mutex死锁检测器
pub struct Detector {
    /// 资源表
    pub inner: UPSafeCell<DetectorInner>,
}

impl Detector {
    /// 创建一个新的MutexDetector
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(DetectorInner {
                    available: BTreeMap::new(),
                    allocation: BTreeMap::new(),
                    need: BTreeMap::new(),
                })
            },
        }
    }
}

/// 线程Mutex死锁检测器可变部分
pub struct DetectorInner {
    /// 在create时增加;一个锁代表一个资源,其值为count,Mutex资源默认为1,信号量由值决定.
    pub available: BTreeMap<usize, u8>,
    /// 从唤醒->unlock这段时间线程持有的资源
    pub allocation: BTreeMap<usize, BTreeMap<usize, u8>>,
    /// lock等待中的线程
    pub need: BTreeMap<usize, BTreeMap<usize, u8>>,
}

impl DetectorInner {
    /// 初始化可用锁
    pub fn set_available(&mut self, id: usize, count: usize) {
        self.available.insert(id, count as u8);
    }
    
    /// 添加可用锁
    pub fn add_available(&mut self, id: usize) {
        *self.available.entry(id).or_insert(0) += 1;
    }
    
    /// 将锁设为不可用
    pub fn remove_available(&mut self, id: usize) {
        if let Some(count) = self.available.get_mut(&id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
    
    /// 添加分配锁
    pub fn add_allocation(&mut self, id: usize) {
        let task = current_task().unwrap();
        let thread_id = task.inner_exclusive_access().res.as_ref().unwrap().tid;
        
        let thread_alloc = self.allocation.entry(thread_id).or_insert_with(BTreeMap::new);
        *thread_alloc.entry(id).or_insert(0) += 1;
    }
    
    /// 移除分配锁
    pub fn remove_allocation(&mut self, id: usize) {
        let task = current_task().unwrap();
        let thread_id = task.inner_exclusive_access().res.as_ref().unwrap().tid;
        
        if let Some(thread_alloc) = self.allocation.get_mut(&thread_id) {
            if let Some(count) = thread_alloc.get_mut(&id) {
                if *count > 0 {
                    *count -= 1;
                }
                if *count == 0 {
                    thread_alloc.remove(&id);
                }
            }
        }
    }
    
    /// 添加需要锁
    pub fn add_need(&mut self, id: usize) {
        let task = current_task().unwrap();
        let thread_id = task.inner_exclusive_access().res.as_ref().unwrap().tid;
        
        let thread_need = self.need.entry(thread_id).or_insert_with(BTreeMap::new);
        *thread_need.entry(id).or_insert(0) += 1;
    }
    
    /// 移除需要锁
    pub fn remove_need(&mut self, id: usize) {
        let task = current_task().unwrap();
        let thread_id = task.inner_exclusive_access().res.as_ref().unwrap().tid;
        
        if let Some(thread_need) = self.need.get_mut(&thread_id) {
            if let Some(count) = thread_need.get_mut(&id) {
                if *count > 0 {
                    *count -= 1;
                }
                if *count == 0 {
                    thread_need.remove(&id);
                }
            }
        }
    }

    /// 检测死锁
    pub fn detect_deadlock(&self) -> bool {
        // 创建一个包含所有线程ID的集合
        let mut all_thread_ids = Vec::new();
        for &thread_id in self.allocation.keys() {
            if !all_thread_ids.contains(&thread_id) {
                all_thread_ids.push(thread_id);
            }
        }
        for &thread_id in self.need.keys() {
            if !all_thread_ids.contains(&thread_id) {
                all_thread_ids.push(thread_id);
            }
        }
        
        // 准备工作向量
        let mut work = self.available.clone();
        let mut finish = vec![false; all_thread_ids.len()];
        
        let mut found = true;
        while found {
            found = false;
            for (idx, &thread_id) in all_thread_ids.iter().enumerate() {
                if !finish[idx] {
                    // 检查线程的所有资源需求是否能被满足
                    let mut can_allocate = true;
                    
                    if let Some(thread_need) = self.need.get(&thread_id) {
                        for (&res_id, &need_count) in thread_need {
                            let available_count = *work.get(&res_id).unwrap_or(&0);
                            if need_count > available_count {
                                can_allocate = false;
                                break;
                            }
                        }
                    }
                    
                    if can_allocate {
                        // 如果所有资源都能满足，则标记为完成，并释放已分配资源
                        if let Some(thread_alloc) = self.allocation.get(&thread_id) {
                            for (&res_id, &alloc_count) in thread_alloc {
                                *work.entry(res_id).or_insert(0) += alloc_count;
                            }
                        }
                        finish[idx] = true;
                        found = true; // 找到至少一个可以完成的线程，需要继续循环
                    }
                }
            }
        }
        
        // 检查是否所有线程都已完成
        finish.iter().any(|&x| !x) // 如果有未完成的线程，说明存在死锁
    }
}