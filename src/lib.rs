extern crate libc;
extern crate heartbeats_sys;

pub mod poet;

/*
use heartbeats::energy_reader::EnergyReader;
use heartbeats::heartbeat::Heartbeat;
use poet::POET;
use poet::POETStates;

fn main() {
    let mut er = EnergyReader::new().ok().expect("Failed to get energy reader");
    let mut hb = Heartbeat::new(None, 20, 20, "heartbeat.log", &mut er).ok().expect("Failed to get heartbeat");
    let mut poet_states = POETStates::new().ok().expect("Failed to load POET states");
    let mut poet = POET::new(&mut hb, 100.0, &mut poet_states, None, None, 20u32, "poet.log").ok().expect("Failed to initialize POET");
    for i in 0..100 {
        hb.heartbeat(i, 1, 0.0, None);
        poet.apply_control();
        std::thread::sleep_ms(100);
    }
    println!("Hello, world!");
    // using Drop trait automatically cleans up
}
*/
