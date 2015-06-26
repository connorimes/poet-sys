use libc::{c_void, c_char, c_int, c_uint, c_double};
use std::ffi::CString;
use std::ptr;

pub type POETApplyFn = unsafe extern fn(states: *mut c_void,
                                        num_states: c_uint,
                                        id: c_uint,
                                        last_id: c_uint);

pub type POETCurrentStateFn = unsafe extern fn(states: *const c_void,
                                               num_states: c_uint,
                                               curr_state_id: *mut c_uint) -> i32;

#[link(name = "poet")]
extern {
    /*
     * Core functions - poet.h
     */

    fn poet_init(perf_goal: c_double,
                 num_system_states: c_uint,
                 control_states: *mut POETControlState,
                 apply_states: *mut POETCpuState,
                 apply_func: POETApplyFn,
                 curr_state_func: POETCurrentStateFn,
                 period: c_uint,
                 buffer_depth: c_uint,
                 log_filename: *const c_char) -> *mut c_void;

    fn poet_apply_control(state: *mut c_void,
                          id: u64,
                          perf: c_double,
                          pwr: c_double);

    fn poet_destroy(state: *mut c_void);

    /*
     * Configuration/utility functions - poet_config.h
     */

    fn apply_cpu_config(states: *mut c_void,
                        num_states: c_uint,
                        id: c_uint,
                        last_id: c_uint);
    
    pub fn get_control_states(path: *const c_char,
                              states: *mut *mut POETControlState,
                              num_states: *mut c_uint) -> c_int;

    pub fn get_cpu_states(path: *const c_char,
                          states: *mut *mut POETCpuState,
                          num_states: *mut c_uint) -> c_int;

    fn get_current_cpu_state(states: *const c_void,
                             num_states: c_uint,
                             curr_state_id: *mut c_uint) -> c_int;

}

#[repr(C)]
pub struct POETControlState {
    pub id: c_uint,
    pub speedup: c_double,
    pub cost: c_double,
}

impl POETControlState {
    pub fn new() -> Result<(*mut POETControlState, u32), String> {
        let mut control_states = ptr::null_mut();
        let mut num_ctl_states : u32 = 0;
        let res = unsafe {
            get_control_states(ptr::null(),
                               &mut control_states,
                               &mut num_ctl_states)
        };
        if res != 0 {
            return Err("Failed to load control states".to_string());
        }
        Ok((control_states, num_ctl_states))
    }
}

#[repr(C)]
pub struct POETCpuState {
    id: c_uint,
    freq: c_uint,
    cores: c_uint,
}

impl POETCpuState {
    pub fn new() -> Result<(*mut POETCpuState, u32), String> {
        let mut cpu_states = ptr::null_mut();
        let mut num_cpu_states : u32 = 0;
        let res = unsafe {
            get_cpu_states(ptr::null(),
                           &mut cpu_states,
                           &mut num_cpu_states)
        };
        if res != 0 {
            return Err("Failed to load cpu states".to_string());
        }
        Ok((cpu_states, num_cpu_states))
    }
}

pub struct POET {
    pub poet: *mut c_void,
}

impl POET {
    pub fn new(perf_goal: f64,
               control_states: *mut POETControlState,
               cpu_states: *mut POETCpuState,
               num_states: u32,
               apply_func: Option<POETApplyFn>,
               curr_state_func: Option<POETCurrentStateFn>,
               period: u32,
               buffer_depth: u32,
               log_filename: &str) -> Result<POET, String> {
        let apply_func = match apply_func {
            Some(p) => p,
            None => apply_cpu_config,
        };
        let curr_state_func = match curr_state_func {
            Some(p) => p,
            None => get_current_cpu_state,
        };
        let poet = unsafe {
            poet_init(perf_goal,
                      num_states, control_states, cpu_states,
                      apply_func, curr_state_func,
                      period, buffer_depth,
                      CString::new(log_filename).unwrap().as_ptr())
        };
        if poet.is_null() {
            return Err("Failed to instantiate POET object".to_string());
        }
        Ok(POET { poet: poet, })
    }

    pub fn apply_control(&mut self, tag: u64, window_rate: f64, window_power: f64) {
        unsafe {
            poet_apply_control(self.poet, tag, window_rate, window_power);
        }
    }
}

impl Drop for POET {
    fn drop(&mut self) {
        unsafe {
            poet_destroy(self.poet);
        }
        println!("Cleaned up POET");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use libc::{self, c_void};

    #[test]
    fn test_basic() {
        let (control_states, num_ctl_states): (*mut POETControlState, u32) = POETControlState::new().ok().expect("Failed to load control states");
        let (cpu_states, num_cpu_states): (*mut POETCpuState, u32) = POETCpuState::new().ok().expect("Failed to load cpu states");
        if num_ctl_states != num_cpu_states {
            panic!("Number of control and cpu states don't match");
        }
        let mut poet = POET::new(100.0,
                                 control_states, cpu_states, num_ctl_states,
                                 None, None,
                                 20u32, 1u32, "poet.log").ok().expect("Failed to initialize POET");
        poet.apply_control(0, 1.0, 1.0);
        unsafe {
            libc::free(control_states as *mut c_void);
            libc::free(cpu_states as *mut c_void);
        }
    }

}
