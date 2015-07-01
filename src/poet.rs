//! POET - the Performance with Optimal Energy Toolkit. Used to meet timing constraints while
//! minimizing energy consumption.

use libc::{c_void, c_char, c_int, c_uint, c_double};
use std::ffi::CString;
use std::ptr;

/// Typedef for an "apply" function - used to manipulate system or application settings.
pub type POETApplyFn = unsafe extern fn(states: *mut c_void,
                                        num_states: c_uint,
                                        id: c_uint,
                                        last_id: c_uint);

/// Typedef for "current state" function - used to try and determine the current system or
/// application state.
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
    
    fn get_control_states(path: *const c_char,
                          states: *mut *mut POETControlState,
                          num_states: *mut c_uint) -> c_int;

    fn get_cpu_states(path: *const c_char,
                      states: *mut *mut POETCpuState,
                      num_states: *mut c_uint) -> c_int;

    fn get_current_cpu_state(states: *const c_void,
                             num_states: c_uint,
                             curr_state_id: *mut c_uint) -> c_int;

}

#[repr(C)]
/// Representation of native struct `poet_control_state_t`.
pub struct POETControlState {
    pub id: c_uint,
    pub speedup: c_double,
    pub cost: c_double,
}

impl POETControlState {
    /// Attempt to load control states from a file.
    pub fn new(filename: Option<&str>) -> Result<(*mut POETControlState, u32), String> {
        let filename = match filename {
            Some(f) => CString::new(f).unwrap().as_ptr(),
            None => ptr::null(),
        };
        let mut control_states = ptr::null_mut();
        let mut num_ctl_states : u32 = 0;
        let res = unsafe {
            get_control_states(filename,
                               &mut control_states,
                               &mut num_ctl_states)
        };
        if res != 0 {
            return Err("Failed to load control states".to_string());
        }
        Ok((control_states, num_ctl_states))
    }
}

impl Default for POETControlState {
    fn default() -> POETControlState {
        POETControlState {
            id: 0,
            speedup: 1.0,
            cost: 1.0,
        }
    }
}

#[repr(C)]
/// Representation of native struct `poet_cpu_state_t`.
pub struct POETCpuState {
    id: c_uint,
    freq: c_uint,
    cores: c_uint,
}

impl POETCpuState {
    /// Attempt to load cpu states from a file.
    pub fn new(filename: Option<&str>) -> Result<(*mut POETCpuState, u32), String> {
        let filename = match filename {
            Some(f) => CString::new(f).unwrap().as_ptr(),
            None => ptr::null(),
        };
        let mut cpu_states = ptr::null_mut();
        let mut num_cpu_states : u32 = 0;
        let res = unsafe {
            get_cpu_states(filename,
                           &mut cpu_states,
                           &mut num_cpu_states)
        };
        if res != 0 {
            return Err("Failed to load cpu states".to_string());
        }
        Ok((cpu_states, num_cpu_states))
    }
}

impl Default for POETCpuState {
    fn default() -> POETCpuState {
        POETCpuState {
            id: 0,
            freq: 0,
            cores: 0,
        }
    }
}

/// The `POET` struct wraps an underyling C struct.
pub struct POET {
    /// The underlying C struct `poet_state`.
    pub poet: *mut c_void,
}

impl POET {
    /// Attempt to initialize POET and allocate resources in the underlying C struct.
    pub fn new(perf_goal: f64,
               control_states: *mut POETControlState,
               cpu_states: *mut POETCpuState,
               num_states: u32,
               apply_func: Option<POETApplyFn>,
               curr_state_func: Option<POETCurrentStateFn>,
               period: u32,
               buffer_depth: u32,
               log_filename: Option<&str>) -> Result<POET, String> {
        let apply_func = match apply_func {
            Some(p) => p,
            None => apply_cpu_config,
        };
        let curr_state_func = match curr_state_func {
            Some(p) => p,
            None => get_current_cpu_state,
        };
        let log_filename = match log_filename {
            Some(l) => CString::new(l).unwrap().as_ptr(),
            None => ptr::null(),
        };
        let poet = unsafe {
            poet_init(perf_goal,
                      num_states, control_states, cpu_states,
                      apply_func, curr_state_func,
                      period, buffer_depth, log_filename)
        };
        if poet.is_null() {
            return Err("Failed to instantiate POET object".to_string());
        }
        Ok(POET { poet: poet, })
    }

    /// Call at every iteration - at specified periods this function will (potentially) order
    /// changes to system or application state to try and meet timing constraints.
    pub fn apply_control(&mut self, tag: u64, window_rate: f64, window_power: f64) {
        unsafe {
            poet_apply_control(self.poet, tag, window_rate, window_power);
        }
    }
}

impl Drop for POET {
    /// Cleanup POET and deallocate resources in the underlying C struct.
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
    use libc::{c_void, c_uint};

    #[test]
    fn test_basic() {
        let mut control_states = vec![POETControlState::default()];
        let mut cpu_states = vec![POETCpuState::default()];
        let mut poet = POET::new(100.0,
                                 control_states.as_mut_ptr(), cpu_states.as_mut_ptr(), 1,
                                 None, None,
                                 20u32, 1u32, None).unwrap();
        poet.apply_control(0, 1.0, 1.0);
    }

    #[test]
    fn test_rust_callbacks() {
        let mut control_states = vec![POETControlState::default()];
        let mut cpu_states = vec![POETCpuState::default()];
        let mut poet = POET::new(100.0,
                                 control_states.as_mut_ptr(), cpu_states.as_mut_ptr(), 1,
                                 Some(dummy_apply), Some(dummy_curr_state),
                                 20u32, 1u32, None).unwrap();
        for i in 0..50 {
            poet.apply_control(i, 1.0, 1.0);
        }
    }

    unsafe extern fn dummy_apply(_states: *mut c_void,
                                 _num_states: c_uint,
                                 _id: c_uint,
                                 _last_id: c_uint) {
        // do nothing
        println!("Received apply call");
    }

    unsafe extern fn dummy_curr_state (_states: *const c_void,
                                       _num_states: c_uint,
                                       _curr_state_id: *mut c_uint) -> i32 {
        println!("Received curr state call");
        // this is actually an invalid value, but forces the apply function to be called
        *_curr_state_id = _num_states;
        0
    }

}
