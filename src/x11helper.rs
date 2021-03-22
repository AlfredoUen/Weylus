use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::sync::Arc;

use tracing::debug;

use crate::cerror::CError;

extern "C" {
    fn XOpenDisplay(name: *const c_char) -> *mut c_void;
    fn XCloseDisplay(disp: *mut c_void) -> c_int;

    fn create_capturables(
        disp: *mut c_void,
        handles: *mut *mut c_void,
        size: c_int,
        err: *mut CError,
    ) -> c_int;

    fn clone_capturable(handle: *const c_void) -> *mut c_void;
    fn destroy_capturable(handle: *mut c_void);
    fn get_capturable_name(handle: *const c_void) -> *const c_char;
    fn capturable_before_input(handle: *mut c_void, err: *mut CError);
    fn get_geometry_relative(
        handle: *const c_void,
        x: *mut c_float,
        y: *mut c_float,
        width: *mut c_float,
        height: *mut c_float,
        err: *mut CError,
    );

    fn map_input_device_to_entire_screen(
        disp: *mut c_void,
        device_name: *const c_char,
        libinput: c_int,
        err: *mut CError,
    );
}

pub struct X11Capturable {
    handle: *mut c_void,
    // keep a reference to the display so it is not closed while a capturable still exists
    disp: Arc<XDisplay>,
}

impl Clone for X11Capturable {
    fn clone(&self) -> Self {
        let handle = unsafe { clone_capturable(self.handle) };
        Self {
            handle,
            disp: self.disp.clone(),
        }
    }
}

unsafe impl Send for X11Capturable {}

impl X11Capturable {
    pub unsafe fn handle(&mut self) -> *mut c_void {
        self.handle
    }

    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(get_capturable_name(self.handle))
                .to_string_lossy()
                .into()
        }
    }

    pub fn geometry(&self) -> Result<CaptureGeometry, CError> {
        let mut x: c_float = 0.0;
        let mut y: c_float = 0.0;
        let mut width: c_float = 0.0;
        let mut height: c_float = 0.0;
        let mut err = CError::new();
        fltk::app::lock().unwrap();
        unsafe {
            get_geometry_relative(
                self.handle,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
                &mut err,
            );
        }
        fltk::app::unlock();
        if err.is_err() {
            return Err(err);
        }
        Ok(CaptureGeometry {
            x: x.into(),
            y: y.into(),
            width: width.into(),
            height: height.into(),
        })
    }

    pub fn before_input(&mut self) -> Result<(), CError> {
        let mut err = CError::new();
        fltk::app::lock().unwrap();
        unsafe { capturable_before_input(self.handle, &mut err) };
        fltk::app::unlock();
        if err.is_err() {
            Err(err)
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for X11Capturable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Drop for X11Capturable {
    fn drop(&mut self) {
        unsafe {
            destroy_capturable(self.handle);
        }
    }
}

pub struct CaptureGeometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

struct XDisplay {
    handle: *mut c_void,
}

impl XDisplay {
    pub fn new() -> Option<Self> {
        let handle = unsafe { XOpenDisplay(std::ptr::null()) };
        if handle.is_null() {
            return None;
        }
        Some(Self { handle })
    }
}

impl Drop for XDisplay {
    fn drop(&mut self) {
        fltk::app::lock().unwrap();
        unsafe { XCloseDisplay(self.handle) };
        fltk::app::unlock();
    }
}

pub struct X11Context {
    disp: Arc<XDisplay>,
}

impl X11Context {
    pub fn new() -> Option<Self> {
        let disp = XDisplay::new()?;
        Some(Self {
            disp: Arc::new(disp),
        })
    }

    pub fn capturables(&mut self) -> Result<Vec<X11Capturable>, CError> {
        let mut err = CError::new();
        let mut handles = [std::ptr::null_mut::<c_void>(); 128];
        fltk::app::lock().unwrap();
        let size = unsafe {
            create_capturables(
                self.disp.handle,
                handles.as_mut_ptr(),
                handles.len() as c_int,
                &mut err,
            )
        };
        fltk::app::unlock();
        if err.is_err() {
            if err.code() == 2 {
                debug!("{}", err);
            } else {
                return Err(err);
            }
        }
        Ok(handles[0..size as usize]
            .iter()
            .map(|handle| X11Capturable {
                handle: *handle,
                disp: self.disp.clone(),
            })
            .collect::<Vec<X11Capturable>>())
    }

    pub fn map_input_device_to_entire_screen(&mut self, device_name: &str, pen: bool) -> CError {
        fltk::app::lock().unwrap();
        let mut err = CError::new();
        let device_name_c_str = CString::new(device_name).unwrap();
        unsafe {
            map_input_device_to_entire_screen(
                self.disp.handle,
                device_name_c_str.as_ptr(),
                pen.into(),
                &mut err,
            )
        };
        fltk::app::unlock();
        if err.is_err() {
            debug!("Failed to map input device to screen: {}", &err);
        }
        err
    }
}
