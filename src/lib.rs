#![allow(non_camel_case_types)]

extern crate libc;

use libc::{c_void, c_char, c_int, c_uint, c_double};

/// Typedef for an "apply" function - used to manipulate system or application settings.
pub type poet_apply_func = extern fn(states: *mut c_void,
	                                 num_states: c_uint,
	                                 id: c_uint,
	                                 last_id: c_uint);

/// Typedef for "current state" function - used to try and determine the current system or
/// application state.
pub type poet_curr_state_func = extern fn(states: *const c_void,
                                          num_states: c_uint,
                                          curr_state_id: *mut c_uint) -> i32;

#[repr(C)]
/// Representation of native struct `poet_control_state_t`.
pub struct poet_control_state_t {
    pub id: c_uint,
    pub speedup: c_double,
    pub cost: c_double,
}

#[repr(C)]
/// Representation of native struct `poet_cpu_state_t`.
pub struct poet_cpu_state_t {
    pub id: c_uint,
    pub freq: c_uint,
    pub cores: c_uint,
}

#[repr(C)]
pub struct poet_state;

extern {

    // Core functions - poet.h

    pub fn poet_init(perf_goal: c_double,
	                 num_system_states: c_uint,
	                 control_states: *mut poet_control_state_t,
	                 apply_states: *mut poet_cpu_state_t,
	                 apply_func: poet_apply_func,
	                 curr_state_func: poet_curr_state_func,
	                 period: c_uint,
	                 buffer_depth: c_uint,
	                 log_filename: *const c_char) -> *mut poet_state;

    pub fn poet_apply_control(state: *mut poet_state,
	                          id: u64,
	                          perf: c_double,
	                          pwr: c_double);

    pub fn poet_destroy(state: *mut poet_state);

    // Configuration/utility functions - poet_config.h

    pub fn apply_cpu_config(states: *mut c_void,
	                        num_states: c_uint,
	                        id: c_uint,
	                        last_id: c_uint);

    pub fn get_control_states(path: *const c_char,
	                          states: *mut *mut poet_control_state_t,
	                          num_states: *mut c_uint) -> c_int;

    pub fn get_cpu_states(path: *const c_char,
	                      states: *mut *mut poet_cpu_state_t,
	                      num_states: *mut c_uint) -> c_int;

    pub fn get_current_cpu_state(states: *const c_void,
	                             num_states: c_uint,
	                             curr_state_id: *mut c_uint) -> c_int;

}
