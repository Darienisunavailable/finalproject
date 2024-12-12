use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc as mpsc;
use ureq;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io;

// Create a struct to hold the status of the website
// Uses debug to allow printing the struct with {:?}
#[derive(Debug)]
struct Status {
    url: String,
    status: Result<u16, String>,
    response_time: Duration,
}

// Function to check the status of a website
fn check_website(url: String, timeout: Duration, max_retries: usize) -> Status {
    let mut retries = 0;
    // Measure the time it takes to get a response
    let start = Instant::now();
    let mut response = ureq::get(&url).timeout(timeout).call();
    while retries < max_retries && response.is_err() {
        response = ureq::get(&url).timeout(timeout).call();
        retries += 1;
    }
    let response_time = start.elapsed();
    // Match the response to get the status code or error message
    let status = match response {
        Ok(resp) => Ok(resp.status()),
        Err(err) => Err(err.to_string()),
    };
    Status {
        url,
        status,
        response_time,
    }
}

fn read_websites_from_file(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect("Unable to open file");
    // Create a buffered reader to read the file line by line
    let reader = BufReader::new(file);
    let mut websites = vec![];
    for line in reader.lines() {
        websites.push(line.unwrap());
    }
    websites
}

fn main() {
    let websites = read_websites_from_file("websites.txt");

    println!("Please enter the number of worker threads (press Enter to use default 4 threads):");
    let mut input = String::new();
    // Read the input from the user
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let num_threads: usize = if input.trim().is_empty() {
        4
    } else {
        // Parse the input to a number
        input.trim().parse().expect("Please enter a valid number")
    };

    println!("Please enter the timeout duration in seconds (press Enter to use default 5 seconds):");
    // Clear the input buffer
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let timeout_secs: u64 = if input.trim().is_empty() {
        5
    } else {
        input.trim().parse().expect("Please enter a valid number")
    };
    // Create a Duration from the timeout in seconds
    let timeout = Duration::from_secs(timeout_secs);

    println!("Please enter the maximum retries per website (press Enter to use default 3 retries):");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let max_retries: usize = if input.trim().is_empty() {
        3
    } else {
        input.trim().parse().expect("Please enter a valid number")
    };

    println!("Checking websites...");

    let mut handles = vec![];
    // Create a channel to send the status of the websites
    let (tx, rx) = mpsc::channel();

    for url in websites {
        // Clone the variables to move into the thread
        let tx = tx.clone();
        let url = url.to_string();
        let timeout = timeout.clone();
        // Spawn a new thread to check the website
        let handle = thread::spawn(move || {
            let status = check_website(url, timeout, max_retries);
            tx.send(status).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Close the sending side of the channel
    drop(tx);

    // Receive the statuses from the threads
    while let Ok(status) = rx.recv() {
        match status.status {
            Ok(code) => println!(
                "The website {} is up! Status code: {}, Response time: {:?}",
                status.url, code, status.response_time
            ),
            Err(err) => println!(
                "The website {} is down! Error: {}, Response time: {:?}",
                status.url, err, status.response_time
            ),
        }
    }
}