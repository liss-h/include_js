#![feature(const_fn_transmute)]

use std::{borrow::Borrow, ops::Deref};
use std::convert::TryFrom;

pub type JSParseError = boa::syntax::parser::ParseError;

/// Wrapper around `str` that ensures it contains _syntactically_ valid Javascript.
/// This is the borrowed version of `JSString` so `&JSStr` is to `JSString` what `&str` is to `String`
#[repr(transparent)]
pub struct JSStr {
    data: str,
}

/// Wrapper around `String` that ensures it contains _syntactically_ valid Javascript.
/// See docs for `JSStr` for more info.
pub struct JSString {
    code: String,
}

pub trait JSTemplate {
    fn render_template(&self) -> JSString;
}


impl JSStr {
    /// Checks if the content of `js` is syntactically valid Javascript before
    /// coersing it to `&JSStr`
    /// 
    /// # Examples
    ///
    /// ```rust
    /// use include_js::JSStr;
    /// 
    /// let js_str = JSStr::new("function f() {}");
    /// assert!(js_str.is_ok());
    /// ```
    /// 
    /// ```rust
    /// use include_js::JSStr;
    ///
    /// let js_str = JSStr::new("#include <vector>");
    /// assert!(js_str.is_err());
    /// ```
    pub fn new(js: &str) -> Result<&Self, JSParseError> {
        let _ = boa::parse(js, false)?;

        // SAFETY: follows from safety of `new_unchecked` and from the line above
        Ok(unsafe { JSStr::new_unchecked(js) })
    }

    /// Coerses `js` directly into a `&JSStr` without checking for validity
    pub const unsafe fn new_unchecked(js: &str) -> &Self {
        // SAFETY: JSStr is repr(transparent) and contains `str` so transmuting from &str to &JSStr is safe
        std::mem::transmute(js)
    }

    /// Converts the `&JSStr` back into an `&str`, this should be a noop.
    pub fn as_str(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<&'a str> for &'a JSStr {
    type Error = JSParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        JSStr::new(value)
    }
}

impl AsRef<JSStr> for JSStr {
    fn as_ref(&self) -> &JSStr {
        self
    }
}

impl AsRef<str> for JSStr {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> Into<&'a str> for &'a JSStr {
    fn into(self) -> &'a str {
        &self.data
    }
}

impl ToOwned for JSStr {
    type Owned = JSString;

    fn to_owned(&self) -> Self::Owned {
        // SAFETY: self.as_str() comes from JSStr so it must be valid javascript
        unsafe {
            JSString::new_unchecked(self.as_str().to_owned())
        }
    }
}

impl JSString {
    pub fn new(code: String) -> Result<Self, JSParseError> {
        let _ = JSStr::new(&code)?;
        Ok(JSString{ code })
    }

    pub unsafe fn new_unchecked(code: String) -> Self {
        JSString{ code }
    }

    pub fn into_string(self) -> String {
        self.code
    }
}

impl TryFrom<String> for JSString {
    type Error = JSParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        JSString::new(value)
    }
}

impl Into<String> for JSString {
    fn into(self) -> String {
        self.code
    }
}

impl Borrow<JSStr> for JSString {
    fn borrow(&self) -> &JSStr {
        // SAFETY: we are already in JSString so `code` must be valid javascript
        unsafe { JSStr::new_unchecked(&self.code) }
    }
}

impl AsRef<JSStr> for JSString {
    fn as_ref(&self) -> &JSStr {
        self.borrow()
    }
}

impl Deref for JSString {
    type Target = JSStr;

    fn deref(&self) -> &Self::Target {
        // SAFETY: we are already in JSString so `code` must be valid javascript
        unsafe { JSStr::new_unchecked(&self.code) }
    }
}
