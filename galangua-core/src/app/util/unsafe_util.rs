pub unsafe fn peep<'a, T>(t: &T) -> &'a mut T {
    &mut *(t as *const T as *mut T)
}
