/*
 * SPDX-FileCopyrightText: 2023 Andrew Gunnerson
 * SPDX-License-Identifier: GPL-3.0-only
 */

use std::{
    fmt, io,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use tracing::debug;
use tracing_subscriber::{
    filter::Directive,
    fmt::{format::Writer, time::FormatTime},
    EnvFilter,
};

use crate::cli::{avb, boot, completion, cpio, fec, hashtree, key, ota};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub enum Command {
    Avb(avb::AvbCli),
    Boot(boot::BootCli),
    Completion(completion::CompletionCli),
    Cpio(cpio::CpioCli),
    Fec(fec::FecCli),
    HashTree(hashtree::HashTreeCli),
    Key(key::KeyCli),
    Ota(ota::OtaCli),
    /// (Deprecated: Use `avbroot ota patch` instead.)
    Patch(ota::PatchCli),
    /// (Deprecated: Use `avbroot ota extract` instead.)
    Extract(ota::ExtractCli),
    /// (Deprecated: Use `avbroot boot magisk-info` instead.)
    MagiskInfo(boot::MagiskInfoCli),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_possible_value().ok_or(fmt::Error)?.get_name())
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogFormat {
    Short,
    Medium,
    Long,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Short
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_possible_value().ok_or(fmt::Error)?.get_name())
    }
}

#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Lowest log message severity to output.
    #[arg(long, global = true, value_name = "LEVEL", default_value_t)]
    pub log_level: LogLevel,

    /// Output format for log messages.
    #[arg(long, global = true, value_name = "FORMAT", default_value_t)]
    pub log_format: LogFormat,
}

#[derive(Debug, Clone, Copy)]
pub struct ShortUptime {
    epoch: Instant,
}

impl Default for ShortUptime {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl FormatTime for ShortUptime {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        let e = self.epoch.elapsed();
        write!(w, "{:3}.{:03}s", e.as_secs(), e.subsec_millis())
    }
}

pub fn init_logging(default_directive: Directive, log_format: LogFormat) {
    let filter = EnvFilter::builder()
        .with_default_directive(default_directive)
        .from_env_lossy();

    let builder = tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(filter);

    match log_format {
        LogFormat::Short => {
            let format = tracing_subscriber::fmt::format()
                .with_timer(ShortUptime::default())
                .with_target(false);

            builder.event_format(format).init();
        }
        LogFormat::Medium => {
            builder.with_timer(ShortUptime::default()).init();
        }
        LogFormat::Long => {
            builder.pretty().init();
        }
        LogFormat::Json => {
            builder.json().with_current_span(false).init();
        }
    }
}

pub fn main(logging_initialized: &AtomicBool, cancel_signal: &AtomicBool) -> Result<()> {
    let cli = Cli::parse();

    let default_directive: Directive = format!(
        "{}={}",
        env!("CARGO_PKG_NAME").replace('-', "_"),
        cli.log_level.to_possible_value().unwrap().get_name(),
    )
    .parse()
    .expect("Broken hardcoded directive");

    init_logging(default_directive, cli.log_format);
    logging_initialized.store(true, Ordering::SeqCst);

    debug!(?cli);

    match cli.command {
        Command::Avb(c) => avb::avb_main(&c, cancel_signal),
        Command::Boot(c) => boot::boot_main(&c),
        Command::Completion(c) => completion::completion_main(&c),
        Command::Cpio(c) => cpio::cpio_main(&c, cancel_signal),
        Command::Fec(c) => fec::fec_main(&c, cancel_signal),
        Command::HashTree(c) => hashtree::hash_tree_main(&c, cancel_signal),
        Command::Key(c) => key::key_main(&c),
        Command::Ota(c) => ota::ota_main(&c, cancel_signal),
        // Deprecated aliases.
        Command::Patch(c) => ota::patch_subcommand(&c, cancel_signal),
        Command::Extract(c) => ota::extract_subcommand(&c, cancel_signal),
        Command::MagiskInfo(c) => boot::magisk_info_subcommand(&c),
    }
}
