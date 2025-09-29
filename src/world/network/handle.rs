use crate::world::network::{ServerMessage, process_message};
use crossbeam::channel;
use crossbeam::channel::TryIter;
use std::net::ToSocketAddrs;
use voxer_network::{NetworkSerializable, NetworkingError, UdpChannel};

pub struct NetworkHandle {
    channel: UdpChannel,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    send_handle: Option<channel::Sender<ServerMessage>>,
    recv_handle: channel::Receiver<ServerMessage>,
}

impl NetworkHandle {
    pub fn bind<A: ToSocketAddrs>(addr: A, buffer_size: usize) -> Self {
        let channel = UdpChannel::bind(addr, buffer_size);
        let (send_handle, recv_handle) = channel::unbounded::<ServerMessage>();
        Self {
            channel,
            thread_handle: None,
            send_handle: Some(send_handle),
            recv_handle,
        }
    }

    pub fn try_iter_messages(&self) -> TryIter<'_, ServerMessage> {
        self.recv_handle.try_iter()
    }

    pub fn take_messages(&self, max: usize) -> Vec<ServerMessage> {
        self.recv_handle.try_iter().take(max).collect()
    }

    pub fn send_to(
        &self,
        data: Box<dyn NetworkSerializable>,
        addr: &impl ToSocketAddrs,
    ) -> Result<(), NetworkingError> {
        self.channel.send_to(data, addr)
    }

    pub fn listen(&mut self) {
        let net = self.channel.clone_handle();
        let send_handle = self.send_handle.take().unwrap();
        self.thread_handle = Some(std::thread::spawn(move || {
            NetworkHandle::recv_loop(net, send_handle);
        }));
    }

    fn recv_loop(mut net: UdpChannel, send_channel: channel::Sender<ServerMessage>) {
        loop {
            if let Some(msg) = net.recv_single() {
                let server_msg = process_message(msg);
                send_channel.send(server_msg).unwrap()
            }
        }
    }
}
