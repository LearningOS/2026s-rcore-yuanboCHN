//! Process management syscalls
use crate::config::{PAGE_SIZE, TRAP_CONTEXT_BASE};
use crate::mm::translated_byte_buffer;
use crate::task::{
    change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_count, mmap,
    munmap, read_byte_from_user, suspend_current_and_run_next, write_byte_to_user,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// SV39 address width mask (low 39 bits determine the physical translation).
const VA_MASK_SV39: usize = (1usize << 39) - 1;

/// Return `true` for addresses that are *not* user-accessible data: the
/// trampoline and trap-context pages reside at the top of the address space, and
/// because SV39 ignores the upper 25 bits, their aliases (e.g. `isize::MAX`)
/// must also be rejected here.
fn is_reserved_addr(addr: usize) -> bool {
    let low39 = addr & VA_MASK_SV39;
    let trap_context_low39 = TRAP_CONTEXT_BASE & VA_MASK_SV39;
    low39 >= trap_context_low39
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

/// get time with second and microsecond
///
/// We translate the user pointer through the page table via
/// [`translated_byte_buffer`] so that a [`TimeVal`] crossing a page boundary is
/// still written correctly.
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let token = current_user_token();
    let buffers = translated_byte_buffer(token, ts as *const u8, core::mem::size_of::<TimeVal>());
    let mut value = [0u8; core::mem::size_of::<TimeVal>()];
    value[0..8].copy_from_slice(&(us / 1_000_000).to_le_bytes());
    value[8..16].copy_from_slice(&(us % 1_000_000).to_le_bytes());
    let mut pos = 0;
    for buf in buffers {
        for b in buf {
            *b = value[pos];
            pos += 1;
        }
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

/// The `sys_trace` syscall.
///
/// - `Read`: return the byte at address `id`, or -1 if not readable.
/// - `Write`: write `data` to address `id`, return 0 or -1.
/// - `Syscall`: return how many times `id` was invoked by the current task.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        r if r == TraceRequest::Read as usize => {
            if is_reserved_addr(id) {
                return -1;
            }
            match read_byte_from_user(id) {
                Some(v) => v as isize,
                None => -1,
            }
        }
        r if r == TraceRequest::Write as usize => {
            if is_reserved_addr(id) {
                return -1;
            }
            if write_byte_to_user(id, data as u8) {
                0
            } else {
                -1
            }
        }
        r if r == TraceRequest::Syscall as usize => get_syscall_count(id) as isize,
        _ => -1,
    }
}

/// mmap: map `[start, start + len)` with permission `port` (1=R, 2=W, 3=RW).
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap start={:#x} len={:#x} port={}", start, len, port);
    if start % PAGE_SIZE != 0 || port == 0 || (port & !3) != 0 {
        return -1;
    }
    mmap(start, len, port)
}

/// munmap: unmap `[start, start + len)`.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap start={:#x} len={:#x}", start, len);
    if start % PAGE_SIZE != 0 || len % PAGE_SIZE != 0 {
        return -1;
    }
    munmap(start, len)
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
