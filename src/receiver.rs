use anyhow::Error;
use nix::mqueue::{mq_open, mq_receive, MQ_OFlag, MqAttr, MqdT};
use nix::sys::stat::Mode;
use serde::Deserialize;

/// Receiver.
/// Creates queue and listen for incoming messages.
pub struct IpcReceiver<const MESSAGE_SIZE: usize> {
    descriptor: MqdT,
    buffer: [u8; MESSAGE_SIZE],
}

impl<const MESSAGE_SIZE: usize> IpcReceiver<MESSAGE_SIZE> {
    /// Creates new queue with provided name and capacity.
    /// # Example
    /// ```
    ///use rust_m_queue::receiver::IpcReceiver;
    ///
    ///const MESSAGE_SIZE: usize = 1024;
    ///const QUEUE_NAME: &str = "/test_queue";
    ///let mut receiver = IpcReceiver::<MESSAGE_SIZE>::init(QUEUE_NAME, 10)?;
    /// ```
    pub fn init(name: &str, capacity: i64) -> Result<Self, Error>
    {
        let flags = MQ_OFlag::O_CREAT | MQ_OFlag::O_RDONLY;
        let mode = Mode::S_IRUSR | Mode::S_IWUSR;
        let attributes = MqAttr::new(0, capacity, MESSAGE_SIZE as i64, 0);
        let mqd0 = mq_open(name, flags, mode, Some(&attributes))?;
        Ok(Self {
            descriptor: mqd0,
            buffer: [0; MESSAGE_SIZE],
        })
    }

    /// Thread blocking receive first message from queue.
    /// It will deserialize via bincode automatically data into provided generic type.
    /// # Example
    /// ```
    /// use serde::{Deserialize};
    /// use rust_m_queue::receiver::IpcReceiver;
    ///
    /// #[derive(Deserialize)]
    /// struct Message {
    ///    pub data: String,
    /// }
    /// const MESSAGE_SIZE: usize = 1024;
    ///const QUEUE_NAME: &str = "/test_queue";
    ///let mut receiver = IpcReceiver::<MESSAGE_SIZE>::init(QUEUE_NAME, 10)?;
    ///loop{
    ///    let data = receiver.receive::<Message>()?; //thread blocking
    /// }
    pub fn receive<'a, T>(&'a mut self) -> Result<T, Error>
    where
        T: Deserialize<'a>,
    {
        let mut prio = 0u32;
        let len = mq_receive(&self.descriptor, &mut self.buffer, &mut prio)?;
        Ok(bincode::deserialize::<T>(&self.buffer[0..len])?)
    }
}

#[cfg(test)]
mod tests {
    use crate::receiver::IpcReceiver;
    use nix::mqueue::{mq_close, mq_open, mq_send, MQ_OFlag, MqAttr, MqdT};
    use nix::sys::stat::Mode;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    struct Message {
        pub data: String,
    }
    const MESSAGE_SIZE: usize = 1024;

    const QUEUE_NAME: &str = "/test_queue_rec";

    fn create_queue() -> MqdT
    {
        let flags = MQ_OFlag::O_CREAT | MQ_OFlag::O_RDONLY |  MQ_OFlag::O_WRONLY;
        let mode = Mode::S_IWUSR | Mode::S_IRUSR;
        let attributes = MqAttr::new(0, 10, MESSAGE_SIZE as i64, 0);
        mq_open(QUEUE_NAME, flags, mode, Some(&attributes)).unwrap()
    }
    fn send_message(descriptor: &MqdT, message: &[u8])
    {
        mq_send(descriptor, message, 0).unwrap();
    }

    #[test]
    fn test_success_create() {
        let receiver = IpcReceiver::<MESSAGE_SIZE>::init(QUEUE_NAME, 10);
        assert_eq!(receiver.is_ok(), true);
    }
    #[test]
    fn test_receive() {
        let message_string = String::from("test2");
        let message = Message {
            data: message_string
        };
        let mq = create_queue();
        send_message(&mq, bincode::serialize(&message).unwrap().as_ref());
        let _ = mq_close(mq);
        let mut receiver = IpcReceiver::<MESSAGE_SIZE>::init(QUEUE_NAME, 10).unwrap();
        let data = receiver.receive::<Message>().unwrap();
        assert_eq!(data, message);
    }
}