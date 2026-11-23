//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};
use crate::mm::{translated_byte_buffer,PageTable,VirtAddr,MapPermission};
use crate::task::current_user_token;
use crate::timer::get_time_us;
use core::mem::size_of;
use crate::task::get_syscall_count;
use crate::config::PAGE_SIZE;
use crate::task::{current_mmap, current_munmap};

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
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us=get_time_us();
    let sec=us/1000000;
    let usec=us%1000000;

    let mut source=sec.to_ne_bytes().into_iter().chain(usec.to_ne_bytes());
    let buffers=translated_byte_buffer(current_user_token(),ts.cast::<u8>(),size_of::<TimeVal>());
    for dst in buffers{
        for byte in dst{
            *byte=source.next().unwrap();
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request{
        0 | 1=>{
            let token=current_user_token();
            let pagetable=PageTable::from_token(token);
            //let buffers=translated_byte_buffer(token,_id as *const u8,1);
            let va=VirtAddr::from(_id);
            let pte=pagetable.translate(va.floor()).unwrap();
            //let flags=pte.flags();
            if !pte.is_valid() || !pte.visitable() {
                return -1;
            }
            let mut buffers=translated_byte_buffer(token,_id as *const u8,1);
            if _trace_request==0{
                if !pte.readable(){
                    return -1;
                }
                buffers[0][0] as isize
            }else{
                if !pte.writable(){
                    return -1;
                }
                buffers[0][0]=_data as u8;
                0
            }
            
        }
        2=>get_syscall_count(_id),
        _=>-1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start%PAGE_SIZE != 0{
        return -1;
    }
    if _port &!0x7 != 0{
        return -1;
    }
    if _port & 0x7 == 0{
        return -1;
    }
    if _len==0{
        return 0;
    }
    let mut permission = MapPermission::U;
    if _port&1 != 0{
        permission |= MapPermission::R;
    }
    if _port&2 != 0{
        permission |= MapPermission::W;
    }
    if _port&4 !=0{
        permission |= MapPermission::X;
    }
    let start_va=VirtAddr::from(_start);
    let end_va=VirtAddr::from(_start+_len); 
    current_mmap(start_va,end_va,permission)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start%PAGE_SIZE!=0{
        return -1;
    }
    if _len==0{
        return 0;
    }
    let start_va=VirtAddr(_start);
    let end_va=VirtAddr(_start+_len);
    current_munmap(start_va,end_va)
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
