fn main() {
    println!("Hello, world!");
    let max: i32 = 10;

    let (sender, receiver) = std::sync::mpsc::channel();
    let sender2 = sender.clone();
    //Beware: the receiver cannot be cloned since it's single consumer

    let f1 = move || {
        for i in 0..max {
            let s: String = format!("From Thread 1: {}", i);
            sender.send(s).unwrap(); //send consumes/moves information, we cannot use s after this ---> ownership
            //std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }; //this is a closure, an anonymous function that can capture variables from its environment
        //move keyword is necessary to move the ownership of the captured variables into the closure, since they will be used in a different thread
        //this is important since the current scope will end before the thread can use the variables, so we need to move them into the closure to ensure they live long enough
        //the || syntax is used to define the parameters of the closure, in this case we have no parameters, but we could have some if we wanted to
        //example: let f = |x: i32| -> i32 { x + 1 }; this is a closure that takes an i32 and returns an i32, it adds 1 to the input and returns it

    let f2 = move || {
        for i in 10..max+10 {
            let s: String = format!("From Thread 2: {}", i);
            sender2.send(s).unwrap(); //we cannot use sender again here since we had already moved it in f1
            //std::thread::sleep(std::time::Duration::from_millis(500));
        }
    };

    let t1 = std::thread::spawn(f1);
    let t2 = std::thread::spawn(f2);

    // for message in receiver {
    //     println!("Received {}", message);
    // }

    while let Ok(message) = receiver.recv() { //.recv() is blocking until a value is received
        //we could also use try_recv() which isn't blocking and allows for parallel work; it simply returns a Result<T, E>
        println!("Received {}", message);
    }

    {
        t1.join().unwrap();
        t2.join().unwrap(); //channels must always be closed, otherwise execution will eventually halt
    }

    //Mutexes
    let counter = 0;
    let mut_counter = std::sync::Mutex::new(counter); //we've defended the counter with a mutex
        //using a mutex here was not necessary since we are inside the main thread, we couldn't have been disturbed
        //we simply did it for demonstration purposes
        //Anyways, we can't use this standard counter in order threads, we need an atomic one, Arc<T>, Atomic Reference Counters
        //In all threads, we make use of pointers to a shared object

    let c = std::sync::Arc::new(mut_counter);
        //this is the Arc, not the mutex; we could reference it with (*...) but it's not necessary

    let c1 = c.clone();
    let c2 = c.clone();

    let f3 = move || {
        {
            for _i in 0..1000042 {
                let mut data = c1.lock().unwrap();       
                *data += 1;   
            }
        }
        
    };
    let t3 = std::thread::spawn(f3);

    let f4 = move || {
        {
            for _i in 0..1000000 {
                let mut data = c2.lock().unwrap();       
                *data -= 1;   
            }
        }
        
    };
    let t4 = std::thread::spawn(f4);

    {
        t3.join().unwrap();
        t4.join().unwrap();

        let data = c.lock().unwrap();
        println!("Shared counter value: {}", *data);
    }
    //putting all of this inside curly braces (statement) automatically unlocks the object at the end


    /*
    / New Exercise: shared state and message passing
    */
    let (new_sender, new_receiver) = std::sync::mpsc::channel();
    let new_sender1 = new_sender.clone();

    let vec: Vec<i32> = Vec::new();
    let mutex_vec = std::sync::Mutex::new(vec);
    let shared_vec = std::sync::Arc::new(mutex_vec);
    let shared_vec1 = shared_vec.clone();
    let shared_vec2 = shared_vec.clone();
    let max2 = 10;
    let f5 = move || {
        {
            for i in 0..max2 {
                let mut point = shared_vec1.lock().unwrap();
                point.push(i);
                let s: String = format!("From Thread 5: {}", i);
                new_sender.send(s).unwrap();
            }
        }
    };

    let t5 = std::thread::spawn(f5);

    let f6 = move || {
        {
            for i in 10..max2+10 {
                let mut point = shared_vec2.lock().unwrap();
                point.push(i);
                let s: String = format!("From Thread 6: {}", i);
                new_sender1.send(s).unwrap();
            }
        }
    };

    let t6 = std::thread::spawn(f6);

    while let Ok(message) = new_receiver.recv() { 
        println!("Received {}", message);
    }

    {
        t5.join().unwrap();
        t6.join().unwrap();
        let data = shared_vec.lock().unwrap();
        println!("Final vector: {:#?}", *data);
    }

}




//MPSC (Multiple Producer Single Consumer): several threads can send messages to a single thread that receives them. 
//This is useful for coordinating work between threads without sharing state directly.
//SPSC is more performing than MPSC, but MPSC is more flexible since it allows for multiple producers.
#[allow(dead_code)]
fn create_channel() {
    //let (sender, receiver) = std::sync::mpsc::channel();
}

//A mutex is an object that defends another object from race conditions
//Thanks to mutextes, you can access an object concurrently from several threads
//lock() is an atomic operation, from a hardware POV it can only be touched by one thread at a time, guaranteed
//Performance can be an issue if several threads want to lock the same object
#[allow(dead_code)]
fn mutexes() {

}
