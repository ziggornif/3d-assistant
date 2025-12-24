use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::future::Future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Rate limiter configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: u32,
    /// Maximum burst size
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 50,
        }
    }
}

/// Rate limiter layer
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(config.requests_per_second).unwrap())
            .allow_burst(NonZeroU32::new(config.burst_size).unwrap());

        let limiter = Arc::new(RateLimiter::direct(quota));

        Self { limiter }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

/// Rate limiter service
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let limiter = self.limiter.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            match limiter.check() {
                Ok(_) => {
                    // Request allowed
                    inner.call(req).await
                }
                Err(_) => {
                    // Rate limit exceeded
                    tracing::warn!("Rate limit exceeded");

                    let response = Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .header("Content-Type", "application/json")
                        .header("Retry-After", "1")
                        .body(Body::from(
                            r#"{"error":"Too many requests","message":"Rate limit exceeded. Please try again later."}"#,
                        ))
                        .unwrap();

                    Ok(response)
                }
            }
        })
    }
}

/// Create a default rate limiter for the application
pub fn create_rate_limiter() -> RateLimitLayer {
    let config = RateLimitConfig {
        requests_per_second: 100, // 100 requests per second
        burst_size: 500,          // Allow burst of 500 requests (for dev/test scenarios)
    };

    RateLimitLayer::new(config)
}

/// Create a very strict rate limiter for admin login (brute force protection)
pub fn create_login_rate_limiter() -> RateLimitLayer {
    let config = RateLimitConfig {
        requests_per_second: 1, // 1 login attempt per second
        burst_size: 5,          // Allow only 5 attempts then block
    };

    RateLimitLayer::new(config)
}
