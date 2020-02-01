use std::borrow::{Borrow, Cow};
use std::ffi::CStr;
use std::fmt;
use std::ops::{Deref, Index, RangeFull};
use std::os::raw::c_char;
use std::str;

#[macro_export]
macro_rules! im_str {
    ($e:tt) => ({
        unsafe {
          $crate::ImStr::from_utf8_with_nul_unchecked(concat!($e, "\0").as_bytes())
        }
    });
    ($e:tt, $($arg:tt)*) => ({
        unsafe {
          $crate::ImString::from_utf8_with_nul_unchecked(format!(concat!($e, "\0"), $($arg)*).into_bytes())
        }
    })
}

/// A UTF-8 encoded, growable, implicitly null-terminated string.
#[derive(Clone, Hash, Ord, Eq, PartialOrd, PartialEq)]
pub struct ImString(pub(crate) Vec<u8>);

impl ImString {
    /// Creates a new `ImString` from an existing string.
    pub fn new<T: Into<String>>(value: T) -> ImString {
        unsafe {
            let mut s = ImString::from_utf8_unchecked(value.into().into_bytes());
            s.refresh_len();
            s
        }
    }
    /// Creates a new empty `ImString` with a particular capacity
    pub fn with_capacity(capacity: usize) -> ImString {
        let mut v = Vec::with_capacity(capacity + 1);
        v.push(b'\0');
        ImString(v)
    }
    /// Converts a vector of bytes to a `ImString` without checking that the string contains valid
    /// UTF-8
    pub unsafe fn from_utf8_unchecked(mut v: Vec<u8>) -> ImString {
        v.push(b'\0');
        ImString(v)
    }
    /// Converts a vector of bytes to a `ImString` without checking that the string contains valid
    /// UTF-8
    pub unsafe fn from_utf8_with_nul_unchecked(v: Vec<u8>) -> ImString {
        ImString(v)
    }
    /// Truncates this `ImString`, removing all contents
    pub fn clear(&mut self) {
        self.0.clear();
        self.0.push(b'\0');
    }
    /// Appends the given character to the end of this `ImString`
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        self.push_str(ch.encode_utf8(&mut buf));
    }
    /// Appends a given string slice to the end of this `ImString`
    pub fn push_str(&mut self, string: &str) {
        self.0.pop();
        self.0.extend(string.bytes());
        self.0.push(b'\0');
        unsafe {
            self.refresh_len();
        }
    }
    /// Returns the capacity of this `ImString` in bytes
    pub fn capacity(&self) -> usize {
        self.0.capacity() - 1
    }
    /// Returns the capacity of this `ImString` in bytes, including the implicit null byte
    pub fn capacity_with_nul(&self) -> usize {
        self.0.capacity()
    }
    /// Ensures that the capacity of this `ImString` is at least `additional` bytes larger than the
    /// current length.
    ///
    /// The capacity may be increased by more than `additional` bytes.
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
    /// Ensures that the capacity of this `ImString` is at least `additional` bytes larger than the
    /// current length
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }
    /// Returns a raw pointer to the underlying buffer
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr() as *const c_char
    }
    /// Returns a raw mutable pointer to the underlying buffer.
    ///
    /// If the underlying data is modified, `refresh_len` *must* be called afterwards.
    pub fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr() as *mut c_char
    }
    /// Updates the underlying buffer length based on the current contents.
    ///
    /// This function *must* be called if the underlying data is modified via a pointer
    /// obtained by `as_mut_ptr`.
    pub unsafe fn refresh_len(&mut self) {
        let len = CStr::from_ptr(self.0.as_ptr() as *const c_char)
            .to_bytes_with_nul()
            .len();
        self.0.set_len(len);
    }
}

impl<'a> Default for ImString {
    fn default() -> ImString {
        ImString(vec![b'\0'])
    }
}

impl From<String> for ImString {
    fn from(s: String) -> ImString {
        ImString::new(s)
    }
}

impl<'a> From<ImString> for Cow<'a, ImStr> {
    fn from(s: ImString) -> Cow<'a, ImStr> {
        Cow::Owned(s)
    }
}

impl<'a> From<&'a ImString> for Cow<'a, ImStr> {
    fn from(s: &'a ImString) -> Cow<'a, ImStr> {
        Cow::Borrowed(s)
    }
}

impl<'a, T: ?Sized + AsRef<ImStr>> From<&'a T> for ImString {
    fn from(s: &'a T) -> ImString {
        s.as_ref().to_owned()
    }
}

impl AsRef<ImStr> for ImString {
    fn as_ref(&self) -> &ImStr {
        self
    }
}

impl Borrow<ImStr> for ImString {
    fn borrow(&self) -> &ImStr {
        self
    }
}

impl AsRef<str> for ImString {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl Borrow<str> for ImString {
    fn borrow(&self) -> &str {
        self.to_str()
    }
}

impl Index<RangeFull> for ImString {
    type Output = ImStr;
    fn index(&self, _index: RangeFull) -> &ImStr {
        self
    }
}

impl fmt::Debug for ImString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.to_str(), f)
    }
}

impl fmt::Display for ImString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.to_str(), f)
    }
}

impl Deref for ImString {
    type Target = ImStr;
    fn deref(&self) -> &ImStr {
        // as_ptr() is used, because we need to look at the bytes to figure out the length
        // self.0.len() is incorrect, because there might be more than one nul byte in the end, or
        // some interior nuls in the data
        unsafe {
            &*(CStr::from_ptr(self.0.as_ptr() as *const c_char) as *const CStr as *const ImStr)
        }
    }
}

impl fmt::Write for ImString {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c);
        Ok(())
    }
}

/// A UTF-8 encoded, implicitly null-terminated string slice.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImStr(CStr);

impl<'a> Default for &'a ImStr {
    fn default() -> &'a ImStr {
        static SLICE: &[u8] = &[0];
        unsafe { ImStr::from_utf8_with_nul_unchecked(SLICE) }
    }
}

impl fmt::Debug for ImStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for ImStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.to_str(), f)
    }
}

impl ImStr {
    /// Wraps a raw UTF-8 encoded C string
    pub unsafe fn from_ptr_unchecked<'a>(ptr: *const c_char) -> &'a ImStr {
        ImStr::from_cstr_unchecked(CStr::from_ptr(ptr))
    }
    /// Converts a slice of bytes to an imgui-rs string slice without checking for valid UTF-8 or
    /// null termination.
    pub unsafe fn from_utf8_with_nul_unchecked(bytes: &[u8]) -> &ImStr {
        &*(bytes as *const [u8] as *const ImStr)
    }
    /// Converts a CStr reference to an imgui-rs string slice without checking for valid UTF-8.
    pub unsafe fn from_cstr_unchecked(value: &CStr) -> &ImStr {
        &*(value as *const CStr as *const ImStr)
    }
    /// Converts an imgui-rs string slice to a raw pointer
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }
    /// Converts an imgui-rs string slice to a normal string slice
    pub fn to_str(&self) -> &str {
        // CStr::to_bytes does *not* include the null terminator
        unsafe { str::from_utf8_unchecked(self.0.to_bytes()) }
    }
    /// Returns true if the imgui-rs string slice is empty
    pub fn is_empty(&self) -> bool {
        self.0.to_bytes().is_empty()
    }
}

impl AsRef<CStr> for ImStr {
    fn as_ref(&self) -> &CStr {
        &self.0
    }
}

impl AsRef<ImStr> for ImStr {
    fn as_ref(&self) -> &ImStr {
        self
    }
}

impl AsRef<str> for ImStr {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl<'a> From<&'a ImStr> for Cow<'a, ImStr> {
    fn from(s: &'a ImStr) -> Cow<'a, ImStr> {
        Cow::Borrowed(s)
    }
}

impl ToOwned for ImStr {
    type Owned = ImString;
    fn to_owned(&self) -> ImString {
        ImString(self.0.to_owned().into_bytes())
    }
}

#[test]
fn test_imstring_constructors() {
    let s = ImString::new("test");
    assert_eq!(s.0, b"test\0");

    let s = ImString::with_capacity(100);
    assert_eq!(s.0, b"\0");

    let s = unsafe { ImString::from_utf8_unchecked(vec![b't', b'e', b's', b't']) };
    assert_eq!(s.0, b"test\0");

    let s = unsafe { ImString::from_utf8_with_nul_unchecked(vec![b't', b'e', b's', b't', b'\0']) };
    assert_eq!(s.0, b"test\0");
}

#[test]
fn test_imstring_operations() {
    let mut s = ImString::new("test");
    s.clear();
    assert_eq!(s.0, b"\0");
    s.push('z');
    assert_eq!(s.0, b"z\0");
    s.push('ä');
    assert_eq!(s.0, b"z\xc3\xa4\0");
    s.clear();
    s.push_str("imgui-rs");
    assert_eq!(s.0, b"imgui-rs\0");
    s.push_str("öä");
    assert_eq!(s.0, b"imgui-rs\xc3\xb6\xc3\xa4\0");
}

#[test]
fn test_imstring_fmt_write() {
    use std::fmt::Write;
    let mut s = ImString::default();
    let _ = write!(s, "format {:02x}", 0x42);
    assert_eq!(s.0, b"format 42\0");
}

#[test]
fn test_imstring_refresh_len() {
    let mut s = ImString::new("testing");
    unsafe {
        let mut ptr = s.as_mut_ptr() as *mut u8;
        ptr = ptr.wrapping_add(2);
        *ptr = b'z';
        ptr = ptr.wrapping_add(1);
        *ptr = b'\0';
    }
    assert_eq!(s.0, b"tez\0ing\0");
    unsafe { s.refresh_len() };
    assert_eq!(s.0, b"tez\0");
}

#[test]
fn test_imstring_interior_nul() {
    let s = ImString::new("test\0ohno");
    assert_eq!(s.0, b"test\0");
    assert_eq!(s.to_str(), "test");
    assert!(!s.is_empty());

    let s = ImString::new("\0ohno");
    assert_eq!(s.to_str(), "");
    assert!(s.is_empty());
}
