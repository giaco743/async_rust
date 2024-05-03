// struct SelfRef<'a>{
//     data: String,
//     reference: Option<&'a String>,
// }

// impl<'a> SelfRef<'a> {
//     fn new(data: String) -> Self {
//         SelfRef {
//             data,
//             reference: None,
//         }
//     }
//     fn init(&mut self) {
//         self.reference = Some(&self.data);
//     }
// }

use std::mem::swap;

mod free_to_move {

    use std::fmt::Display;

    pub struct SelfRef {
        data: String,
        reference: *const String,
    }

    impl Display for SelfRef {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.reference.is_null() {
                return write!(f, "data: {}, reference: null", self.data);
            }
            unsafe { write!(f, "data: {}, reference: {}", self.data, *self.reference) }
        }
    }

    impl SelfRef {
        pub fn new(data: String) -> Self {
            SelfRef {
                data,
                reference: std::ptr::null(),
            }
        }
        pub fn init(&mut self) {
            self.reference = &self.data as *const String;
        }
    }
}

mod unpin_pinned {

    use std::fmt::Display;

    pub struct SelfRef {
        data: String,
        reference: *const String,
    }

    impl Display for SelfRef {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.reference.is_null() {
                return write!(f, "data: {}, reference: null", self.data);
            }
            unsafe { write!(f, "data: {}, reference: {}", self.data, *self.reference) }
        }
    }

    impl SelfRef {
        pub fn new(data: String) -> Self {
            SelfRef {
                data,
                reference: std::ptr::null(),
            }
        }
        pub fn init(mut self: std::pin::Pin<&mut Self>) {
            self.reference = &self.data as *const String;
        }
    }
}

mod not_unpin_pinned {

    use std::fmt::Display;

    pub struct SelfRef {
        data: String,
        reference: *const String,
        _pin: std::marker::PhantomPinned,
    }

    impl Display for SelfRef {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.reference.is_null() {
                return write!(f, "data: {}, reference: null", self.data);
            }
            unsafe { write!(f, "data: {}, reference: {}", self.data, *self.reference) }
        }
    }

    impl SelfRef {
        pub fn new(data: String) -> Self {
            SelfRef {
                data,
                reference: std::ptr::null(),
                _pin: std::marker::PhantomPinned,
            }
        }
        pub fn init(mut self: std::pin::Pin<&mut Self>) {
            unsafe {
                let this = self.as_mut().get_unchecked_mut();
                this.reference = &this.data as *const String;
            }
        }
    }
}

fn swap_free_to_move_self_ref() {
    use free_to_move::SelfRef;

    let mut self_ref_a = SelfRef::new(String::from("Hello"));
    let mut self_ref_b = SelfRef::new(String::from("World"));

    println!("Before init:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    swap(&mut self_ref_a, &mut self_ref_b);

    println!("Before init after swap:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    self_ref_a.init();
    self_ref_b.init();

    println!("After init:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    swap(&mut self_ref_a, &mut self_ref_b);

    println!("After init after swap:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);
}

fn swap_unpin_pinned_self_ref() {
    use unpin_pinned::SelfRef;

    let mut self_ref_a = SelfRef::new(String::from("Hello"));
    let mut self_ref_b = SelfRef::new(String::from("World"));

    println!("Before init:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    swap(&mut self_ref_a, &mut self_ref_b);

    println!("Before init after swap:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    let mut self_ref_a = Box::pin(self_ref_a);
    let mut self_ref_b = Box::pin(self_ref_b);

    self_ref_a.as_mut().init();
    self_ref_b.as_mut().init();

    println!("After init:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    let mut self_ref_a = std::pin::Pin::into_inner(self_ref_a);
    let mut self_ref_b = std::pin::Pin::into_inner(self_ref_b);

    swap(&mut *self_ref_a, &mut *self_ref_b);

    println!("After init after swap:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);
}

fn swap_not_unpin_pinned_self_ref() {
    use not_unpin_pinned::SelfRef;

    let mut self_ref_a = SelfRef::new(String::from("Hello"));
    let mut self_ref_b = SelfRef::new(String::from("World"));

    println!("Before init:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    swap(&mut self_ref_a, &mut self_ref_b);

    println!("Before init after swap:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    let mut self_ref_a = Box::pin(self_ref_a);
    let mut self_ref_b = Box::pin(self_ref_b);

    self_ref_a.as_mut().init();
    self_ref_b.as_mut().init();

    println!("After init:");
    println!("self_ref_a: {}", self_ref_a);
    println!("self_ref_b: {}", self_ref_b);

    // Since not_unpin_pinned::SelfRef does NOT implements Unpin, we have to use unsafe code to unpin it
    unsafe {
        let mut self_ref_a = std::pin::Pin::into_inner_unchecked(self_ref_a);
        let mut self_ref_b = std::pin::Pin::into_inner_unchecked(self_ref_b);

        swap(&mut *self_ref_a, &mut *self_ref_b);

        println!("After init after swap:");
        println!("self_ref_a: {}", self_ref_a);
        println!("self_ref_b: {}", self_ref_b);
    }
}

fn main() {
    println!("--- swap_free_to_move_self_ref ---");
    swap_free_to_move_self_ref();
    // println!("\n--- swap_unpin_pinned_self_ref ---");
    // swap_unpin_pinned_self_ref();
    // println!("\n--- swap_not_unpin_pinned_self_ref ---");
    // swap_not_unpin_pinned_self_ref();
}
