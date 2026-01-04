use aether_traits::{TrafficStrategy, TrafficMetrics};
use async_trait::async_trait;
use rand::Rng;
use std::time::Duration;
use tokio::sync::Barrier;
use std::sync::Arc;
use chrono::{Timelike, Local};

/// Normal operation strategy: Consistent, low-jitter cadence.
pub struct SmoothFlow;

#[async_trait]
impl TrafficStrategy for SmoothFlow {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        let mut rng = rand::thread_rng();
        let base_ms = 300;
        let jitter_ms = rng.gen_range(0..50);
        Duration::from_millis(base_ms + jitter_ms)
    }

    fn name(&self) -> &str {
        "SmoothFlow"
    }
}

/// Defensive strategy: High variance, long delays to mimic human-like or erratic behavior.
pub struct StealthJitter;

#[async_trait]
impl TrafficStrategy for StealthJitter {
    async fn next_delay(&self, metrics: &TrafficMetrics) -> Duration {
        let mut rng = rand::thread_rng();
        let base_ms = 2000;
        let jitter_ms = rng.gen_range(0..5000);
        let error_penalty_ms = if metrics.error_count > 0 { 5000 } else { 0 };
        Duration::from_millis(base_ms + jitter_ms + error_penalty_ms)
    }

    fn name(&self) -> &str {
        "StealthJitter"
    }
}

/// [Category A] Poisson Arrival: Simulates independent events at a constant average rate.
pub struct PoissonArrival {
    pub target_rps: f64,
}

#[async_trait]
impl TrafficStrategy for PoissonArrival {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        let mut rng = rand::thread_rng();
        let u: f64 = rng.gen_range(0.0..1.0);
        let u = if u == 0.0 { 0.0001 } else { u };
        let delay_sec = -u.ln() / self.target_rps;
        Duration::from_secs_f64(delay_sec)
    }

    fn name(&self) -> &str {
        "PoissonArrival"
    }
}

/// [Category A] Micro-Burst: Toggles between extreme RPS and silence.
pub struct MicroBurst {
    pub burst_rps: f64,
    pub burst_duration_ms: u64,
    pub idle_duration_ms: u64,
}

#[async_trait]
impl TrafficStrategy for MicroBurst {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let cycle_ms = self.burst_duration_ms + self.idle_duration_ms;
        let phase = now % cycle_ms;

        if phase < self.burst_duration_ms {
            Duration::from_secs_f64(1.0 / self.burst_rps)
        } else {
            Duration::from_millis(cycle_ms - phase)
        }
    }

    fn name(&self) -> &str {
        "MicroBurst"
    }
}

/// [Category A] Slowloris: Maintains open connections by sending traffic just before timeout.
pub struct Slowloris {
    pub timeout_ms: u64,
}

#[async_trait]
impl TrafficStrategy for Slowloris {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        let mut rng = rand::thread_rng();
        let min_delay = (self.timeout_ms as f64 * 0.90) as u64;
        let max_delay = (self.timeout_ms as f64 * 0.95) as u64;
        Duration::from_millis(rng.gen_range(min_delay..max_delay))
    }

    fn name(&self) -> &str {
        "Slowloris"
    }
}

/// [Category A] Race Condition Trigger: Synchronizes multiple workers to fire at the exact same moment.
pub struct RaceConditionTrigger {
    pub barrier: Arc<Barrier>,
    pub base_delay_ms: u64,
}

#[async_trait]
impl TrafficStrategy for RaceConditionTrigger {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        Duration::from_millis(self.base_delay_ms)
    }

    async fn wait(&self) {
        self.barrier.wait().await;
    }

    fn name(&self) -> &str {
        "RaceConditionTrigger"
    }
}

/// [Category A] Heartbeat: Periodic "pulse" traffic with absolute periodicity.
pub struct Heartbeat {
    pub interval_ms: u64,
}

#[async_trait]
impl TrafficStrategy for Heartbeat {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        Duration::from_millis(self.interval_ms)
    }

    fn name(&self) -> &str {
        "Heartbeat"
    }
}

/// [Category A] Jittered Constant: Baseline RPS with Gaussian approximation variance.
pub struct JitteredConstant {
    pub target_rps: f64,
}

#[async_trait]
impl TrafficStrategy for JitteredConstant {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        let mut rng = rand::thread_rng();
        let sum: f64 = (0..6).map(|_| rng.gen_range(0.0..1.0)).sum();
        let gaussian_approx = (sum / 6.0) * 0.4 + 0.8; 
        
        let target_delay = 1.0 / self.target_rps;
        Duration::from_secs_f64(target_delay * gaussian_approx)
    }

    fn name(&self) -> &str {
        "JitteredConstant"
    }
}

/// [Category A] Working Hours: Ramps down or pauses during "Off" hours.
pub struct WorkingHours {
    pub start_hour: u32,
    pub end_hour: u32,
    pub off_hour_rps: f64,
    pub on_hour_rps: f64,
}

#[async_trait]
impl TrafficStrategy for WorkingHours {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        let hour = Local::now().hour();
        
        let active = if self.start_hour < self.end_hour {
            hour >= self.start_hour && hour < self.end_hour
        } else {
            hour >= self.start_hour || hour < self.end_hour
        };

        let rps = if active { self.on_hour_rps } else { self.off_hour_rps };
        
        if rps <= 0.0 {
            Duration::from_secs(60) 
        } else {
            Duration::from_secs_f64(1.0 / rps)
        }
    }

    fn name(&self) -> &str {
        "WorkingHours"
    }
}

/// [Category A] Geo-Latency: Synthetic synthetic propagation delay for global simulation.
pub struct GeoLatency {
    pub region_latency_ms: u64,
    pub baseline_rps: f64,
}

#[async_trait]
impl TrafficStrategy for GeoLatency {
    async fn next_delay(&self, _metrics: &TrafficMetrics) -> Duration {
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..20); 
        
        Duration::from_millis(self.region_latency_ms + jitter + (1000.0 / self.baseline_rps) as u64)
    }

    fn name(&self) -> &str {
        "GeoLatency"
    }
}

/// [Category A] Decoy & Sniper: Camouflages intense bursts within low-volume background noise.
pub struct DecoySniper {
    pub decoy_strategy: Arc<dyn TrafficStrategy>,
    pub sniper_strategy: Arc<dyn TrafficStrategy>,
    pub sniper_ratio: f64, // e.g. 0.1 for 10% sniper
}

#[async_trait]
impl TrafficStrategy for DecoySniper {
    async fn next_delay(&self, metrics: &TrafficMetrics) -> Duration {
        let use_sniper = {
            let mut rng = rand::thread_rng();
            rng.gen_range(0.0..1.0) < self.sniper_ratio
        };
        
        if use_sniper {
            self.sniper_strategy.next_delay(metrics).await
        } else {
            self.decoy_strategy.next_delay(metrics).await
        }
    }

    async fn wait(&self) {
        // [Feature] Decoy & Sniper: Always call wait, internal strategies may or may not implement it.
        self.decoy_strategy.wait().await;
        self.sniper_strategy.wait().await;
    }

    fn name(&self) -> &str {
        "DecoySniper"
    }
}
