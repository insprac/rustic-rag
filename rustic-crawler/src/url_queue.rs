use std::{collections::HashSet, sync::Mutex};

/// A simple async FILO URL queue that keeps track of all URLs already seen as well as ones yet to
/// be crawled.
///
/// ## Example
///
/// ```
/// let queue = UrlQueue::new(vec!["https://example.com".to_string()]);
///
/// queue.push(vec![
///     "https://example.com/home".to_string(),
///     "https://example.com/example".to_string()
/// ]);
///
/// let Some(url) = queue.take();
/// println!("{url}");
/// ```
pub struct UrlQueue {
    queue: Mutex<Vec<String>>,
    seen: Mutex<HashSet<String>>,
}

impl UrlQueue {
    /// Create a new queue with any number of starting URLs.
    pub fn new(start_urls: Vec<String>) -> Self {
        Self {
            queue: Mutex::new(start_urls.clone()),
            seen: Mutex::new(HashSet::from_iter(start_urls)),
        }
    }

    /// Adds a list of URLs to the back of the queue and keeps track of which ones have been seen.
    /// URLs are filtered out if they've been added previously ensuring all added URLs are unique.
    pub fn push(&self, urls: Vec<String>) {
        let mut queue = self.queue.lock().expect("queue lock was poisoned");
        let mut seen = self.seen.lock().expect("queue lock was poisoned");

        for url in urls {
            if !seen.contains(&url) {
                queue.push(url.clone());
                seen.insert(url);
            }
        }
    }

    /// Take a single URL off the back of the queue.
    /// Returns `None` if the queue is empty.
    pub fn take(&self) -> Option<String> {
        let mut queue = self.queue.lock().expect("queue lock was poisoned");
        queue.pop()
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use tokio::task::JoinSet;

    use super::*;

    #[test]
    fn sync_test() {
        let queue = UrlQueue::new(vec!["https://example.com/1".to_string()]);

        queue.push(vec![
            "https://example.com/2".to_string(),
            "https://example.com/3".to_string(),
        ]);

        assert_eq!(queue.take(), Some("https://example.com/3".to_string()));
        assert_eq!(queue.take(), Some("https://example.com/2".to_string()));

        queue.push(vec![
            "https://example.com/4".to_string(),
            "https://example.com/5".to_string(),
        ]);

        assert_eq!(queue.take(), Some("https://example.com/5".to_string()));
        assert_eq!(queue.take(), Some("https://example.com/4".to_string()));
        assert_eq!(queue.take(), Some("https://example.com/1".to_string()));
        assert_eq!(queue.take(), None);
    }

    #[tokio::test]
    async fn async_test() {
        let queue = Arc::new(UrlQueue::new(vec!["https://example.com/1".to_string()]));
        let taken_urls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let mut set = JoinSet::new();

        // Spawn 10 publishers each adding unique and some duplicate URLs.
        for index in 0..10 {
            let queue = queue.clone();
            set.spawn(async move {
                queue.push(vec![
                    "https://example.com/duplicate".to_string(),
                    format!("https://example.com/{index}/1"),
                    format!("https://example.com/{index}/2"),
                ]);

                queue.push(vec![
                    format!("https://example.com/{index}/3"),
                    format!("https://example.com/{index}/4"),
                    "https://example.com/duplicate".to_string(),
                ]);
            });
        }

        // Spawn 10 consumers which consume 10 URLs each.
        for _ in 0..10 {
            let queue = queue.clone();
            let taken_urls = taken_urls.clone();
            set.spawn(async move {
                for _ in 0..10 {
                    let mut taken_urls = taken_urls.lock().expect("taken_urls lock was poisoned");
                    if let Some(taken_url) = queue.take() {
                        taken_urls.push(taken_url);
                    }
                }
            });
        }

        set.join_all().await;

        // Queue should be empty.
        assert_eq!(queue.take(), None);

        // The total taken URLs should be: PUBLISHERS * 4 + DUPLICATES + STARTING_URL
        // PUBLISHERS: 10
        // DUPLICATES: 1
        // STARTING_URL: 1
        let taken_urls = taken_urls.lock().expect("taken_urls lock was poisoned");
        assert_eq!(taken_urls.len(), 42);

        // All taken URLs should be unique.
        let taken_urls_hashset: HashSet<String> = HashSet::from_iter(taken_urls.clone());
        assert_eq!(taken_urls_hashset.len(), 42);
    }
}
