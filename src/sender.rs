use std::sync::mpsc::Sender;
use anyhow::Error;
use nix::mqueue::{mq_open, mq_send, MQ_OFlag, MqdT};
use nix::sys::stat::Mode;
use serde::Serialize;
use nix::mqueue::MqAttr;
use nix::mqueue::mq_close;
use std::os::fd::FromRawFd;
use std::os::fd::AsRawFd;


/// Sender.
/// Connects to previously created queue (via Receiver) and send messages to it.
pub struct IpcSender<const MESSAGE_SIZE: usize> {
    descriptor:  MqdT,
}

impl<const MESSAGE_SIZE: usize> IpcSender<MESSAGE_SIZE> {
    /// Connects to previously created queue.
    /// # Example
    /// ```
    ///use rust_m_queue::sender::IpcSender;
    ///
    ///const MESSAGE_SIZE: usize = 1024;
    ///const QUEUE_NAME: &str = "/test_queue";
    ///let sender = IpcSender::<MESSAGE_SIZE>::connect_to_queue(QUEUE_NAME)?;
    ///
    /// ```
    pub fn connect_to_queue(name: &str) -> Result<Self, Error> {
        let flags =  MQ_OFlag::O_RDWR;
        let mode = Mode::S_IWUSR;
        let mqd0 = mq_open(name, flags, mode, None)?;
        Ok(Self { descriptor: mqd0 })
    }

    /// Sends message to queue with priority.
    /// Command must implement Serialize to automatically serialize via bincode.
    /// # Example
    /// ```
    /// use serde::Serialize;
    /// use rust_m_queue::sender::IpcSender;
    ///
    /// #[derive(Serialize)]
    /// struct Message {
    ///     pub data: String,
    /// }
    /// const MESSAGE_SIZE: usize = 1024;
    /// const QUEUE_NAME: &str = "/test_queue";
    /// let sender = IpcSender::<MESSAGE_SIZE>::connect_to_queue(QUEUE_NAME)?;
    /// let message = Message {
    ///     data: String::from("test")
    /// };
    /// let priority = 3;
    /// sender.send(message, priority)?;
    /// ```
    pub fn send(&self, command: impl Serialize, priority: u32) -> Result<(), Error> {
        let msg = bincode::serialize(&command)?;
        mq_send(&self.descriptor, &msg, priority)?;
        Ok(())
    }
}


impl<const MESSAGE_SIZE: usize> Drop for IpcSender<MESSAGE_SIZE> {
    fn drop(&mut self) {
        let descriptor:MqdT;
        unsafe {
            descriptor = MqdT::from_raw_fd(self.descriptor.as_raw_fd());
        }
        mq_close(descriptor);
    }
}

#[cfg(test)]
mod tests {
    use crate::sender::IpcSender;
    use nix::mqueue::{mq_close, mq_open, mq_receive, MQ_OFlag, MqAttr, MqdT,mq_unlink};
    use nix::sys::stat::Mode;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    struct Message {
        pub data: String,
    }
    const MESSAGE_SIZE: usize = 1024;

    const QUEUE_NAME: &str = "/test_queue_sen";

    fn create_queue() -> MqdT
    {
        let flags = MQ_OFlag::O_CREAT | MQ_OFlag::O_RDONLY;
        let mode = Mode::S_IWUSR | Mode::S_IRUSR;
        let attributes = MqAttr::new(0, 10, MESSAGE_SIZE as i64, 0);
        mq_open(QUEUE_NAME, flags, mode, Some(&attributes)).unwrap()
    }

    fn receive_message(descriptor: &MqdT, buffer: &mut [u8; MESSAGE_SIZE], prior: &mut u32)
    {
        let _len = mq_receive(descriptor, buffer, prior).unwrap();
    }
    #[test]
    fn test_success_connect() {
        let mq = create_queue();
        let sender = IpcSender::<MESSAGE_SIZE>::connect_to_queue(QUEUE_NAME);
        assert_eq!(sender.is_ok(), true);
        let _ = mq_close(mq);
        mq_unlink(QUEUE_NAME);
    }

    #[test]
    fn test_send_success()
    {
        let mq = create_queue();
        let sender = IpcSender::<MESSAGE_SIZE>::connect_to_queue(QUEUE_NAME).unwrap();
        let message = Message {
            data: String::from("test")
        };
        sender.send(message.clone(), 3).unwrap();
        let mut buffer = [0; MESSAGE_SIZE];
        let mut priority = 0u32;
        receive_message(&mq, &mut buffer, &mut priority);
        let _ = mq_close(mq);
        mq_unlink(QUEUE_NAME);
        assert_eq!(priority, 3);
        assert_eq!(bincode::deserialize::<Message>(buffer.as_ref()).unwrap(), message);
    }
}
