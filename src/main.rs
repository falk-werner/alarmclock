extern crate calloop;
extern crate chrono;
extern crate mockall;

use calloop::{timer::Timer, EventLoop};
use chrono::prelude::*;
use mockall::predicate::*;
use mockall::*;
use std::io::*;

struct AlarmClock {
    display: Box<dyn Display>,
    alarm: Option<DateTime<Local>>,
}

impl AlarmClock {
    fn new(display: Box<dyn Display>) -> Self {
        Self { display: display, alarm: None}
    }

    fn tick(&mut self, now: DateTime<Local>) {
        let mut displayed_text = format!("{}", now.format("%H:%M:%S"));
        if let Some(x) = self.alarm  {
            if x <= now {
                displayed_text += " ALARM";
            }
        }

        self.display.print(&displayed_text);
    }

    fn set_alarm(&mut self, alarm: DateTime<Local>) {
        self.alarm = Some(alarm);
    }
}

#[automock]
trait Display {
    fn print(&self, now: &str);
}

struct Console {}

impl Display for Console {
    fn print(&self, now: &str) {
        print!("\r{}", now);
        std::io::stdout().flush().unwrap();
    }
}

#[cfg(test)]
mod test {
    use chrono::Local;
    use mockall::predicate::str::ends_with;
    use crate::AlarmClock;
    use crate::MockDisplay;
    use mockall::predicate::*;

    #[test]
    fn test_alarm_clock() {
        let mut printer = MockDisplay::new();
        printer.expect_print().times(1).returning(|_| ());
        let mut alarm_clock = AlarmClock::new(Box::new(printer));

        alarm_clock.tick(Local::now());
    }

    #[test]
    fn test_single_alarm() {
        let mut printer = MockDisplay::new();
        let now = Local::now();

        printer.expect_print().with(ends_with(" ALARM")).times(1).returning(|_| ());

        let mut alarm_clock = AlarmClock::new(Box::new(printer));
        alarm_clock.set_alarm(now.clone());
        alarm_clock.tick(now);
    }
}

fn main() {
    let mut event_loop: EventLoop<AlarmClock> =
        EventLoop::try_new().expect("Failed to initialize the event loop!");

    let handle = event_loop.handle();
    let source = Timer::new().expect("Failed to create timer event source!");

    let printer: Console = Console {};
    let mut alarm_clock = AlarmClock::new(Box::new(printer));

    let timer_handle = source.handle();
    timer_handle.add_timeout(std::time::Duration::from_millis(500), 1);

    handle
        .insert_source(source, |_event, timer_handle, alarm_clock| {
            alarm_clock.tick(Local::now());
            timer_handle.add_timeout(std::time::Duration::from_millis(500), 1);
        })
        .expect("Failed to insert event source!");

    event_loop
        .run(
            std::time::Duration::from_millis(1000),
            &mut alarm_clock,
            |_alarm_clock| {
            },
        )
        .expect("Error during event loop!");
}
