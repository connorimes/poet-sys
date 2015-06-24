use libc::{self, c_void, c_char, c_int, c_uint, c_double};
use std::ffi::CString;
use std::ptr;
use heartbeats_sys::heartbeat::Heartbeat;

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

    fn poet_init(heart: *mut c_void,
                 perf_goal: c_double,
                 num_system_states: c_uint,
                 control_states: *mut c_void,
                 apply_states: *mut c_void,
                 apply_func: POETApplyFn,
                 curr_state_func: POETCurrentStateFn,
                 buffer_depth: c_uint,
                 log_filename: *const c_char) -> *mut c_void;

    fn poet_apply_control(state: *mut c_void);

    fn poet_destroy(state: *mut c_void);

    /*
     * Configuration/utility functions - poet_config.h
     */

    fn apply_cpu_config(states: *mut c_void,
                        num_states: c_uint,
                        id: c_uint,
                        last_id: c_uint);
    
    fn get_control_states(path: *const c_char,
                          states: *mut *mut c_void,
                          num_states: *mut c_uint) -> c_int;

    fn get_cpu_states(path: *const c_char,
                      states: *mut *mut c_void,
                      num_states: *mut c_uint) -> c_int;

    fn get_current_cpu_state(states: *const c_void,
                             num_states: c_uint,
                             curr_state_id: *mut c_uint) -> c_int;

}

pub struct POETStates {
    pub control_states: *mut c_void,
    pub cpu_states: *mut c_void,
    pub num_states: u32,
}

impl POETStates  {
    pub fn new() -> Result<POETStates, String> {
        let mut control_states = ptr::null_mut();
        let mut cpu_states = ptr::null_mut();
        let mut num_ctl_states : u32 = 0;
        let mut num_cpu_states : u32 = 0;
        let res = unsafe {
            get_control_states(ptr::null(),
                               &mut control_states,
                               &mut num_ctl_states)
        };
        if res != 0 {
            return Err("Failed to load control states".to_string());
        }
        let res = unsafe {
            get_cpu_states(ptr::null(),
                           &mut cpu_states,
                           &mut num_cpu_states)
        };
        if res != 0 {
            unsafe {
                libc::free(control_states);
            }
            return Err("Failed to load cpu states".to_string());
        }
        if num_ctl_states != num_cpu_states {
            unsafe {
                libc::free(control_states);
                libc::free(cpu_states);
            }
            return Err("Number of control and cpu states don't match".to_string());
        }
        return Ok(POETStates {
                    control_states: control_states,
                    cpu_states: cpu_states,
                    num_states: num_ctl_states,
                });
    }
}

impl Drop for POETStates {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.control_states);
            libc::free(self.cpu_states);
        }
        println!("Cleaned up POET states");
    }
}

pub struct POET {
    pub poet: *mut c_void,
}

impl POET {
    pub fn new(hb: &mut Heartbeat,
               perf_goal: f64,
               poet_states: &mut POETStates,
               apply_func: Option<POETApplyFn>,
               curr_state_func: Option<POETCurrentStateFn>,
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
            poet_init(hb.hb, perf_goal,
                      poet_states.num_states, poet_states.control_states, poet_states.cpu_states,
                      apply_func, curr_state_func,
                      buffer_depth,
                      CString::new(log_filename).unwrap().as_ptr())
        };
        if poet.is_null() {
            return Err("Failed to instantiate POET object".to_string());
        }
        Ok(POET { poet: poet, })
    }

    pub fn apply_control(&mut self) {
        unsafe {
            poet_apply_control(self.poet);
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

