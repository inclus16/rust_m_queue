use crate::command::RawCommand;
use anyhow::Error;
use nix::mqueue::mq_attr_member_t;
use nix::mqueue::{mq_close, mq_open, mq_receive, mq_send, MQ_OFlag, MqAttr, MqdT};
use nix::sys::stat::Mode;

pub struct IpcReceiver<const MESSAGE_SIZE: i64> {
    descriptor: MqdT,
    buffer: [u8; MESSAGE_SIZE as usize],
}

impl<const MESSAGE_SIZE: i64> IpcReceiver<MESSAGE_SIZE> {
    pub fn init(name: &str, capacity: u8) -> Result<Self, Error>
    {
        let flags = MQ_OFlag::O_CREAT | MQ_OFlag::O_RDONLY;
        let mode = Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH;
        let attributes = MqAttr::new(0, capacity, MESSAGE_SIZE, 0);
        let mqd0 = mq_open(name, flags, mode, Some(&attributes))?;
        Ok(Self {
            descriptor: mqd0,
            buffer: [0; MESSAGE_SIZE as usize],
        })
    }

    pub fn receive(&mut self) -> Result<RawCommand, Error>
    {
        let mut prio = 0u32;
        let len = mq_receive(&self.descriptor, &mut self.buffer, &mut prio)?;
        Ok(bincode::deserialize::<RawCommand>(&self.buffer[0..len])?)
    }
}