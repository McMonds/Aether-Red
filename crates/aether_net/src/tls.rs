use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use crate::transport::FuzzerStream;
use tokio_rustls::TlsConnector as RustlsConnector;
use tokio_rustls::rustls::{ClientConfig as RustlsConfig, RootCertStore, ServerName, OwnedTrustAnchor};
use tokio_rustls::rustls::client::Resumption;
use std::pin::Pin;

/// [Directive 7] Tri-State TLS Architecture.
/// Abstract interface for TLS providers to bypass fingerprinting/detection.
/// 
/// [Category C] Granular protocol control for adversarial evasion.
#[derive(Debug, Clone, Default)]
pub struct AttackProfile {
    pub force_http1: bool,
    pub force_http10: bool,
    pub force_tls11: bool,
    pub use_0rtt: bool,
    pub fragment_handshake: bool,
}

#[async_trait]
pub trait TlsImpersonator: Send + Sync {
    /// Handshakes the underlying stream and returns an adversarial TLS session.
    async fn handshake(
        &self, 
        domain: &str, 
        stream: FuzzerStream, 
        profile: AttackProfile
    ) -> Result<FuzzerStream>;
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

        // [Category C] Session Replay: Shared memory-backed session cache
        let session_cache = rustls::client::ClientSessionMemoryCache::new(256);
        let resumption = Resumption::store(Arc::new(session_cache));

        // CRITICAL FIX #3: Two configs - one for HTTP/1.1 only, one for HTTP/2
        // [Category C] Early Data (0-RTT) enabled on all configs
        let mut config_http1 = RustlsConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store.clone())
            .with_no_client_auth();
        config_http1.alpn_protocols = vec![b"http/1.1".to_vec()];
        config_http1.resumption = resumption.clone();
        config_http1.enable_early_data = true;

        let mut config_http2 = RustlsConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        config_http2.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        config_http2.resumption = resumption;
        config_http2.enable_early_data = true;

        Ok(Self { 
            config_http1: Arc::new(config_http1),
            config_http2: Arc::new(config_http2),
        })
    }
}

#[async_trait]
impl TlsImpersonator for NativeProvider {
    async fn handshake(&self, domain: &str, stream: FuzzerStream, profile: AttackProfile) -> Result<FuzzerStream> {
        // [Category C] Protocol Downgrade: NativeProvider only supports TLS 1.2+
        if profile.force_tls11 {
            return Err(anyhow::anyhow!("NativeProvider (rustls) does not support TLS 1.0/1.1. use LegacyProvider."));
        }

        // CRITICAL FIX #3: Select config based on attack type
        let config = if profile.force_http1 {
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
    async fn handshake(&self, domain: &str, stream: FuzzerStream, profile: AttackProfile) -> Result<FuzzerStream> {
        use openssl::ssl::{SslConnector, SslMethod, SslOptions};
        use tokio_openssl::SslStream;

        // [Category C] Protocol Downgrade logic
        let method = if profile.force_tls11 {
            // Force TLS 1.1 strictly
            SslMethod::tls() 
        } else {
            SslMethod::tls()
        };

        let mut builder = SslConnector::builder(method)?;
        
        if profile.force_tls11 {
            // Disable everything except TLS 1.1
            builder.set_options(SslOptions::NO_TLSV1 | SslOptions::NO_TLSV1_2 | SslOptions::NO_TLSV1_3);
        }

        // CRITICAL FIX #3: Configure ALPN based on attack type
        if profile.force_http1 {
            builder.set_alpn_protos(b"\x08http/1.1")?;
        } else {
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
    async fn handshake(&self, _domain: &str, _stream: FuzzerStream, _profile: AttackProfile) -> Result<FuzzerStream> {
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
    async fn handshake(&self, domain: &str, stream: FuzzerStream, profile: AttackProfile) -> Result<FuzzerStream> {
        let i = self.index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.providers.len();
        self.providers[i].handshake(domain, stream, profile).await
    }
}
