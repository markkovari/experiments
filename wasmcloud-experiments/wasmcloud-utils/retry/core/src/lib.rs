use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum BackoffStrategy {
    Fixed,
    Exponential,
    ExponentialJitter,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub strategy: BackoffStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetrySchedule {
    pub attempt: u32,
    pub delay_ms: u64,
    pub is_last: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetryError {
    NotInitialized,
    Exhausted,
    InvalidConfig,
}

// NOTE: Thread-local config — test stub. In a deployed WASM component this
// would typically be initialised once per component instance at startup.
thread_local! {
    static CONFIG: RefCell<Option<RetryConfig>> = const { RefCell::new(None) };
}

fn with_config<R>(f: impl FnOnce(&RetryConfig) -> R) -> Result<R, RetryError> {
    CONFIG.with(|c| match c.borrow().as_ref() {
        Some(cfg) => Ok(f(cfg)),
        None => Err(RetryError::NotInitialized),
    })
}

/// Initialise the retry policy. Returns `InvalidConfig` when max_attempts is 0
/// or base_delay_ms exceeds max_delay_ms.
pub fn init(config: RetryConfig) -> Result<(), RetryError> {
    if config.max_attempts == 0 {
        return Err(RetryError::InvalidConfig);
    }
    if config.base_delay_ms > config.max_delay_ms {
        return Err(RetryError::InvalidConfig);
    }
    CONFIG.with(|c| *c.borrow_mut() = Some(config));
    Ok(())
}

/// Compute the delay for `attempt` (1-based).
///
/// Formulas (all capped at `max_delay_ms`):
/// - Fixed:              `base_delay_ms`
/// - Exponential:        `base_delay_ms * 2^(attempt-1)`
/// - ExponentialJitter:  exponential ± 25 % via a deterministic pseudo-random
///   function seeded from the attempt number (no OS entropy needed in WASM).
pub fn next_delay(attempt: u32, seed: u64) -> Result<RetrySchedule, RetryError> {
    with_config(|cfg| {
        if attempt == 0 || attempt > cfg.max_attempts {
            return Err(RetryError::Exhausted);
        }
        let base = cfg.base_delay_ms;
        let shift = (attempt - 1).min(62); // guard against overflow
        let raw = match cfg.strategy {
            BackoffStrategy::Fixed => base,
            BackoffStrategy::Exponential => base.saturating_mul(1u64 << shift),
            BackoffStrategy::ExponentialJitter => {
                let exp = base.saturating_mul(1u64 << shift);
                // Deterministic jitter: ±25 % using an LCG seeded by attempt + seed.
                let noise = lcg(seed.wrapping_add(attempt as u64));
                // noise in [0, exp/2); centre around exp → [exp*3/4, exp*5/4)
                let jitter = noise % (exp / 2 + 1);
                exp.saturating_sub(exp / 4).saturating_add(jitter)
            }
        };
        let delay_ms = raw.min(cfg.max_delay_ms);
        Ok(RetrySchedule { attempt, delay_ms, is_last: attempt == cfg.max_attempts })
    })?
}

/// Return the full schedule for every attempt.
pub fn full_schedule(seed: u64) -> Result<Vec<RetrySchedule>, RetryError> {
    let max = with_config(|cfg| cfg.max_attempts)?;
    (1..=max).map(|a| next_delay(a, seed)).collect()
}

/// True when `attempt` is within the allowed window.
pub fn should_retry(attempt: u32) -> Result<bool, RetryError> {
    with_config(|cfg| attempt >= 1 && attempt <= cfg.max_attempts)
}

/// Simple LCG for deterministic jitter — no OS entropy required.
fn lcg(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_cfg(max: u32, base: u64) -> RetryConfig {
        RetryConfig {
            max_attempts: max,
            base_delay_ms: base,
            max_delay_ms: 60_000,
            strategy: BackoffStrategy::Fixed,
        }
    }

    fn exp_cfg(max: u32, base: u64) -> RetryConfig {
        RetryConfig {
            max_attempts: max,
            base_delay_ms: base,
            max_delay_ms: 60_000,
            strategy: BackoffStrategy::Exponential,
        }
    }

    #[test]
    fn test_fixed_delay_constant() {
        init(fixed_cfg(3, 200)).unwrap();
        for a in 1..=3 {
            let s = next_delay(a, 0).unwrap();
            assert_eq!(s.delay_ms, 200, "attempt {}", a);
            assert_eq!(s.is_last, a == 3);
        }
    }

    #[test]
    fn test_exponential_doubles() {
        init(exp_cfg(4, 100)).unwrap();
        let expected = [100, 200, 400, 800];
        for (i, &exp) in expected.iter().enumerate() {
            let s = next_delay(i as u32 + 1, 0).unwrap();
            assert_eq!(s.delay_ms, exp, "attempt {}", i + 1);
        }
    }

    #[test]
    fn test_max_delay_cap() {
        let cfg = RetryConfig {
            max_attempts: 5,
            base_delay_ms: 1000,
            max_delay_ms: 3000,
            strategy: BackoffStrategy::Exponential,
        };
        init(cfg).unwrap();
        // attempt 4: 1000 * 8 = 8000 → capped at 3000
        let s = next_delay(4, 0).unwrap();
        assert_eq!(s.delay_ms, 3000);
    }

    #[test]
    fn test_exhausted_beyond_max() {
        init(fixed_cfg(2, 100)).unwrap();
        assert_eq!(next_delay(3, 0).unwrap_err(), RetryError::Exhausted);
        assert_eq!(next_delay(0, 0).unwrap_err(), RetryError::Exhausted);
    }

    #[test]
    fn test_should_retry() {
        init(fixed_cfg(3, 50)).unwrap();
        assert!(should_retry(1).unwrap());
        assert!(should_retry(3).unwrap());
        assert!(!should_retry(4).unwrap());
        assert!(!should_retry(0).unwrap());
    }

    #[test]
    fn test_full_schedule_length() {
        init(exp_cfg(5, 100)).unwrap();
        let schedule = full_schedule(42).unwrap();
        assert_eq!(schedule.len(), 5);
        assert!(schedule.last().unwrap().is_last);
        assert!(!schedule.first().unwrap().is_last);
    }

    #[test]
    fn test_exponential_jitter_within_range() {
        let cfg = RetryConfig {
            max_attempts: 4,
            base_delay_ms: 1000,
            max_delay_ms: 60_000,
            strategy: BackoffStrategy::ExponentialJitter,
        };
        init(cfg).unwrap();
        for a in 1..=4 {
            let base_exp = 1000u64 << (a - 1);
            let s = next_delay(a, 0).unwrap();
            // Must be within [base*3/4, base*5/4] before cap
            let lo = base_exp * 3 / 4;
            let hi = (base_exp * 5 / 4).min(60_000);
            assert!(
                s.delay_ms >= lo && s.delay_ms <= hi,
                "attempt {}: delay {} not in [{}, {}]",
                a,
                s.delay_ms,
                lo,
                hi
            );
        }
    }

    #[test]
    fn test_invalid_config_zero_attempts() {
        assert_eq!(
            init(RetryConfig {
                max_attempts: 0,
                base_delay_ms: 100,
                max_delay_ms: 1000,
                strategy: BackoffStrategy::Fixed,
            })
            .unwrap_err(),
            RetryError::InvalidConfig
        );
    }

    #[test]
    fn test_invalid_config_base_exceeds_max() {
        assert_eq!(
            init(RetryConfig {
                max_attempts: 3,
                base_delay_ms: 2000,
                max_delay_ms: 1000,
                strategy: BackoffStrategy::Fixed,
            })
            .unwrap_err(),
            RetryError::InvalidConfig
        );
    }

    #[test]
    fn test_not_initialized() {
        let result = std::thread::spawn(|| next_delay(1, 0)).join().unwrap();
        assert_eq!(result.unwrap_err(), RetryError::NotInitialized);
    }
}
