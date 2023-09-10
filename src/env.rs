use once_cell::sync::Lazy;
use std::env;

pub static PRIMARY_LUNCH_CHANNEL: Lazy<String> = Lazy::new(|| {
    env::var("PRIMARY_LUNCH_CHANNEL")
        .ok()
        .unwrap_or("".to_owned())
});

pub static SCHOOL_KEY: Lazy<String> = Lazy::new(|| {
    env::var("API_SCHOOL_KEY")
        .ok()
        .expect("Expected API_SCHOOL_KEY in the environment")
});
