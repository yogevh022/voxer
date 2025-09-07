use crate::world::network::{ServerMessage, process_message};
use crossbeam::channel;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use crossbeam::channel::TryIter;
use parking_lot::Mutex;
use voxer_network::UdpChannel;

pub struct NetworkHandle<const BUFFER: usize> {
    pub channel: Arc<Mutex<UdpChannel<BUFFER>>>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    send_handle: Option<channel::Sender<ServerMessage>>,
    recv_handle: channel::Receiver<ServerMessage>,
}

impl<const BUFFER: usize> NetworkHandle<BUFFER> {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Self {
        let net_channel = UdpChannel::<BUFFER>::bind(addr);
        let (send_handle, recv_handle) = channel::unbounded::<ServerMessage>();
        Self {
            channel: Arc::new(Mutex::new(net_channel)),
            thread_handle: None,
            send_handle: Some(send_handle),
            recv_handle,
        }
    }

    pub fn try_iter_messages(&self) -> TryIter<'_, ServerMessage> {
        self.recv_handle.try_iter()
    }

    pub fn listen(&mut self) {
        let net = self.channel.clone();
        let send_handle = self.send_handle.take().unwrap();
        self.thread_handle = Some(std::thread::spawn(move || {
            NetworkHandle::<BUFFER>::recv_loop(net, send_handle);
        }));
    }

    fn recv_loop(net: Arc<Mutex<UdpChannel<BUFFER>>>, send_channel: channel::Sender<ServerMessage>) {
        loop {
            if let Some(msg) = net.lock().recv_single() {
                let server_msg = process_message(msg);
                send_channel.send(server_msg).unwrap()
            } else {
                // todo sleep?
            }
        }
    }
}
