//! Process management syscalls
use crate::{mm::{translated_pte, translated_to_mut, VirtAddr}, task::{change_program_brk, current_user_token, exit_current_and_run_next, get_task_trace, mmap, munmap, suspend_current_and_run_next}, timer::get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    if let Some(ts) = translated_to_mut::<TimeVal>(current_user_token(), _ts){
        ts.sec = us / 1_000_000;
        ts.usec = us % 1_000_000;
        0
    }
    else{
        error!("kernel: sys_get_time: invalid address");
        -1
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    let va: VirtAddr = _id.into();
    let pte = translated_pte(token, va);
    match _trace_request {
        0 => {
            if let Some(pte) = pte {
                if !pte.is_valid() || !pte.readable() || !pte.is_user() {
                    return -1;
                }
                let ppn = pte.ppn().0;
                let offset = va.page_offset();
                let addr = (ppn << 12) + offset;
                return (unsafe { (addr as *const u8).read_volatile() }) as isize;
            } else {
                error!("kernel: sys_trace: invalid address");
                return -1;
            }
        },
        1 => {
            if let Some(pte) = pte {
                if !pte.is_valid() || !pte.writable() || !pte.is_user() {
                    return -1;
                }
                let ppn = pte.ppn().0;
                let offset = va.page_offset();
                let addr = ((ppn << 12) + offset) as *mut u8;
                unsafe { 
                    addr.write_volatile(_data as u8); 
                };
                0
            } else {
                error!("kernel: sys_trace: invalid address");
                return -1;
            }
        },
        2 => {
            get_task_trace(_id) as isize
        },
        _ => {
            error!("kernel: sys_trace: invalid trace request {}", _trace_request);	
            -1
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
