use crate::command::RawCommand;
use anyhow::Error;
use nix::mqueue::mq_attr_member_t;
use nix::mqueue::{mq_close, mq_open, mq_receive, mq_send, MQ_OFlag, MqAttr, MqdT};
use nix::sys::stat::Mode;

pub struct IpcSender<const MESSAGE_SIZE: usize> {
    descriptor: MqdT,
    sender_id: u8,
}

impl<const MESSAGE_SIZE: usize> IpcSender<MESSAGE_SIZE> {
    pub fn init(name: &str, sender_id: u8, capacity: i64) -> Result<Self, Error> {
        let flags = MQ_OFlag::O_WRONLY;
        let mode = Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH;
        let attributes = MqAttr::new(0, capacity, MESSAGE_SIZE as i64, 0);
        let mqd0 = mq_open(name, flags, mode, Some(&attributes))?;
        Ok(Self { descriptor: mqd0, sender_id })
    }

    pub fn send(&self, command: RawCommand, priority: u32) -> Result<(), Error> {
        let msg = bincode::serialize(&command)?;
        mq_send(&self.descriptor, &msg, priority)?;
        Ok(())
    }
}