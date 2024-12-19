This library is a simple OOP-like wrapper around nix posix m_queue to communicate between processes through queue in
unix environment.

It contains two structs: IpcReceiver and IpcSender.

Basic usage of sender:

```rust    
    #[derive(Serialize)]
struct Message {
    pub data: String,
}
const MESSAGE_SIZE: usize = 1024;
const QUEUE_NAME: &str = "/test_queue";
let sender = IpcSender::<MESSAGE_SIZE>::connect_to_queue(QUEUE_NAME).unwrap();
let message = Message {
data: String::from("test")
};
let priority = 3;
sender.send(message.clone(), priority).unwrap();
```

And receiver:

```rust    
    #[derive(Serialize)]
struct Message {
    pub data: String,
}
const MESSAGE_SIZE: usize = 1024;
const QUEUE_NAME: &str = "/test_queue";
let mut receiver = IpcReceiver::<MESSAGE_SIZE>::init(QUEUE_NAME, 10).unwrap();
loop{
let data = receiver.receive::< Message >().unwrap(); //thread blocking
}
```
    
