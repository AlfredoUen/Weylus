use std::boxed::Box;
use std::error::Error;
use std::os::raw::{c_int, c_uint, c_void};
use std::slice::from_raw_parts;

use crate::cerror::CError;
use crate::screen_capture::ScreenCapture;
use crate::video::PixelProvider;
use crate::x11helper::X11Capturable;

extern "C" {
    fn start_capture(handle: *const c_void, ctx: *mut c_void, err: *mut CError) -> *mut c_void;
    fn capture_sceen(
        handle: *mut c_void,
        img: *mut CImage,
        capture_cursor: c_int,
        err: *mut CError,
    );
    fn stop_capture(handle: *mut c_void, err: *mut CError);
}

#[repr(C)]
struct CImage {
    data: *const u8,
    width: c_uint,
    height: c_uint,
}

impl CImage {
    pub fn new() -> Self {
        Self {
            data: std::ptr::null(),
            width: 0,
            height: 0,
        }
    }

    pub fn size(&self) -> usize {
        (self.width * self.height * 4) as usize
    }

    pub fn data(&self) -> &[u8] {
        unsafe { from_raw_parts(self.data, self.size()) }
    }
}

pub struct ScreenCaptureX11 {
    handle: *mut c_void,
    // keep a reference to the capturable so it is not destroyed until we are done
    #[allow(dead_code)]
    capturable: X11Capturable,
    img: CImage,
    capture_cursor: bool,
}

impl ScreenCaptureX11 {
    pub fn new(mut capturable: X11Capturable, capture_cursor: bool) -> Result<Self, CError> {
        let mut err = CError::new();
        fltk::app::lock().unwrap();
        let handle = unsafe { start_capture(capturable.handle(), std::ptr::null_mut(), &mut err) };
        fltk::app::unlock();
        if err.is_err() {
            Err(err)
        } else {
            Ok(Self {
                handle,
                capturable,
                img: CImage::new(),
                capture_cursor,
            })
        }
    }
}

impl Drop for ScreenCaptureX11 {
    fn drop(&mut self) {
        let mut err = CError::new();
        fltk::app::lock().unwrap();
        unsafe {
            stop_capture(self.handle, &mut err);
        }
        fltk::app::unlock();
    }
}

impl ScreenCapture for ScreenCaptureX11 {
    fn capture(&mut self) -> Result<(), Box<dyn Error>> {
        let mut err = CError::new();
        fltk::app::lock().unwrap();
        unsafe {
            capture_sceen(
                self.handle,
                &mut self.img,
                self.capture_cursor.into(),
                &mut err,
            );
        }
        fltk::app::unlock();
        if err.is_err() {
            self.img.data = std::ptr::null();
            Err(err.into())
        } else {
            Ok(())
        }
    }

    fn pixel_provider(&self) -> crate::video::PixelProvider {
        if self.img.data.is_null() {
            PixelProvider::None
        } else {
            PixelProvider::BGR0(self.img.data())
        }
    }

    fn size(&self) -> (usize, usize) {
        (self.img.width as usize, self.img.height as usize)
    }
}
