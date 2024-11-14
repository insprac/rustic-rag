use std::sync::{Arc, Mutex};

pub struct CrawlerConfig {
    pub rate_limit: u64,
    pub allow_urls: Vec<String>,
    pub disallow_urls: Vec<String>,
    pub thread_count: u64,
}

/// A thread safe URL queue
struct UrlQueue {
    queue: Mutex<Vec<String>>,

}

impl UrlQueue {
    pub fn new(start_url: String) -> Self {
        Self {
            queue: Mutex::new(vec![start_url]),
        }
    }

    /// Adds a list of urls to the queue
    pub fn add(&self, urls: &mut Vec<String>) {
        let mut queue = self.queue.lock().expect("url queue lock was poisoned");
        queue.append(urls);
    }

    pub fn take(&self) -> Option<String> {
        let mut queue = self.queue.lock().expect("url queue lock was poisoned");
        queue.pop()
    }
}

pub struct Crawler {
    config: Arc<CrawlerConfig>,
    /// A thread safe URL queue
    /// These URLs have already been validated against the config allow/disallow globs.
    urls: Arc<UrlQueue>,
}

impl Crawler {
    pub fn new(config: CrawlerConfig, start_url: String) -> Self {
        Crawler {
            config: Arc::new(config),
            urls: Arc::new(UrlQueue::new(start_url)),
        }
    }

    pub fn start(&self) {
        println!("Crawling...");
    }
}
