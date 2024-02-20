use std::sync::{Arc, Mutex};

use cslt::blocking::ClientBuilder;
use q::blocking::ClientBuilder as QBuilder;

use crate::config::Config;

pub struct Client {
    config: Arc<Config>,

    cslt: cslt::blocking::Client,
    q: q::blocking::Q,

    logined: Arc<Mutex<bool>>,
    selected_courses: Arc<Mutex<Vec<String>>>,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Client {
            config: Arc::new(config),
            cslt: ClientBuilder::new().build().unwrap(),
            q: QBuilder::new().build().unwrap(),
            logined: Arc::new(Mutex::new(false)),
            selected_courses: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
