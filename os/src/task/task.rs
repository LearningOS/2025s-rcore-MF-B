//! Types related to task management
use crate::config::MAX_SYSCALL_NUM;

use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The task trace
    pub task_trace: TaskTrace,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

#[derive(Clone, Copy)]
pub struct TaskTrace {
    pub syscall_map: [(usize,usize); MAX_SYSCALL_NUM],
    pub syscall_count: usize,
}

impl TaskTrace {
    pub fn new() -> Self {
        TaskTrace {
            syscall_map: [(0, 0); MAX_SYSCALL_NUM],
            syscall_count: 0,
        }
    }

    pub fn record_syscall_count(&mut self, syscall_id: usize) {
        // 如果已经存在这个系统调用的记录，就增加它的计数
        for i in 0..self.syscall_count {
            if self.syscall_map[i].0 == syscall_id {
                self.syscall_map[i].1 += 1;
                return;
            }
        }
        // 不存在则添加新记录
        if self.syscall_count < MAX_SYSCALL_NUM {
            self.syscall_map[self.syscall_count] = (syscall_id, 1);
            self.syscall_count += 1;
        }
        // 数组满则panic
        else {
            error!("syscall_map is full");
        }
    }

    pub fn get_syscall_count(&self, syscall_id: usize) -> usize {
        for i in 0..self.syscall_count {
            if self.syscall_map[i].0 == syscall_id {
                return self.syscall_map[i].1;
            }
        }
        0
    }
}