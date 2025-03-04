/*
 * SPDX-FileCopyrightText: 2023 Andrew Gunnerson
 * SPDX-License-Identifier: GPL-3.0-only
 */

pub mod args;
pub mod avb;
pub mod boot;
pub mod completion;
pub mod cpio;
pub mod fec;
pub mod hashtree;
pub mod key;
pub mod ota;

macro_rules! status {
    ($($arg:tt)*) => {
        eprintln!("\x1b[1m[*] {}\x1b[0m", format!($($arg)*))
    }
}

macro_rules! warning {
    ($($arg:tt)*) => {
        eprintln!("\x1b[1;31m[WARNING] {}\x1b[0m", format!($($arg)*))
    }
}

pub(crate) use status;
pub(crate) use warning;
