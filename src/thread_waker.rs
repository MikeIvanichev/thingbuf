feature! {
    #![feature = "std"]

    use core::task::{RawWaker, RawWakerVTable, Waker};

    use crate::loom::sync::Arc;
    use crate::loom::thread::{self, Thread};
    use crate::loom::thread_local;

    thread_local!{
        static CURRENT: Waker= ThreadWaker::new().into_waker();
    }

    pub(crate) fn current() -> Waker {
        CURRENT.with(|waker| waker.clone())
    }

    struct ThreadWaker(Arc<Inner>);

    impl ThreadWaker {
        fn new() -> Self {
            Self(Arc::new(Inner::new(thread::current())))
        }

        fn into_waker(self) -> Waker {
            unsafe {
                let raw = thread_waker_to_raw_waker(self.0);
                Waker::from_raw(raw)
                }
        }
    }

    struct Inner(Thread);

    impl Inner {
        fn new(thread: Thread) -> Self {
            Self(thread)
        }
    }

    impl Inner {
        #[allow(clippy::wrong_self_convention)]
        fn into_raw(this: Arc<Inner>) -> *const () {
            Arc::into_raw(this) as *const ()
        }

        unsafe fn from_raw(ptr: *const ()) -> Arc<Inner> {
            Arc::from_raw(ptr as *const Inner)
        }
    }

    unsafe fn thread_waker_to_raw_waker(thread_waker: Arc<Inner>) -> RawWaker {
        RawWaker::new(
            Inner::into_raw(thread_waker),
            &RawWakerVTable::new(clone, wake, wake_by_ref, drop_waker),
        )
    }

    unsafe fn clone(raw: *const ()) -> RawWaker {
        Arc::increment_strong_count(raw as *const Inner);
        thread_waker_to_raw_waker(Inner::from_raw(raw))
    }

    unsafe fn drop_waker(raw: *const ()) {
        drop(Inner::from_raw(raw));
    }

    unsafe fn wake(raw: *const ()) {
        let unparker = Inner::from_raw(raw);
        unparker.0.unpark();
    }

    unsafe fn wake_by_ref(raw: *const ()) {
        let raw = raw as *const Inner;
        (*raw).0.unpark();
    }

}
