fn main() {
    use std::sync::{Arc, Mutex};
    let mut xs: Vec<i32> = Vec::with_capacity(1000);
    xs.push(1);
    xs.push(11);

    // Wrap xs with Mutex to allow thread-safe modification of content inside it
    let mutex = Mutex::new(xs);
    // Wrap Mutex to thread-safe reference-counting pointer to safely share it with another thread
    let arced_mutex: Arc<Mutex<Vec<i32>>> = Arc::new(mutex);
    // We clone ARC so we can pass it to another thread
    let cloned_arced_mutex = arced_mutex.clone();
    let t = std::thread::spawn(move || {
        // Try to read xs from new thread
        let mut xs = cloned_arced_mutex
            .lock()
            .expect("unable to lock the mutex in the new thread!");
        println!("{:?}: {:?}", std::thread::current(), xs);
        // And push new value
        xs.push(42);
    });
    t.join().expect("Failed to wait the thread to finish");
    let x = arced_mutex
        .lock()
        .expect("unable to lock the mutex in the main thread!");
    println!("{:?}: {:?}", std::thread::current(), x);
}
