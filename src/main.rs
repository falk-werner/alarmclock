extern crate chrono;
extern crate calloop;

use chrono::prelude::*;
use calloop::{timer::Timer, EventLoop, LoopSignal};

fn main() {
    let mut event_loop: EventLoop<LoopSignal> =
    EventLoop::try_new().expect("Failed to initialize the event loop!");

    let handle = event_loop.handle();
    let source = Timer::new().expect("Failed to create timer event source!");
    
    let timer_handle = source.handle();
    timer_handle.add_timeout(std::time::Duration::from_millis(500), "Timeout reached!");

    handle
        .insert_source(
            source,
            |_event, metadata, _shared_data| {
            let now: DateTime<Local> = Local::now();
            println!("{}", now.format("%H:%M:%S"));
            metadata.add_timeout(std::time::Duration::from_millis(500), "Timeout reached!");  
            },
        )
        .expect("Failed to insert event source!");

    let mut shared_data = event_loop.get_signal();

    event_loop
        .run(
            std::time::Duration::from_millis(20),
            &mut shared_data,
            |_shared_data| { },
        )
        .expect("Error during event loop!");
}
