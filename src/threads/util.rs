use std::sync::mpsc::SyncSender;

pub fn try_sending<T>(sender: &SyncSender<T>, message: T, thread_name: &str, queue_name: &str) {
    if let Err(error) = sender.send(message) {
        eprintln!("Send error.{thread_name}, {queue_name}. {error}")
    }
}
