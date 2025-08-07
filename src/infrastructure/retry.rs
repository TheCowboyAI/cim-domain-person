//! Retry policies and dead letter queue handling

use async_nats::{jetstream, Client};
use cim_domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, warn, info};

use super::streaming::{RetryPolicy, EventMetadata};

/// Retry handler for failed event processing
pub struct RetryHandler {
    client: Client,
    jetstream: jetstream::Context,
    policy: RetryPolicy,
    dlq_subject: String,
}

impl RetryHandler {
    /// Create a new retry handler
    pub fn new(
        client: Client,
        jetstream: jetstream::Context,
        policy: RetryPolicy,
        dlq_subject: String,
    ) -> Self {
        Self {
            client,
            jetstream,
            policy,
            dlq_subject,
        }
    }
    
    /// Execute a function with retry logic
    pub async fn execute_with_retry<F, T, E>(
        &self,
        operation: F,
        context: &str,
    ) -> Result<T, E>
    where
        F: Fn() -> futures::future::BoxFuture<'static, Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempts = 0;
        let mut backoff = self.policy.initial_backoff;
        
        loop {
            match operation().await {
                Ok(result) => {
                    if attempts > 0 {
                        info!("Operation {} succeeded after {} attempts", context, attempts + 1);
                    }
                    return Ok(result);
                }
                Err(err) => {
                    attempts += 1;
                    
                    if attempts > self.policy.max_retries {
                        error!(
                            "Operation {} failed after {} attempts: {}",
                            context, attempts, err
                        );
                        return Err(err);
                    }
                    
                    warn!(
                        "Operation {} failed (attempt {}/{}): {}, retrying in {:?}",
                        context, attempts, self.policy.max_retries + 1, err, backoff
                    );
                    
                    sleep(backoff).await;
                    
                    // Exponential backoff with jitter
                    backoff = std::cmp::min(
                        backoff.mul_f64(self.policy.multiplier),
                        self.policy.max_backoff,
                    );
                    
                    // Add jitter (±10%)
                    let jitter = backoff.as_secs_f64() * 0.1 * (rand::random::<f64>() - 0.5);
                    backoff = backoff + Duration::from_secs_f64(jitter);
                }
            }
        }
    }
    
    /// Send a failed event to the dead letter queue
    pub async fn send_to_dlq(&self, event: FailedEvent) -> DomainResult<()> {
        let payload = serde_json::to_vec(&event)
            .map_err(|e| DomainError::SerializationError(e.to_string()))?;
        
        self.jetstream
            .publish(self.dlq_subject.clone(), payload.into())
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to publish to DLQ: {e}"),
            })?;
        
        info!(
            "Event {} sent to DLQ after {} failures",
            event.event_id, event.failure_count
        );
        
        Ok(())
    }
    
    /// Check if retry handler is connected
    pub async fn is_connected(&self) -> bool {
        // Use the client to check connection status
        match self.client.connection_state() {
            async_nats::connection::State::Connected => {
                debug!("Retry handler client is connected");
                true
            }
            state => {
                warn!("Retry handler client state: {:?}", state);
                false
            }
        }
    }
    
    /// Reprocess failed events from DLQ
    pub async fn reprocess_dlq_events(&self, limit: usize) -> DomainResult<Vec<FailedEvent>> {
        use futures::StreamExt;
        
        info!("Reprocessing up to {} events from DLQ", limit);
        
        // Subscribe to DLQ using the client
        let mut subscription = self.client
            .subscribe(self.dlq_subject.clone())
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS".to_string(),
                message: format!("Failed to subscribe to DLQ: {}", e),
            })?;
        
        let mut reprocessed = Vec::new();
        let mut count = 0;
        
        while let Some(msg) = subscription.next().await {
            if count >= limit {
                break;
            }
            
            // Deserialize failed event
            if let Ok(failed_event) = serde_json::from_slice::<FailedEvent>(&msg.payload) {
                debug!("Reprocessing event {} from DLQ", failed_event.event_id);
                reprocessed.push(failed_event);
                
                // For regular NATS messages, no explicit ack needed
                // (ack is only for JetStream messages)
            }
            
            count += 1;
        }
        
        info!("Reprocessed {} events from DLQ", reprocessed.len());
        Ok(reprocessed)
    }
}

/// Failed event information for dead letter queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedEvent {
    /// Original event ID
    pub event_id: uuid::Uuid,
    /// Original subject
    pub original_subject: String,
    /// Event payload
    pub payload: serde_json::Value,
    /// Failure reason
    pub failure_reason: String,
    /// Number of delivery attempts
    pub failure_count: u32,
    /// First failure timestamp
    pub first_failed_at: chrono::DateTime<chrono::Utc>,
    /// Last failure timestamp
    pub last_failed_at: chrono::DateTime<chrono::Utc>,
    /// Consumer that failed to process
    pub failed_consumer: String,
    /// Original event metadata
    pub metadata: EventMetadata,
}

/// Circuit breaker for handling repeated failures
pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    state: tokio::sync::RwLock<CircuitState>,
}

#[derive(Debug, Clone)]
struct CircuitState {
    status: CircuitStatus,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitStatus {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout,
            state: tokio::sync::RwLock::new(CircuitState {
                status: CircuitStatus::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
            }),
        }
    }
    
    /// Execute a function with circuit breaker protection
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, E>>,
    {
        // Check if circuit should transition from Open to HalfOpen
        {
            let mut state = self.state.write().await;
            if state.status == CircuitStatus::Open {
                if let Some(last_failure) = state.last_failure_time {
                    let elapsed = chrono::Utc::now().signed_duration_since(last_failure);
                    if elapsed.to_std().unwrap_or_default() >= self.timeout {
                        state.status = CircuitStatus::HalfOpen;
                        state.success_count = 0;
                    }
                }
            }
        }
        
        // Check current state
        let current_status = {
            let state = self.state.read().await;
            state.status.clone()
        };
        
        match current_status {
            CircuitStatus::Open => Err(CircuitBreakerError::Open),
            CircuitStatus::Closed | CircuitStatus::HalfOpen => {
                match operation().await {
                    Ok(result) => {
                        self.record_success().await;
                        Ok(result)
                    }
                    Err(err) => {
                        self.record_failure().await;
                        Err(CircuitBreakerError::OperationFailed(err))
                    }
                }
            }
        }
    }
    
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        state.success_count += 1;
        state.failure_count = 0;
        
        if state.status == CircuitStatus::HalfOpen && state.success_count >= self.success_threshold {
            state.status = CircuitStatus::Closed;
            info!("Circuit breaker closed after {} successful operations", state.success_count);
        }
    }
    
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        state.failure_count += 1;
        state.success_count = 0;
        state.last_failure_time = Some(chrono::Utc::now());
        
        if state.failure_count >= self.failure_threshold {
            state.status = CircuitStatus::Open;
            warn!("Circuit breaker opened after {} failures", state.failure_count);
        }
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,
    OperationFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::Open => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::OperationFailed(err) => write!(f, "Operation failed: {}", err),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for CircuitBreakerError<E> {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(2, 2, Duration::from_secs(5));
        
        // First failure
        let result: Result<(), &str> = breaker.execute(|| {
            Box::pin(async { Err("fail") })
        }).await;
        assert!(matches!(result, Err(CircuitBreakerError::OperationFailed(_))));
        
        // Second failure - should open circuit
        let result: Result<(), &str> = breaker.execute(|| {
            Box::pin(async { Err("fail") })
        }).await;
        assert!(matches!(result, Err(CircuitBreakerError::OperationFailed(_))));
        
        // Third attempt - circuit should be open
        let result: Result<(), &str> = breaker.execute(|| {
            Box::pin(async { Ok(()) })
        }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }
}