//! Process management syscalls
use crate::{
    config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT, MAX_APP_NUM, USER_STACK_SIZE},
    loader::user_stack_addr,
    task::{exit_current_and_run_next, get_syscall_count, suspend_current_and_run_next},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// The kinds of requests that can be sent through the `sys_trace` syscall.
#[repr(usize)]
#[derive(Clone, Copy, PartialEq)]
enum TraceRequest {
    /// Read a byte from a user-space address
    Read = 0,
    /// Write a byte to a user-space address
    Write = 1,
    /// Query how many times a syscall has been invoked by the current task
    Syscall = 2,
}

/// Return `true` if `addr` belongs to a region that the user application is
/// allowed to read from or write to directly.
///
/// In chapter 3 there is no virtual memory, so the "user accessible" regions
/// are the fixed area where apps are loaded into memory
/// ([`APP_BASE_ADDRESS`]) plus the static user stack array
/// ([`user_stack_addr`]).
fn is_user_addr(addr: usize) -> bool {
    let app_lo = APP_BASE_ADDRESS;
    let app_hi = APP_BASE_ADDRESS + MAX_APP_NUM * APP_SIZE_LIMIT;
    let usk_lo = user_stack_addr();
    let usk_hi = usk_lo + MAX_APP_NUM * USER_STACK_SIZE;
    (addr >= app_lo && addr < app_hi) || (addr >= usk_lo && addr < usk_hi)
}

pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request as usize {
        r if r == TraceRequest::Read as usize => {
            if is_user_addr(id) {
                unsafe { (id as *const u8).read() as isize }
            } else {
                -1
            }
        }
        r if r == TraceRequest::Write as usize => {
            if is_user_addr(id) {
                unsafe {
                    (id as *mut u8).write(data as u8);
                }
                0
            } else {
                -1
            }
        }
        r if r == TraceRequest::Syscall as usize => {
            if id < crate::config::MAX_SYSCALL_NUM {
                get_syscall_count(id) as isize
            } else {
                -1
            }
        }
        _ => -1,
    }
}
