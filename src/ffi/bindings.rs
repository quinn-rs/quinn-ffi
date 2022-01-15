use crate::{
    ffi::{
        ConnectionHandle,
        EndpointHandle,
        FFIResult,
        FFIResultKind,
        Handle,
        Out,
        QuinnError,
        Ref,
        RustlsClientConfigHandle,
        RustlsServerConfigHandle,
    },
    proto::{
        ClientConfig,
        DatagramEvent,
        Dir,
        Endpoint,
        EndpointConfig,
        ReadError,
        ServerConfig,
        StreamId,
    },
    proto_impl::{
        generate_self_signed_cert,
        ConnectionImpl,
        EndpointImpl,
        EndpointPoller,
        IpAddr,
        QuinnErrorKind,
        SkipServerVerification,
    },
};
use bytes::BytesMut;
use libc::size_t;
use quinn_proto::{
    VarInt,
    VarIntBoundsExceeded,
};
use rustls::{
    client::{
        ServerCertVerified,
        ServerCertVerifier,
    },
    Certificate,
    KeyLogFile,
    PrivateKey,
    RootCertStore,
};
use std::{
    io::Write,
    net::SocketAddr,
    sync::{
        Arc,
        Mutex,
    },
    time::Instant,
};

use Into;

ffi! {
    /// Creates a server endpoint with a certain configuration.
    ///
    /// * `handle`: Valid `RustlsServerConfigHandle` pointer for the duration of the function call.
    /// * `endpoint_id`: Allocated memory for the endpoint id of the server endpoint.
    /// * `out_endpoint_handle`: Allocated memory for a pointer that will be initialized with `EndpointHandle`.
    ///
    /// Use the returned `EndpointHandle` for endpoint related FFI functions.
    fn create_server_endpoint(handle: RustlsServerConfigHandle, out_endpoint_id: Out<u8>, out_endpoint_handle: Out<EndpointHandle>) -> FFIResult {
        let endpoint_config = Arc::new(EndpointConfig::default());

        let mut endpoint = None;
        let _ = handle.mut_access(&mut |server_config| {
           endpoint = Some(Endpoint::new(endpoint_config.clone(), Some(Arc::from(server_config.clone()))));
           Ok(())
        });

        let endpoint = EndpointImpl::new(endpoint.unwrap());
        let endpoint_id = endpoint.id;

        let mut endpoint_handle = EndpointHandle::new(endpoint);

        let (poller, poll_notifier) = EndpointPoller::new(endpoint_handle.clone());
        poller.start_polling();

        let mut endpoint_lock = endpoint_handle.mut_access(&mut move |endpoint| {
            endpoint.set_poll_notifier(poll_notifier.clone());
            Ok(())
        });

        unsafe {
            out_endpoint_id.init(endpoint_id);
            out_endpoint_handle.init(endpoint_handle);
        }

        FFIResult::ok()
    }

    /// Creates a client endpoint with a certain configuration.
    ///
    /// * `handle`: Valid `RustlsClientConfigHandle` pointer for the duration of the function call.
    /// * `endpoint_id`: Allocated memory for the endpoint id of the new endpoint.
    /// * `out_endpoint_handle`: Allocated memory for a pointer that will be initialized with `EndpointHandle`.
    ///
    /// Use the returned `EndpointHandle` for endpoint related FFI functions.
    fn create_client_endpoint(
        handle: RustlsClientConfigHandle,
        endpoint_id: Out<u8>,
        out_endpoint_handle: Out<EndpointHandle>
    ) -> FFIResult {
        let endpoint_config = Arc::new(EndpointConfig::default());

        let mut proto_endpoint = Endpoint::new(endpoint_config, None);
        let mut endpoint = EndpointImpl::new(proto_endpoint);

        let _ = handle.mut_access(&mut |client_config| {
          endpoint.set_default_client_config(client_config.clone());
           Ok(())
        });

        let endpoint_identifier = endpoint.id;

        let endpoint = EndpointHandle::new(endpoint);

        let (poller, poll_notifier) = EndpointPoller::new(endpoint.clone());
        poller.start_polling();

        let mut endpoint_lock = endpoint.lock().unwrap();
        endpoint_lock.set_poll_notifier(poll_notifier);
        drop(endpoint_lock);
        unsafe {
            endpoint_id.init(endpoint_identifier);
            out_endpoint_handle.init(endpoint)
        }

        FFIResult::ok()
    }

    /// Connects a client to some remote address.
    ///
    /// * `handle`: Valid `EndpointHandle` pointer for the duration of the function call.
    /// * `address`: A type defining a socket address. Make sure to use correct layout.
    /// * `out_connection`: Allocated memory for a pointer that will be initialized with `ConnectionHandle`.
    /// * `out_connection_id`: Allocated memory for the connection id of the new connection.
    ///
    /// Use the returned `ConnectionHandle` for connection related FFI functions.
    fn connect_client(
        handle: EndpointHandle,
        address: IpAddr,
        out_connection: Out<ConnectionHandle>,
        out_connection_id: Out<u32>
    ) -> FFIResult {
        handle.mut_access(&mut |endpoint| {
            // TODO: remove localhost with Ref<u8> pointing to string.
            let connection = endpoint.connect(address.into(), "localhost").unwrap();

            unsafe {
                out_connection_id.init(connection.connection_handle.0 as u32);
                out_connection.init(ConnectionHandle::new(connection))
            }
           Ok(())
       }).into()
    }

    /// Handles the given datagram.
    ///
    /// * `handle`: Valid `EndpointHandle` pointer for the duration of the function call.
    /// * `data`: Reference to memory storing the buffer containing the datagram.
    /// * `length`: The length of the buffer storing the datagram.
    /// * `address`: A type defining a socket address. Make sure to use correct layout.
    fn handle_datagram(handle: EndpointHandle, data: Ref<u8>, length: size_t, address: IpAddr) -> FFIResult {
        handle.mut_access(&mut |endpoint| {
            let slice = unsafe { data.as_bytes(length) };

            let addr: SocketAddr = address.into();

            match endpoint
                .inner
                .handle(Instant::now(), addr, None, None, BytesMut::from(slice))
            {
                Some((handle, DatagramEvent::NewConnection(conn))) => {
                    let mut connection = endpoint.add_connection(handle, conn);
                    connection.poll();

                    let connection_handle = super::ConnectionHandle::new(connection);
                    endpoint.register_pollable_connection(handle, connection_handle.clone());

                    callbacks::on_new_connection( connection_handle, handle.0 as u32,);
                }
                Some((handle, DatagramEvent::ConnectionEvent(event))) => {
                    endpoint.forward_event_to_connection(handle, event)?;

                    endpoint.poll_connection(handle);
                }
                None => {
                    println!("None handled");
                }
            }

            Ok(())
        }).into()

    }
}

ffi! {
    /// Polls a given connection.
    ///
    /// * `handle`: Valid `ConnectionHandle` pointer for the duration of the function call.
    fn poll_connection(handle: ConnectionHandle) -> FFIResult {
      handle.mut_access(&mut |connection| {
        let a = connection.poll();
        a
      }).into()
    }
}

ffi! {
    /// Retrieves the last occurred error.
    ///
    /// * `error_buf`: Allocated memory for the error message destination.
    /// * `error_buf_len`: The size of the allocated error message buffer `error_buf`.
    /// * `actual_error_buf_len`: Allocated memory for the actual length of the error buffer.
    ///
    /// `actual_error_buf_len` could be used to resize buffer if result returns `BufferToSmall`.
   fn last_error(error_buf: Out<u8>, error_buf_len: size_t, actual_error_buf_len: Out<size_t>) -> FFIResult {
        FFIResult::from_last_result(|last_result| {
            if let Some(error_msg) = last_result {
                let error_as_bytes = error_msg.reason.as_bytes();

                // "The out pointer is valid and not mutably aliased elsewhere"
                unsafe {
                    actual_error_buf_len.init(error_as_bytes.len());
                }

                if error_buf_len < error_as_bytes.len() {
                    return FFIResult::buffer_too_small();
                }

                // "The buffer is valid for writes and the length is within the buffer"
                unsafe {
                    error_buf.init_bytes(error_as_bytes);
                }
            }
            FFIResult::ok()
        })
    }
}

ffi! {
    /// Accepts a stream.
    ///
    /// * `handle`: Valid `ConnectionHandle` pointer for the duration of the function call.
    /// * `stream_direction`: The direction of the stream to accept.
    /// * `stream_id_out`: Allocated memory for the `stream_id` of the accepted stream.
    fn accept_stream(handle: ConnectionHandle, stream_direction: u8, stream_id_out: Out<u64>) -> FFIResult {
        let dir = dir_from_u8(stream_direction);
        println!("access read");
        handle.mut_access(&mut |connection| {
           let result = if let Some(stream_id) = connection.inner.streams().accept(dir) {
                connection.mark_pollable();
                unsafe {
                    stream_id_out.init(VarInt::from(stream_id).into());
                }
                Ok(())
            } else {
                Err(QuinnErrorKind::QuinnError {code: 0, reason: "No stream to accept!".to_string()})
            };

            println!("after mut access: {:?}", result);
            result
        }).into()

    }

    /// Reads from a stream.
    ///
    /// * `handle`: Valid `ConnectionHandle` pointer for the duration of the function call.
    /// * `stream_id`: The id of the stream to read from.
    /// * `message_buf`: Allocated memory for the buffer destination.
    /// * `message_buf_len`: The size of the allocated memory buffer `message_buf`.
    /// * `actual_message_len`: Allocated memory for number of bytes read.
    ///
    /// `actual_message_len` could be used to resize buffer if result returns `BufferToSmall`.
    fn read_stream(handle: ConnectionHandle, stream_id: u64, message_buf: Out<u8>, message_buf_len: size_t, actual_message_len: Out<size_t>) -> FFIResult {
         handle.mut_access(&mut |connection| {
            _read_stream(
                connection,
                stream_id,
                &mut message_buf,
                message_buf_len,
                &mut actual_message_len,
            )
        }).into()
    }

    /// Writes to a stream.
    ///
    /// * `handle`: Valid `ConnectionHandle` pointer for the duration of the function call.
    /// * `stream_id`: The id of the stream to write to.
    /// * `buffer`: Allocated and initialized memory for the buffer that is written.
    /// * `buf_len`: Length of the allocated and initialized memory buffer `buffer`.
    /// * `written_bytes`: Allocated memory for the number of bytes written.
    fn write_stream(handle: ConnectionHandle, stream_id: u64, buffer: Ref<u8>, buf_len: size_t, written_bytes: Out<size_t>) -> FFIResult {
        handle.mut_access(&mut move |connection| {
            _write_stream(connection, stream_id, &mut buffer, buf_len, &mut written_bytes).into()
        }).into()
    }

    /// Opens a stream with a certain directionality.
    ///
    /// * `handle`: Valid `ConnectionHandle` pointer for the duration of the function call.
    /// * `stream_direction`: The direction of the stream that is opened.
    /// * `opened_stream_id`: Allocated memory for the stream id that is opened.
    fn open_stream(handle: ConnectionHandle, stream_direction: u8, opened_stream_id: Out<u64>) -> FFIResult {
        handle.mut_access(&mut move |connection| {
           let opened_stream = connection.inner.streams().open(dir_from_u8(stream_direction));

            if let Some(stream_id) = opened_stream {
                unsafe { opened_stream_id.init(_stream_id_to_u64(stream_id)) }
                Ok(())
            } else {
                Err(QuinnErrorKind::QuinnError {code: 0, reason: "Streams in the given direction are currently exhausted".to_string()})
            }
        }).into()
    }
}

ffi! {
    /// Test function for generating server config.
    fn default_server_config(out_handle: Out<RustlsServerConfigHandle>) -> FFIResult {
        // tracing::subscriber::set_global_default(
        //     tracing_subscriber::FmtSubscriber::builder()
        //         .with_env_filter("trace")
        //         .finish(),
        // )
        // .unwrap();

        let (key, cert) = generate_self_signed_cert("cert.der", "key.der");

        let (key, cert) = (PrivateKey(key), Certificate(cert));
        let mut store = RootCertStore::empty();
        store.add(&cert);

        let mut config = rustls::ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13])
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();

        config.key_log = Arc::new(KeyLogFile::new());

        let config = ServerConfig::with_crypto(Arc::new(config));

        unsafe { out_handle.init(RustlsServerConfigHandle::new(ServerConfig::from(config))) }

        FFIResult::ok()
    }

    /// Test function for generating server config.
    fn default_client_config(out_handle: Out<RustlsClientConfigHandle>) -> FFIResult {
        let mut crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(SkipServerVerification::new())
            .with_no_client_auth();

        crypto.key_log = Arc::new(KeyLogFile::new());

        unsafe {
            out_handle.init(RustlsClientConfigHandle::new(ClientConfig::new(Arc::new(
                crypto,
            ))));
        }

        FFIResult::ok()
    }
}

fn _read_stream(
    handle: &mut ConnectionImpl,
    stream_id: u64,
    message_buf: &mut Out<u8>,
    message_buf_len: size_t,
    actual_message_len: &mut Out<size_t>,
) -> Result<(), QuinnErrorKind> {
    let mut stream = handle.inner.recv_stream(_stream_id(stream_id)?);

    let mut result = stream.read(true)?;

    match result.next(message_buf_len) {
        Ok(Some(chunk)) => unsafe {
            let mut buffer = unsafe { message_buf.as_uninit_bytes_mut(message_buf_len) };

            let written = buffer.write(&chunk.bytes)?;

            actual_message_len.init(written);
        },
        Err(e) => {
            if result.finalize().should_transmit() {
                handle.mark_pollable();
            }
            if e == ReadError::Blocked {
                return Err(QuinnErrorKind::QuinErrorKind(FFIResultKind::BufferBlocked));
            }

            return Err(e.into());
        }
        _ => {}
    }

    if result.finalize().should_transmit() {
        handle.mark_pollable();
    }

    Ok(())
}

fn _write_stream(
    handle: &mut ConnectionImpl,
    stream_id: u64,
    buffer: &mut Ref<u8>,
    buf_len: size_t,
    written_bytes: &mut Out<size_t>,
) -> Result<(), QuinnErrorKind> {
    let mut stream = handle.inner.send_stream(_stream_id(stream_id)?);

    let bytes = unsafe { buffer.as_bytes(buf_len) };
    let result = stream.write(bytes)?;
    unsafe {
        written_bytes.init(result);
    }
    handle.mark_pollable();

    Ok(())
}

fn dir_from_u8(dir: u8) -> Dir {
    if dir == 0 {
        Dir::Bi
    } else {
        Dir::Uni
    }
}

fn _stream_id_to_u64(stream_id: StreamId) -> u64 {
    VarInt::from(stream_id).into_inner()
}

fn _stream_id(stream_id: u64) -> Result<StreamId, VarIntBoundsExceeded> {
    Ok(StreamId::from(VarInt::from_u64(stream_id)?))
}

pub mod callbacks {
    //! Callbacks that are invoked when events occure
    use crate::{
        ffi::{
            ConnectionHandle,
            FFIResult,
            Handle,
        },
        proto::{
            Dir,
            StreamId,
            Transmit,
        },
        proto_impl::{
            ConnectionImpl,
            IpAddr,
        },
    };
    use libc::size_t;
    use quinn_proto::VarInt;

    /// Generates FFI methods to set callbacks and declares the static variable to store that callback.
    #[doc(hidden)]
    macro_rules! set_callbacks {
        ($(fn $name:ident ( $($arg_ty:ty),* ) set $body:ident)*) => {
             $(
                // A static option with external function pointer.
                static mut $body: Option<extern "C" fn($($arg_ty),*)> = None;

                #[no_mangle]
                /// Set a callback that will be invoked when some event occurs.
                ///
                /// See the callback function pointer for what arguments are expected.
                 pub extern "cdecl" fn $name (callback: extern "C" fn($($arg_ty),*)) -> FFIResult {
                    unsafe {
                        $body = Some(callback);
                    }
                    FFIResult::ok()
                }
              )*
        };
    }

    /// Generates callback invoke methods.
    #[doc(hidden)]
    macro_rules! set_invokers {
        ($(invoke $name:ident with $fn_name:ident ( $( $arg_ident:ident : $arg_ty:ty),* ) )*) => {
             $(
                /// Invoke the callback.
                pub(crate) fn $fn_name($($arg_ident: $arg_ty),*) {
                    unsafe {
                       $name.unwrap_unchecked()($($arg_ident),*);
                    }
                }
              )*
        };

        // Allows parsing parameters with `call(int as u8)` for example.
        ($(invoke $name:ident with $fn_name:ident ( $( $arg_ident:ident : $arg_ty:ty),* ) { call ($($body:expr),* ) }) *) => {
             $(
                /// Invoke the callback.
                pub(crate) fn $fn_name($($arg_ident: $arg_ty),*) {
                    unsafe {
                       $name.unwrap_unchecked()($($body), *);
                    }
                }
              )*
        };
    }

    set_invokers! {
        invoke ON_NEW_CONNECTION with on_new_connection(handle: ConnectionHandle, con: u32)

        invoke ON_CONNECTED with on_connected(con: u32)

        invoke ON_CONNECTION_LOST with on_connection_lost(con: u32)

        invoke ON_STREAM_AVAILABLE with on_stream_available(con: u32, dir: u8)

        invoke ON_DATAGRAM_RECEIVED with on_datagram_received(con: u32)

        invoke ON_STREAM_OPENED with on_stream_opened(con: u32, stream_id: u64, dir: u8)

        invoke ON_CONNECTION_POLLABLE with on_connection_pollable(con: u32)

    }

    set_invokers! {
        invoke ON_STREAM_READABLE with on_stream_readable(con: u32, stream_id: StreamId) {
            call (con,VarInt::from(stream_id).into(),stream_id.dir() as u8)
        }

        invoke ON_STREAM_WRITABLE with on_stream_writable(con: u32, stream_id: StreamId) {
            call (con,VarInt::from(stream_id).into(),stream_id.dir() as u8)
        }

        invoke ON_STREAM_FINISHED with on_stream_finished(con: u32, stream_id: StreamId) {
            call (con,VarInt::from(stream_id).into(),stream_id.dir() as u8)
        }

        invoke ON_STREAM_STOPPED with on_stream_stopped(con: u32, stream_id: StreamId) {
            call (con,VarInt::from(stream_id).into(),stream_id.dir() as u8)
        }

        invoke ON_TRANSMIT with on_transmit(endpoint_id: u8, transmit: Transmit) {
            call (endpoint_id,transmit.contents.as_ptr(),transmit.contents.len(),&transmit.destination.into())
        }
    }

    set_callbacks! {
        fn set_on_new_connection(super::ConnectionHandle, u32) set ON_NEW_CONNECTION

        fn set_on_connected(u32) set ON_CONNECTED

        fn set_on_connection_lost(u32) set ON_CONNECTION_LOST

        fn set_on_stream_writable(u32, u64, u8) set ON_STREAM_WRITABLE

        fn set_on_stream_readable(u32, u64, u8) set ON_STREAM_READABLE

        fn set_on_stream_finished(u32, u64, u8) set ON_STREAM_FINISHED

        fn set_on_stream_stopped(u32, u64, u8) set ON_STREAM_STOPPED

        fn set_on_stream_available(u32, u8) set ON_STREAM_AVAILABLE

        fn set_on_datagram_received(u32) set ON_DATAGRAM_RECEIVED

        fn set_on_stream_opened(u32, u64, u8) set ON_STREAM_OPENED

        fn set_on_transmit(u8, *const u8, size_t, *const IpAddr) set ON_TRANSMIT

        fn set_on_pollable_connection(u32) set ON_CONNECTION_POLLABLE
    }
}
