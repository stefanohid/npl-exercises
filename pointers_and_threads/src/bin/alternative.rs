use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let shared_vec = Arc::new(Mutex::new(Vec::new()));

    let (message_sender, message_receiver) = mpsc::channel();

    let (thread1_turn_sender, thread1_turn_receiver) = mpsc::channel::<()>();
    let (thread2_turn_sender, thread2_turn_receiver) = mpsc::channel::<()>();
    let start_thread1 = thread1_turn_sender.clone();

    let shared_vec1 = Arc::clone(&shared_vec);
    let message_sender1 = message_sender.clone();

    let thread1 = thread::spawn(move || {
        for value in 1..=5 {
            thread1_turn_receiver.recv().unwrap();

            {
                let mut vector = shared_vec1.lock().unwrap();
                vector.push(value);
            }

            message_sender1
                .send(format!("Thread 1 inserted {}", value))
                .unwrap();

            thread::sleep(Duration::from_millis(100));

            thread2_turn_sender.send(()).unwrap();
        }
    });

    let shared_vec2 = Arc::clone(&shared_vec);
    let message_sender2 = message_sender.clone();

    let thread2 = thread::spawn(move || {
        for multiplier in 1..=5 {
            thread2_turn_receiver.recv().unwrap();

            let value = multiplier * 10;
            {
                let mut vector = shared_vec2.lock().unwrap();
                vector.push(value);
            }

            message_sender2
                .send(format!("Thread 2 inserted {}", value))
                .unwrap();

            thread::sleep(Duration::from_millis(100));

            if multiplier < 5 {
                thread1_turn_sender.send(()).unwrap();
            }
        }
    });

    start_thread1.send(()).unwrap();
    drop(message_sender);

    for message in message_receiver {
        println!("{}", message);
    }

    thread1.join().unwrap();
    thread2.join().unwrap();

    println!("Final vector: {:?}", shared_vec.lock().unwrap());
}
