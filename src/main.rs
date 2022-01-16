extern crate calloop;
extern crate chrono;
extern crate mockall;

use calloop::{timer::Timer, EventLoop};
use calloop::{channel::Channel, channel::Sender, channel::Event, channel::channel};
use chrono::prelude::*;
use mockall::predicate::*;
use mockall::*;
use std::io::*;
use std::env;
use std::thread;
use getch::{Getch};

struct AlarmClock {
    display: Box<dyn Display>,
    alarm: Option<DateTime<Local>>,
    signal: Option<calloop::LoopSignal>
}

impl AlarmClock {
    fn new(display: Box<dyn Display>, signal: Option<calloop::LoopSignal>) -> Self {
        Self { display: display, alarm: None, signal}
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

    fn set_alarm(&mut self, alarm: Option<DateTime<Local>>) {
        self.alarm = alarm;
    }

    fn disable_alarm(&mut self) {
        self.alarm = None;
        self.display.clear();
    }

    fn stop(&mut self) {
        if let Some(signal) = &self.signal {
            signal.stop();
        }
    }
}

#[automock]
trait Display {
    fn print(&self, now: &str);
    fn clear(&self);
}

struct Console {}

impl Display for Console {
    fn print(&self, now: &str) {
        print!("\r{}", now);
        std::io::stdout().flush().unwrap();
    }

    fn clear(&self) {
        print!("\r                        \r");
        std::io::stdout().flush().unwrap();
    }
}

#[cfg(test)]
mod test {
    use chrono::Local;
    use mockall::predicate::str::ends_with;
    use crate::AlarmClock;
    use crate::MockDisplay;

    use std::sync::mpsc::channel;
    use std::thread;
    
    #[test]
    fn test_alarm_clock() {
        let mut printer = MockDisplay::new();
        printer.expect_print().times(1).returning(|_| ());
        let mut alarm_clock = AlarmClock::new(Box::new(printer), None);

        alarm_clock.tick(Local::now());
    }

    #[test]
    fn test_single_alarm() {
        let mut printer = MockDisplay::new();
        let now = Local::now();

        printer.expect_print().with(ends_with(" ALARM")).times(1).returning(|_| ());

        let mut alarm_clock = AlarmClock::new(Box::new(printer), None);
        alarm_clock.set_alarm(Some(now.clone()));
        alarm_clock.tick(now);
    }

    fn expensive_computation() -> u32 {
        42
    }

    #[test]
    fn test_channel() {
        let (sender, receiver) = channel();

        thread::spawn(move|| {
            sender.send(expensive_computation()).unwrap();
        });
        
        println!("{:?}", receiver.recv().unwrap());        
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let alarm = if args.len() > 1 {
        let time: NaiveTime = NaiveTime::parse_from_str(&args[1], "%H:%M:%S").unwrap();
        Local::today().and_time(time)
    } else { None };

    let mut event_loop: EventLoop<AlarmClock> =
        EventLoop::try_new().expect("Failed to initialize the event loop!");

    let handle = event_loop.handle();
    let source = Timer::new().expect("Failed to create timer event source!");

    let printer: Console = Console {};
    let mut alarm_clock = AlarmClock::new(Box::new(printer), Some(event_loop.get_signal()));

    alarm_clock.set_alarm(alarm);

    let timer_handle = source.handle();
    timer_handle.add_timeout(std::time::Duration::from_millis(500), 1);

    handle
        .insert_source(source, |_event, timer_handle, alarm_clock| {
            alarm_clock.tick(Local::now());
            timer_handle.add_timeout(std::time::Duration::from_millis(500), 1);
        })
        .expect("Failed to insert event source!");

    let (sender, receiver) : (Sender<u8>, Channel<u8>) = channel();

    let join_handle = thread::spawn(move|| {
        let getch = Getch::new();
        loop {
            let key = getch.getch();
            if let Ok(code) = key {
                sender.send(code).unwrap();
                if 113 == code
                {
                    return;
                }
            }
        }        
    });

    handle.insert_source(receiver, |event, _source_handle, alarm_clock| {
        match event {
            Event::Msg(10) => {
                alarm_clock.disable_alarm();
            },
            Event::Msg(113) => {
                alarm_clock.stop();
            },
            _ => ()
        }
    }).expect("Failed to insert event source!");

    event_loop
        .run(
            std::time::Duration::from_millis(1000),
            &mut alarm_clock,
            |_alarm_clock| {
            },
        )
        .expect("Error during event loop!");

    join_handle.join().unwrap();
}
