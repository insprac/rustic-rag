use clap::Parser;
use crawler::{Crawler, CrawlerConfig};

mod crawler;
mod url_queue;

#[derive(Parser, Debug)]
struct Args {
    /// The crawler starts here, it will then branch out to all allowed URLs on that page.
    #[arg(short, long)]
    start_url: String,
    /// The maximum number of pages allowed to be crawled per second.
    #[arg(short, long, default_value = "15")]
    rate_limit: u64,
    /// A list of URL patterns that are allowed to be crawled.
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    allow_urls: Vec<String>,
    /// A list of URL patterns that aren't allowed to be crawled, this takes priority over allowed.
    #[arg(short, long, num_args = 0.., value_delimiter = ' ')]
    disallow_urls: Vec<String>,
    /// The number of worker threads to spawn, more threads = more parallelisation and higher
    /// RAM/CPU usage.
    #[arg(short, long, default_value = "20")]
    thread_count: u64,
}

fn main() {
    let Args { start_url, rate_limit, allow_urls, disallow_urls, thread_count } = Args::parse();

    if allow_urls.len() == 0 {
        println!("error: there must be at least 1 allow url");
        return;
    }

    let config = CrawlerConfig {
        rate_limit,
        allow_urls,
        disallow_urls,
        thread_count,
    };

    Crawler::new(config, start_url).start();
}
