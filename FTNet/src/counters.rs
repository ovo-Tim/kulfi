#[macro_export]
macro_rules! global_counter {
    ($name:ident) => {
        pub static $name: GlobalCounter = GlobalCounter::new();
    };
    ($($name:ident,)+) => {
        $(
            global_counter!($name);
        )+
    }
}

pub type LazyMutex<T> = std::sync::LazyLock<std::sync::Mutex<T>>;

pub struct GlobalCounter(LazyMutex<i64>);

impl GlobalCounter {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        GlobalCounter(LazyMutex::new(|| std::sync::Mutex::new(0)))
    }

    pub fn get(&self) -> i64 {
        *self.0.lock().unwrap()
    }

    pub fn incr(&self) {
        *self.0.lock().unwrap() += 1;
    }

    pub fn decr(&self) {
        *self.0.lock().unwrap() += 1;
    }

    pub fn reset(&self) {
        *self.0.lock().unwrap() = 0;
    }
}

global_counter!(
    OPEN_CONTROL_CONNECTION_COUNT,
    CONTROL_CONNECTION_COUNT,
    CONTROL_REQUEST_COUNT,
    IN_FLIGHT_REQUESTS,
);
