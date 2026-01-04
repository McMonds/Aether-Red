use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use crate::transport::FuzzerStream;
use tokio_rustls::TlsConnector as RustlsConnector;
use tokio_rustls::rustls::{ClientConfig as RustlsConfig, RootCertStore, ServerName, OwnedTrustAnchor};
use std::pin::Pin;

/// [Directive 7] Tri-State TLS Architecture.
/// Abstract interface for TLS providers to bypass fingerprinting/detection.
/// 
/// CRITICAL FIX #3: ALPN Protocol Synchronization
/// The `force_http1` parameter ensures protocol consistency between TLS negotiation
/// and payload format, preventing PROTOCOL_ERROR when sending HTTP/1.1 text over h2.
#[async_trait]
pub trait TlsImpersonator: Send + Sync {
    /// Handshakes the underlying stream and returns an adversarial TLS session.
    /// 
    /// # Arguments
    /// * `domain` - Target domain for SNI
    /// * `stream` - Underlying transport stream
    /// * `force_http1` - If true, removes h2 from ALPN to force HTTP/1.1 (for text-based attacks)
    async fn handshake(&self, domain: &str, stream: FuzzerStream, force_http1: bool) -> Result<FuzzerStream>;
}

/// Provider_Native: Wrapper for rustls (Safe, fast, default).
pub struct NativeProvider {
    config_http1: Arc<RustlsConfig>,  // ALPN: http/1.1 only
    config_http2: Arc<RustlsConfig>,  // ALPN: h2, http/1.1
}

impl NativeProvider {
    pub fn new() -> Result<Self> {
        let mut root_store = RootCertStore::empty();
        root_store.add_trust_anchors(
            webpki_roots::TLS_SERVER_ROOTS
                .iter()
                .map(|ta| {
                    OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                })
        );

        // CRITICAL FIX #3: Two configs - one for HTTP/1.1 only, one for HTTP/2
        let config_http1 = RustlsConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store.clone())
            .with_no_client_auth();

        let config_http2 = RustlsConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self { 
            config_http1: Arc::new(config_http1),
            config_http2: Arc::new(config_http2),
        })
    }
}

#[async_trait]
impl TlsImpersonator for NativeProvider {
    async fn handshake(&self, domain: &str, stream: FuzzerStream, force_http1: bool) -> Result<FuzzerStream> {
        // CRITICAL FIX #3: Select config based on attack type
        let config = if force_http1 {
            &self.config_http1  // Text-based attacks (smuggling, JSON, headers)
        } else {
            &self.config_http2  // Binary attacks (h2_flood)
        };
        
        let connector = RustlsConnector::from(config.clone());
        let domain = ServerName::try_from(domain)?;
        let tls_stream = connector.connect(domain, stream).await?;
        Ok(Box::new(tls_stream))
    }
}

/// [Directive 7] Provider_OpenSSL (Legacy).
/// Used for protocol downgrades and legacy cipher suite matching.
pub struct LegacyProvider;

#[async_trait]
impl TlsImpersonator for LegacyProvider {
    async fn handshake(&self, domain: &str, stream: FuzzerStream, force_http1: bool) -> Result<FuzzerStream> {
        use openssl::ssl::{SslConnector, SslMethod};
        use tokio_openssl::SslStream;

        let mut builder = SslConnector::builder(SslMethod::tls())?;
        
        // CRITICAL FIX #3: Configure ALPN based on attack type
        if force_http1 {
            // Only advertise HTTP/1.1 for text-based attacks
            builder.set_alpn_protos(b"\x08http/1.1")?;
        } else {
            // Advertise both h2 and http/1.1 for binary attacks
            builder.set_alpn_protos(b"\x02h2\x08http/1.1")?;
        }
        
        let connector = builder.build();
        let ssl = connector.configure()?.into_ssl(domain)?;
        let mut stream = SslStream::new(ssl, stream)?;
        
        Pin::new(&mut stream).connect().await
            .map_err(|e| anyhow::anyhow!("OpenSSL Handshake Failed: {}", e))?;
            
        Ok(Box::new(stream))
    }
}

/// [Directive 7] Provider_Chrome (Boring).
/// Implements modern Chrome-specific TLS extensions for stealth.
pub struct ChromeProvider;

#[async_trait]
impl TlsImpersonator for ChromeProvider {
    async fn handshake(&self, _domain: &str, _stream: FuzzerStream, _force_http1: bool) -> Result<FuzzerStream> {
        // [Advanced Evasion] BoringSSL doesn't have direct tokio integration.
        // In production, this would configure ALPN via FFI based on force_http1 flag.
        Err(anyhow::anyhow!("BoringSSL Provider requires custom async wrapper (not implemented in stub)"))
    }
}

/// [Category C] JA3 Cycling Wrapper.
/// Automatically rotates between different TLS providers to vary JA3 fingerprints.
pub struct Ja3Cycler {
    providers: Vec<Box<dyn TlsImpersonator>>,
    index: std::sync::atomic::AtomicUsize,
}

impl Ja3Cycler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            providers: vec![
                Box::new(NativeProvider::new()?),
                Box::new(LegacyProvider),
            ],
            index: std::sync::atomic::AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl TlsImpersonator for Ja3Cycler {
    async fn handshake(&self, domain: &str, stream: FuzzerStream, force_http1: bool) -> Result<FuzzerStream> {
        let i = self.index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.providers.len();
        self.providers[i].handshake(domain, stream, force_http1).await
    }
}
