use std::{borrow::Cow, ffi::OsStr, sync::Arc};

pub trait CommandExt {
    /// Adds an argument if `condition` otherwise noop.
    fn arg_if(&mut self, condition: bool, a: impl ArgString) -> &mut Self;

    /// Adds two arguments.
    fn arg2(&mut self, a: impl ArgString, b: impl ArgString) -> &mut Self;

    /// Adds two arguments, the 2nd an option. `None` mean noop.
    fn arg2_opt(&mut self, a: impl ArgString, b: Option<impl ArgString>) -> &mut Self;

    /// Adds two arguments if `condition` otherwise noop.
    fn arg2_if(&mut self, condition: bool, a: impl ArgString, b: impl ArgString) -> &mut Self;
}

impl CommandExt for std::process::Command {
    fn arg_if(&mut self, c: bool, arg: impl ArgString) -> &mut Self {
        match c {
            true => self.arg(arg.arg_string()),
            false => self,
        }
    }

    fn arg2(&mut self, a: impl ArgString, b: impl ArgString) -> &mut Self {
        self.arg(a.arg_string()).arg(b.arg_string())
    }

    fn arg2_opt(&mut self, a: impl ArgString, b: Option<impl ArgString>) -> &mut Self {
        match b {
            Some(b) => self.arg2(a, b),
            None => self,
        }
    }

    fn arg2_if(&mut self, c: bool, a: impl ArgString, b: impl ArgString) -> &mut Self {
        match c {
            true => self.arg2(a, b),
            false => self,
        }
    }
}

pub trait ArgString {
    fn arg_string(&self) -> Cow<'_, OsStr>;
}

macro_rules! impl_arg_string_as_ref {
    ($t:ty) => {
        impl ArgString for $t {
            fn arg_string(&self) -> Cow<'_, OsStr> {
                Cow::Borrowed(self.as_ref())
            }
        }
    };
}
impl_arg_string_as_ref!(String);
impl_arg_string_as_ref!(&'_ String);
impl_arg_string_as_ref!(&'_ str);
impl_arg_string_as_ref!(&'_ &'_ str);
impl_arg_string_as_ref!(&'_ std::path::Path);
impl_arg_string_as_ref!(&'_ std::path::PathBuf);
impl_arg_string_as_ref!(std::path::PathBuf);

macro_rules! impl_arg_string_display {
    ($t:ty) => {
        impl ArgString for $t {
            fn arg_string(&self) -> Cow<'_, OsStr> {
                Cow::Owned(self.to_string().into())
            }
        }
    };
}
impl_arg_string_display!(u8);
impl_arg_string_display!(&u8);
impl_arg_string_display!(u16);
impl_arg_string_display!(&u16);
impl_arg_string_display!(u32);
impl_arg_string_display!(&u32);
impl_arg_string_display!(i32);
impl_arg_string_display!(&i32);
impl_arg_string_display!(f32);
impl_arg_string_display!(&f32);

impl ArgString for Arc<str> {
    fn arg_string(&self) -> Cow<'_, OsStr> {
        Cow::Borrowed((**self).as_ref())
    }
}
