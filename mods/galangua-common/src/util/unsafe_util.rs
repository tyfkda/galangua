use std::cell::UnsafeCell;

pub unsafe fn peep<'a, T>(t: &T) -> &'a mut T {
    get_mut(get_shared(t))
}

pub unsafe fn extend_lifetime<T: ?Sized>(t: &T) -> &'static T {
    std::mem::transmute::<&T, &'static T>(t)
}

// Taken from "Memory layout" in [UnsafeCell in std::cell - Rust](https://doc.rust-lang.org/nightly/std/cell/struct.UnsafeCell.html#memory-layout)

fn get_shared<T>(ptr: &T) -> &UnsafeCell<T> {
    let t = ptr as *const T as *const UnsafeCell<T>;
    // SAFETY: `T` and `UnsafeCell<T>` have the same memory layout
    unsafe { &*t }
}

// Safety: the caller must ensure that there are no references that
// point to the *contents* of the `UnsafeCell`.
unsafe fn get_mut<'a, T>(ptr: &UnsafeCell<T>) -> &'a mut T {
    unsafe { &mut *ptr.get() }
}
