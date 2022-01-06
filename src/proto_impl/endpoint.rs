use crate::{
    ffi::bindings::callbacks,
    proto,
    proto_impl::connection::{
        ConnectionEvent,
        ConnectionInner,
    },
};

use quinn_proto::Transmit;

use crate::proto_impl::QuinnErrorKind;
use std::{
    collections::HashMap,
    sync::{
        atomic::{
            AtomicU8,
            Ordering,
        },
        mpsc,
    },
};

/// Maximum number of datagrams processed in send/recv calls to make before moving on to other processing
///
/// This helps ensure we don't starve anything when the CPU is slower than the link.
/// Value is selected by picking a low number which didn't degrade throughput in benchmarks.
const IO_LOOP_BOUND: usize = 160;

/// An endpoint id that is increased for each created endpoint.
static ENDPOINT_ID: AtomicU8 = AtomicU8::new(0);

#[derive(Debug)]
pub enum EndpointEvent {
    Proto(proto::EndpointEvent),
    Transmit(proto::Transmit),
}

pub struct EndpointInner {
    pub(crate) inner: proto::Endpoint,
    connections: HashMap<proto::ConnectionHandle, mpsc::Sender<ConnectionEvent>>,
    endpoint_events_rx: mpsc::Receiver<(proto::ConnectionHandle, EndpointEvent)>,
    endpoint_events_tx: mpsc::Sender<(proto::ConnectionHandle, EndpointEvent)>,
    pub id: u8,
}

impl EndpointInner {
    pub fn new(endpoint: proto::Endpoint) -> Self {
        let (tx, rx) = mpsc::channel();

        let id = ENDPOINT_ID.load(Ordering::Relaxed).wrapping_add(1);

        EndpointInner {
            inner: endpoint,
            connections: HashMap::new(),
            endpoint_events_tx: tx,
            endpoint_events_rx: rx,
            id,
        }
    }

    pub fn poll(&mut self) {
        while let Some(transmit) = self.inner.poll_transmit() {
            self.notify_transmit(transmit);
        }

        // TODO limit max outgoing, invoke callback to poll again.

        self.handle_connection_events();
    }

    pub fn notify_transmit(&mut self, transmit: Transmit) {
        callbacks::on_transmit(self.id, transmit);
    }

    pub fn add_connection(
        &mut self,
        handle: proto::ConnectionHandle,
        connection: proto::Connection,
    ) -> ConnectionInner {
        let (send, recv) = mpsc::channel();
        let _ = self.connections.insert(handle, send);

        ConnectionInner::new(connection, handle, recv, self.endpoint_events_tx.clone())
    }

    pub fn forward_event_to_connection(
        &mut self,
        handle: proto::ConnectionHandle,
        event: proto::ConnectionEvent,
    ) -> Result<(), QuinnErrorKind> {
        self.connections
            .get_mut(&handle)
            .unwrap()
            .send(ConnectionEvent::Proto(event))?;

        Ok(())
    }

    pub fn handle_connection_events(&mut self) -> Result<bool, QuinnErrorKind> {
        for _ in 0..IO_LOOP_BOUND {
            match self.endpoint_events_rx.try_recv() {
                Ok((handle, event)) => {
                    match event {
                        EndpointEvent::Proto(proto) => {
                            if proto.is_drained() {
                                self.connections.remove(&handle);
                                if self.connections.is_empty() {
                                    //self.idle.notify_waiters();
                                }
                            }

                            if let Some(event) = self.inner.handle_event(handle, proto) {
                                // Ignoring errors from dropped connections that haven't yet been cleaned up
                                println!("endpoint proto event");
                                self.connections
                                    .get_mut(&handle)
                                    .unwrap()
                                    .send(ConnectionEvent::Proto(event))?;
                            }
                        }
                        EndpointEvent::Transmit(transmit) => {
                            self.notify_transmit(transmit);
                        }
                    }
                }
                Err(_) => {
                    // No more messages to be read.
                    return Ok(false);
                }
            }
        }

        return Ok(true);
    }
}
