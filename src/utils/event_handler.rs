use crossterm::event::Event;

use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

pub enum TerminalEvent {
    Event(Event),
    Tick,
}

pub struct EventsHandler {
    receiver: mpsc::Receiver<TerminalEvent>,
    stop_capture: Arc<AtomicBool>,

    #[allow(unused)]
    thread_handle: thread::JoinHandle<()>,
}

impl EventsHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let stop_capture = Arc::new(AtomicBool::new(false));
        
        let thread_handle = {
            let sender = sender.clone();
            let event_stop_capture = stop_capture.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    // poll for tick rate duration, if no event, sent tick event.
                    if crossterm::event::poll(timeout).expect("event poll failed") {
                        let event = crossterm::event::read().expect("failed to read event");
                        sender.send(TerminalEvent::Event(event)).expect("failed to send event");
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(TerminalEvent::Tick).expect("failed to send tick event");
                        last_tick += tick_rate;
                    }

                    if event_stop_capture.load(Ordering::Relaxed) {
                        break;
                    }

                }
            })
        };
        Self {
            receiver,
            stop_capture,
            thread_handle,
        }
    }

    // Attempts to read an event
    pub fn next(&self) -> Result<TerminalEvent, mpsc::RecvError> {
        self.receiver.recv()
    }

    // Close
    pub fn close(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed)
    }

}