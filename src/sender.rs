use anyhow::Error;
use nix::mqueue::{mq_open, mq_send, MQ_OFlag, MqdT};
use nix::sys::stat::Mode;
use serde::Serialize;


pub struct IpcSender<const MESSAGE_SIZE: usize> {
    descriptor: MqdT,
}

impl<const MESSAGE_SIZE: usize> IpcSender<MESSAGE_SIZE> {
    pub fn connect_to_queue(name: &str) -> Result<Self, Error> {
        let flags = MQ_OFlag::O_WRONLY;
        let mode = Mode::S_IWUSR;
        let mqd0 = mq_open(name, flags, mode, None)?;
        Ok(Self { descriptor: mqd0 })
    }

    pub fn send(&self, command: impl Serialize, priority: u32) -> Result<(), Error> {
        let msg = bincode::serialize(&command)?;
        mq_send(&self.descriptor, &msg, priority)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::sender::IpcSender;
    use nix::mqueue::{mq_close, mq_open, mq_receive, MQ_OFlag, MqAttr, MqdT};
    use nix::sys::stat::Mode;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    struct Message {
        pub data: String,
    }
    const MESSAGE_SIZE: usize = 1024;

    const QUEUE_NAME: &str = "/test_queue";

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
        assert_eq!(priority, 3);
        assert_eq!(bincode::deserialize::<Message>(buffer.as_ref()).unwrap(), message);
        let _ = mq_close(mq);
    }
}
