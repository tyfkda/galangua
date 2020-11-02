pub unsafe fn peep<'a, T: ?Sized>(t: &T) -> &'a mut T {
    &mut *(t as *const T as *mut T)
}

pub unsafe fn extend_lifetime<'a, T: ?Sized>(t: &'a T) -> &'static T {
    std::mem::transmute::<&T, &'static T>(t)
}
