use std::ops::{Deref, DerefMut};

/// A simple inlinable box that automatically inlines if
/// a type's size and alignment are less than or equal
/// to that of `usize`.
#[derive(Debug)]
pub struct InlinableBox<T>(usize, std::marker::PhantomData<T>);

const fn can_inline<T>() -> bool {
    std::mem::size_of::<T>() <= std::mem::size_of::<usize>()
        && std::mem::align_of::<T>() <= std::mem::align_of::<usize>()
}

const fn _size_test() {
    #[repr(align(1024))]
    struct Big([u8; 4096]);

    let _: [u8; std::mem::size_of::<usize>()] = [0; std::mem::size_of::<InlinableBox<()>>()];
    let _: [u8; std::mem::align_of::<usize>()] = [0; std::mem::align_of::<InlinableBox<()>>()];

    let _: [u8; std::mem::size_of::<usize>()] = [0; std::mem::size_of::<InlinableBox<Big>>()];
    let _: [u8; std::mem::align_of::<usize>()] = [0; std::mem::align_of::<InlinableBox<Big>>()];
}

impl<T> Drop for InlinableBox<T> {
    fn drop(&mut self) {
        if can_inline::<T>() {
            unsafe { std::ptr::drop_in_place((&mut self.0) as *mut usize as *mut T) }
        } else {
            unsafe { drop(Box::from_raw(self.0 as *mut T)) }
        }
    }
}

impl<T: Clone> Clone for InlinableBox<T> {
    fn clone(&self) -> Self {
        InlinableBox::new(self.deref().clone())
    }
}

impl<T> Deref for InlinableBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let ptr = if can_inline::<T>() {
            (&self.0) as *const usize as *const T
        } else {
            self.0 as *const T
        };

        unsafe { &*ptr }
    }
}

impl<T> DerefMut for InlinableBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        let ptr = if can_inline::<T>() {
            (&mut self.0) as *mut usize as *mut T
        } else {
            self.0 as *mut T
        };

        unsafe { &mut *ptr }
    }
}

impl<T> InlinableBox<T> {
    pub fn new(item: T) -> InlinableBox<T> {
        let integer = if can_inline::<T>() {
            let mut integer = 0_usize;
            unsafe {
                std::ptr::write((&mut integer) as *mut usize as *mut T, item);
            }
            integer
        } else {
            let ptr: *mut T = Box::into_raw(Box::new(item));
            ptr as usize
        };
        InlinableBox(integer, std::marker::PhantomData)
    }

    pub fn take(self) -> T {
        let item: T = if can_inline::<T>() {
            unsafe { std::ptr::read(self.deref()) }
        } else {
            let ptr: *mut T = self.0 as *mut T;
            let boxed: Box<T> = unsafe { Box::from_raw(ptr) };
            *boxed
        };

        std::mem::forget(self);
        item
    }
}
