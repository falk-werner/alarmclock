extern crate chrono;
extern crate calloop;
extern crate mockall;

use chrono::prelude::*;
use calloop::{timer::Timer, EventLoop, LoopSignal};
use mockall::*;
use mockall::predicate::*;
use std::io::*;

struct AlarmClock {
    display: Box<dyn Display>,    
}

impl AlarmClock {
    fn new(display: Box<dyn Display>) -> Self {
        Self { display: display }
    }

    fn tick(&mut self) {
       let now: DateTime<Local> = Local::now();
       let now = format!("{}",now.format("%H:%M:%S"));
       self.display.print(&now);
    }
    
}

#[automock]
trait Display {
    fn print(&self, now: &str);
}

struct Console {
    
}

impl Display for Console {
    fn print(&self, now: &str) {
        print!("\r{}", now);
        std::io::stdout().flush().unwrap();
    }
    
}

#[cfg(test)]
mod test {
    use crate::MockDisplay;
    use crate::AlarmClock;

#[test]
fn test_alarm_clock() {    
    let mut printer = MockDisplay::new();
    printer.expect_print().times(1).returning(|_| () );
    let mut clock = AlarmClock::new(Box::new(printer));
    clock.tick();
}

}

fn main() {
    let mut event_loop: EventLoop<LoopSignal> =
    EventLoop::try_new().expect("Failed to initialize the event loop!");

    let handle = event_loop.handle();
    let source = Timer::new().expect("Failed to create timer event source!");
    
    let printer : Console = Console { };
    let alarm_clock = AlarmClock::new(Box::new(printer));

    let timer_handle = source.handle();
    timer_handle.add_timeout(std::time::Duration::from_millis(500), Box::new(alarm_clock));

    handle
        .insert_source(
            source,
            |mut event, metadata, _shared_data| {            
            event.tick();
            metadata.add_timeout(std::time::Duration::from_millis(500), event);  
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
