use super::*;
use std::ptr;
use std::os::raw::c_void;
use std::cell::UnsafeCell;

/// A special case for non-thread-aware functions.
///
/// This context is used by default and you don't need to create it manually.
#[doc(hidden)]
pub struct GlobalContext {
    _not_thread_safe: UnsafeCell<()>
}

#[doc(hidden)]
pub trait Context {
    fn as_ptr(&self) -> ffi::Context;
}

impl Context for GlobalContext {
    #[inline]
    fn as_ptr(&self) -> ffi::Context {
        ptr::null_mut()
    }
}

impl<'a> Context for &'a ThreadContext {
    #[inline]
    fn as_ptr(&self) -> ffi::Context {
        self.handle
    }
}

/// Per-thread context for multi-threaded operation.
///
/// There are situations where several instances of Little CMS engine have to coexist but on different conditions.
/// For example, when the library is used as a DLL or a shared object, diverse applications may want to use different plug-ins.
/// Another example is when multiple threads are being used in same task and the user wants to pass thread-dependent information to the memory allocators or the logging system.
/// The context is a pointer to an internal structure that keeps track of all plug-ins and static data needed by the THR corresponding function.
///
/// A context-aware app could allocate a new context by calling new() or duplicate a yet-existing one by using clone().
/// Each context can hold different plug-ins, defined by the Plugin parameter. The context can also hold loggers.
///
/// Users may associate private data across a void pointer when creating the context, and can retrieve this pointer later.
///
/// When you see an error "expected reference, found struct `lcms2::GlobalContext`", it means you've mixed global and thread-context objects. They don't work together.
/// For example, if you create a `Transform` with a context (calling `new_*_context()`), then it will only support `Profile` with a context as well.
pub struct ThreadContext {
    handle: ffi::Context,
    // _user_data: PhantomData<UserData>
}

// pub type ContextUserData = *mut std::os::raw::c_void;

impl GlobalContext {
    pub fn new() -> Self {
        Self {
            _not_thread_safe: UnsafeCell::new(()),
        }
    }

    pub fn unregister_plugins(&mut self) {
        unsafe {
            ffi::cmsUnregisterPlugins();
        }
    }
}

impl ThreadContext {
    pub fn new() -> Self {
        unsafe {
            Self::new_handle(ffi::cmsCreateContext(ptr::null_mut(), ptr::null_mut()))
        }
    }

    unsafe fn new_handle(handle: ffi::Context) -> Self {
        assert!(!handle.is_null());
        Self {handle}
    }

    pub fn user_data(&self) -> *mut c_void {
        unsafe {
            ffi::cmsGetContextUserData(self.handle)
        }
    }

    pub unsafe fn install_plugin(&mut self, plugin: *mut c_void) -> bool {
        0 != ffi::cmsPluginTHR(self.handle, plugin)
    }

    pub fn unregister_plugins(&mut self) {
        unsafe {
            ffi::cmsUnregisterPluginsTHR(self.handle);
        }
    }
}

impl Clone for ThreadContext {
    fn clone(&self) -> Self {
        unsafe {
            Self::new_handle(ffi::cmsDupContext(self.handle, ptr::null_mut()))
        }
    }
}

impl Drop for ThreadContext {
    fn drop(&mut self) {
        unsafe {
            ffi::cmsDeleteContext(self.handle)
        }
    }
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ThreadContext {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn context() {
    let mut c = ThreadContext::new();
    assert!(c.user_data().is_null());
    c.unregister_plugins();
    assert!(Profile::new_icc_context(&c, &[]).is_err());

    let _ = GlobalContext::default();
}
