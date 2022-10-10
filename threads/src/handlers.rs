use crate::{AnEvent, Threads};

pub fn handle_pause(threads: &Threads) {
    let this = threads.this_thread();
    this.pause();
}

pub unsafe fn handle_event(e: AnEvent) {
    let AnEvent { event_handler, data } = e;
    event_handler(data);
}
