use super::*;

/// Blocks async progression for `duration`.
///
/// Clamps duration between 0 and [`i32::MAX`] milliseconds
pub async fn delay(duration: std::time::Duration) {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _reject| {
        DelayedFn::new_js(&resolve, duration);
    }))
    .await
    .expect("delay timeout resolves");
}

#[macro_export]
/// Helper macro to log diagnostics about created function
macro_rules! new_delayed_fn {
    ($closure:expr, $delay:expr) => {{
        let delayed_fn = DelayedFn::new_mut($closure, $delay);
        log::debug!("Created {delayed_fn}");
        delayed_fn
    }};
    (Tracked, $closure:expr, $delay:expr) => {{
        let delayed_fn = TrackedDelayedFn::new($closure, $delay);
        log::debug!("Created {delayed_fn}");
        delayed_fn
    }};
}

#[macro_export]
/// Helper macro to log diagnostics about stopped function
macro_rules! stop_delayed_fn {
    ($delayed_fn:expr) => {{
        log::debug!("Stopping {}", $delayed_fn);
        $delayed_fn.stop()
    }};
}

/// handle to a delayed function call
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct DelayedFn {
    id: i32,
}

impl std::fmt::Display for DelayedFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "delayed function {}", self.id)
    }
}

impl DelayedFn {
    /// Creates a new delayed function call
    /// Clamps duration between 0 and [`i32::MAX`] milliseconds
    pub fn new_mut<F: FnMut() + 'static>(function: F, duration: std::time::Duration) -> Self {
        let function: js_sys::Function = wasm_bindgen::closure::Closure::new(function)
            .into_js_value()
            .into();
        Self::new_js(&function, duration)
    }

    /// Creates a new delayed function call
    /// Clamps duration between 0 and [`i32::MAX`] milliseconds
    pub fn new_once<F: FnOnce() + 'static>(function: F, duration: std::time::Duration) -> Self {
        let function: js_sys::Function = wasm_bindgen::closure::Closure::once(function)
            .into_js_value()
            .into();
        Self::new_js(&function, duration)
    }

    /// Creates a new delayed function call
    /// Clamps duration between 0 and [`i32::MAX`] milliseconds
    pub fn new_js(function: &js_sys::Function, duration: std::time::Duration) -> Self {
        let delay_ms = match duration.as_millis().try_into() {
            Ok(delay_ms) => delay_ms,
            Err(err) => {
                log::error!("Invalid duration: {err}");
                i32::MAX
            }
        };

        let id = browser_window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(function, delay_ms)
            .expect("Undocumented error creating callback timeout doesn't occur");
        DelayedFn { id }
    }

    /// Stops the function call from executing if it hasn't already.
    pub fn stop(&self) {
        browser_window().clear_timeout_with_handle(self.id);
    }
}

/// A wrapper around [`Self::DelayedFn`] that tracks execution and stopping
#[derive(Debug)]
pub struct TrackedDelayedFn {
    delayed_fn: DelayedFn,
    executed: std::rc::Rc<std::cell::RefCell<bool>>,
    stopped: bool,
}

impl std::fmt::Display for TrackedDelayedFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "delayed function {}, status: {}",
            self.delayed_fn.id,
            if let Some(true) | None = self.is_executed() {
                "executed"
            } else if self.is_stopped() {
                "stopped"
            } else {
                "pending"
            }
        )
    }
}

impl TrackedDelayedFn {
    /// Creates a new [`Self::DelayedFn`] with addition tracking for stopping and execution
    /// Clamps duration between 0 and [`i32::MAX`] milliseconds
    pub fn new<F: FnMut() + 'static>(mut function: F, duration: std::time::Duration) -> Self {
        let executed = std::rc::Rc::new(std::cell::RefCell::new(false));

        let tracker_executed_ref = std::rc::Rc::clone(&executed);
        let tracker_fn = move || {
            function();
            *tracker_executed_ref.borrow_mut() = true;
        };

        Self {
            delayed_fn: DelayedFn::new_mut(tracker_fn, duration),
            executed,
            stopped: false,
        }
    }

    /// Stops the execution of the delayed function if it hasn't finished executing
    /// Returns if the delayed function was stopped before it could execute
    pub fn stop(&mut self) -> bool {
        if let Some(false) = self.is_executed() {
            self.delayed_fn.stop();
            self.stopped = true;
        }
        self.is_stopped()
    }

    /// Returns if the delayed function was stopped
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// Optionally returns if the delayed function was executed
    /// Returns None if the delayed function is about to finish executing
    pub fn is_executed(&self) -> Option<bool> {
        (*self.executed).try_borrow().ok().as_deref().copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use instant::Duration;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn setup_delayed_function<'a>() -> (impl Fn() + Clone, Duration, std::rc::Rc<std::cell::RefCell<usize>>)
    {
        use std::{cell::RefCell, rc::Rc};

        let fn_call_count = Rc::new(RefCell::new(usize::MIN));
        let closure_fn_calls_ref = Rc::clone(&fn_call_count);
        let closure = move || {
            let mut fn_calls_ref = (*closure_fn_calls_ref).borrow_mut();
            *fn_calls_ref = *fn_calls_ref + 1;
            log::debug!("Incremented fn_calls: {}", *fn_calls_ref);
        };
        (closure, Duration::from_millis(16), fn_call_count)
    }

    #[wasm_bindgen_test]
    async fn test_delayed_fn() {
        wasm_logger::init(wasm_logger::Config::default());

        let (closure, closure_delay, closure_call_count) = setup_delayed_function();

        let _delayed_fn = new_delayed_fn!(closure, closure_delay);
        assert_eq!(*closure_call_count.borrow(), 0);
        delay(closure_delay * 2).await;
        assert_eq!(*closure_call_count.borrow(), 1);
    }

    #[wasm_bindgen_test]
    async fn test_check_executed_delayed_fn() {
        wasm_logger::init(wasm_logger::Config::default());

        let (closure, closure_delay, closure_call_count) = setup_delayed_function();

        let delayed_fn = new_delayed_fn!(Tracked, closure, closure_delay);

        assert_eq!(*closure_call_count.borrow(), 0);
        assert!(!delayed_fn.is_stopped());
        assert!(matches!(delayed_fn.is_executed(), Some(false)));

        while let Some(false) | None = delayed_fn.is_executed() {
            delay(Duration::ZERO).await;
        }

        assert_eq!(*closure_call_count.borrow(), 1);
        assert!(!delayed_fn.is_stopped());
        assert!(matches!(delayed_fn.is_executed(), Some(true)));
    }

    #[wasm_bindgen_test]
    async fn test_stopped_delayed_fn() {
        wasm_logger::init(wasm_logger::Config::default());

        let (closure, closure_delay, closure_call_count) = setup_delayed_function();

        let mut delayed_fn = new_delayed_fn!(Tracked, closure, closure_delay);

        assert_eq!(*closure_call_count.borrow(), 0);
        assert!(!delayed_fn.is_stopped());
        assert!(matches!(delayed_fn.is_executed(), Some(false)));

        assert!(stop_delayed_fn!(delayed_fn));

        assert_eq!(*closure_call_count.borrow(), 0);
        assert!(delayed_fn.is_stopped());
        assert!(matches!(delayed_fn.is_executed(), Some(false)));

        delay(closure_delay * 2).await;

        assert_eq!(*closure_call_count.borrow(), 0);
        assert!(delayed_fn.is_stopped());
        assert!(matches!(delayed_fn.is_executed(), Some(false)));
    }

    #[wasm_bindgen_test]
    async fn test_stop_after_executed_delayed_fn() {
        wasm_logger::init(wasm_logger::Config::default());

        let (closure, closure_delay, closure_call_count) = setup_delayed_function();

        let mut delayed_fn = new_delayed_fn!(Tracked, closure, closure_delay);

        assert_eq!(*closure_call_count.borrow(), 0);
        assert!(!delayed_fn.is_stopped());
        assert!(matches!(delayed_fn.is_executed(), Some(false)));

        while let Some(false) | None = delayed_fn.is_executed() {
            delay(Duration::ZERO).await;
        }

        assert_eq!(*closure_call_count.borrow(), 1);
        assert!(!delayed_fn.is_stopped());
        assert!(matches!(delayed_fn.is_executed(), Some(true)));

        assert!(!stop_delayed_fn!(delayed_fn));

        assert_eq!(*closure_call_count.borrow(), 1);
        assert!(!delayed_fn.is_stopped());
        assert!(matches!(delayed_fn.is_executed(), Some(true)));
    }

    #[wasm_bindgen_test]
    async fn test_race_delayed_fn() {
        wasm_logger::init(wasm_logger::Config::default());

        let (closure, closure_delay, closure_call_count) = setup_delayed_function();

        let mut delayed_fn = new_delayed_fn!(Tracked, closure, closure_delay);

        while let Some(false) = delayed_fn.is_executed() {
            delay(Duration::ZERO).await;
        }

        if let None = delayed_fn.is_executed() {
            assert!(!stop_delayed_fn!(delayed_fn));
        }
    }
}
