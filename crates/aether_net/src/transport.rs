use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use std::net::SocketAddr;
use std::time::Duration;
use anyhow::Result;

/// [Directive 6] FuzzerTransport Abstraction.
/// Provides a protocol-agnostic stream for raw-level adversarial execution.
pub type FuzzerStream = Box<dyn RawStream>;

pub trait RawStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> RawStream for T {}

pub struct TransportBuilder;

impl TransportBuilder {
    /// [Directive 3] Abortive Close Pattern (RST Injection) - FIXED.
    /// [Category C] IP Swarm: Optional local address binding for interface rotation.
    /// 
    /// CRITICAL FIX #2: Pre-flight socket configuration BEFORE connection.
    /// 
    /// # Arguments
    /// * `addr` - Remote target address
    /// * `local_addr` - Optional local binding IP (IP Swarm)
    /// * `_force_http1` - ProtocolVersion constraint (Logical constraint for Fix #3)
    pub async fn connect_adversarial(
        addr: SocketAddr, 
        local_addr: Option<SocketAddr>,
        _force_http1: bool,
    ) -> Result<TcpStream> {
        let domain = if addr.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
        
        // Step 1: Create raw socket (not connected yet)
        let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
        
        // Step 2: PRE-FLIGHT CONFIGURATION (before any network activity)
        socket.set_linger(Some(Duration::from_secs(0)))?;  // Abortive close (RST)
        socket.set_nodelay(true)?;                          // Disable Nagle's algorithm
        socket.set_nonblocking(true)?;                      // Required for async
        
        // Step 3: [Category C] IP Swarm - Bind to specific local interface if provided
        if let Some(la) = local_addr {
            let sock_addr = SockAddr::from(la);
            socket.bind(&sock_addr)?;
        }
        
        // Step 4: NOW connect (socket already knows to use RST on close)
        let sock_addr = SockAddr::from(addr);
        match socket.connect(&sock_addr) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Expected for non-blocking sockets
            },
            Err(e) => return Err(e.into()),
        }
        
        // Step 5: Convert to tokio TcpStream
        socket.set_nonblocking(false)?; // Tokio will manage async
        let std_stream: std::net::TcpStream = socket.into();
        let stream = TcpStream::from_std(std_stream)?;
        
        Ok(stream)
    }

    /// [Directive 6] Wraps a stream for Smuggling attacks.
    /// Agnostic to whether the underlying transport is raw TCP or encrypted TLS.
    pub fn into_fuzzer_stream<S>(stream: S) -> FuzzerStream 
    where S: AsyncRead + AsyncWrite + Unpin + Send + 'static
    {
        Box::new(stream)
    }
}
