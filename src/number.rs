// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Native scalar and typed float conversion on Linux GNU.
use crate::ErrorCode;

#[cfg(all(target_os = "linux", target_env = "gnu"))]
unsafe extern "C" {
    // libc exposes the locale types/lifecycle but not these GNU function declarations.
    fn strtod_l(
        text: *const libc::c_char,
        end: *mut *mut libc::c_char,
        locale: libc::locale_t,
    ) -> libc::c_double;
    fn strtof_l(
        text: *const libc::c_char,
        end: *mut *mut libc::c_char,
        locale: libc::locale_t,
    ) -> libc::c_float;
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(crate) fn scalar_float(text: &str) -> Result<f64, ErrorCode> {
    let previous_errno = errno::errno();
    let mut input = text.as_bytes().to_vec();
    input.push(0);
    // SAFETY: The static C locale name is NUL-terminated; a null base requests a
    // separately owned locale. No process/thread locale is changed.
    let locale =
        unsafe { libc::newlocale(libc::LC_NUMERIC_MASK, c"C".as_ptr(), std::ptr::null_mut()) };
    let result = if locale.is_null() {
        Err(ErrorCode::Resource)
    } else {
        errno::set_errno(errno::Errno(0));
        let mut end = std::ptr::null_mut();
        // SAFETY: input owns a live NUL-terminated byte buffer, end is writable,
        // and locale is a valid owned locale for this call. The C function only
        // reads input; no pointer escapes or end-pointer dereference occurs.
        let value = unsafe { strtod_l(input.as_ptr().cast(), &mut end, locale) };
        let conversion_errno = errno::errno();
        // SAFETY: Every successful newlocale result is released exactly once,
        // after its only conversion; the locale is never shared or reused.
        unsafe { libc::freelocale(locale) };
        if end.cast_const() == input.as_ptr().cast()
            || conversion_errno == errno::Errno(libc::ERANGE)
            || !value.is_finite()
        {
            Err(ErrorCode::InvalidValue)
        } else {
            Ok(value)
        }
    };
    drop(input);
    errno::set_errno(previous_errno);
    result
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub(crate) fn scalar_float(_: &str) -> Result<f64, ErrorCode> {
    Err(ErrorCode::Unsupported)
}

/// Native Param::Get<float> formats the stored double with 17 significant digits.
/// Different textual exponent notation is immaterial to the subsequent stof.
/// The parser supplies a finite double; native float conversion errors default zero.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(crate) fn stored_double_to_float(value: f64) -> Result<f32, ErrorCode> {
    let previous_errno = errno::errno();
    let input = format!("{value:.16e}");
    let result = match float(&input) {
        Err(ErrorCode::InvalidValue) => Ok(0.),
        result => result,
    };
    drop(input);
    errno::set_errno(previous_errno);
    result
}

/// std::stof semantics: prefix conversion and ERANGE, including nonfinite metadata.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(crate) fn float(text: &str) -> Result<f32, ErrorCode> {
    let previous_errno = errno::errno();
    let mut input = text.as_bytes().to_vec();
    input.push(0);
    // SAFETY: NUL-terminated static C name and null base request a fresh owned
    // locale. It never becomes a process or thread locale.
    let locale =
        unsafe { libc::newlocale(libc::LC_NUMERIC_MASK, c"C".as_ptr(), std::ptr::null_mut()) };
    let result = if locale.is_null() {
        Err(ErrorCode::Resource)
    } else {
        errno::set_errno(errno::Errno(0));
        let mut end = std::ptr::null_mut();
        // SAFETY: The owned NUL buffer and valid locale remain alive; end is
        // writable and only compared with input, never dereferenced or retained.
        let converted = unsafe { strtof_l(input.as_ptr().cast(), &mut end, locale) };
        let conversion_errno = errno::errno();
        // SAFETY: Release this successful allocation exactly once after use.
        unsafe { libc::freelocale(locale) };
        if end.cast_const() == input.as_ptr().cast()
            || conversion_errno == errno::Errno(libc::ERANGE)
        {
            Err(ErrorCode::InvalidValue)
        } else {
            Ok(converted)
        }
    };
    drop(input);
    errno::set_errno(previous_errno);
    result
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub(crate) fn stored_double_to_float(_: f64) -> Result<f32, ErrorCode> {
    Err(ErrorCode::Unsupported)
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub(crate) fn float(_: &str) -> Result<f32, ErrorCode> {
    Err(ErrorCode::Unsupported)
}
