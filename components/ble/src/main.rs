//! @file
//! @brief BLE diagnostic and authenticated session command entry point.

use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod ownership;
mod probe;
mod state;
mod worker;

/// @brief Parse commands before acquiring any runtime or Bluetooth resources.
#[derive(Parser)]
#[command(
    version,
    about = "QuantumTooth BLE diagnostic and authenticated sessions"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// @brief Expose the public probe and authenticated session lifecycle commands.
#[derive(Subcommand)]
enum Command {
    /// @brief Run one bounded public-token transport test.
    #[command(about = "Run one unencrypted public-token write/indication test (30 s / 120 s)")]
    Probe,
    /// @brief Establish fresh keys, send one record and clean up.
    Send { text: String },
    /// @brief Retain worker-owned connection and keys between commands.
    Sendkeep { text: String },
    /// @brief Best-effort close followed by unconditional local teardown.
    Closekeep,
    /// @brief Internal on-demand owner; never a boot-enabled service.
    #[command(name = "__worker", hide = true)]
    Worker,
}

/// @brief Dispatch the explicit diagnostic command.
/// @return Success only after the requested operation completes.
fn main() -> ExitCode {
    let cli = Cli::parse();
    if !matches!(cli.command, Command::Probe) {
        return session_command(cli.command);
    }
    let started = std::time::Instant::now();
    // @note Keep ownership until after runtime shutdown closes the BlueZ connection.
    let _lock = match ownership::ProbeLock::acquire(std::path::Path::new(
        "/run/qtooth-ble-probe.lock",
    )) {
        Ok(lock) => lock,
        Err(_) => {
            eprintln!(
                "phase=lock error=probe_running_or_run_directory_not_writable transport_success=false"
            );
            return ExitCode::FAILURE;
        }
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => {
            eprintln!("phase=runtime error=cannot_start_runtime");
            return ExitCode::FAILURE;
        }
    };
    let result = runtime.block_on(async {
        let mut interrupt = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::interrupt(),
        )
        .map_err(|_| probe::Failure {
            phase: "signals",
            reason: "cannot_watch_sigint",
        })?;
        let mut terminate = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )
        .map_err(|_| probe::Failure {
            phase: "signals",
            reason: "cannot_watch_sigterm",
        })?;
        match cli.command {
            Command::Probe => {
                probe::run(async move {
                    tokio::select! { _ = interrupt.recv() => {}, _ = terminate.recv() => {} }
                })
                .await
            }
            _ => unreachable!("session commands dispatched before probe ownership"),
        }
    });
    // @note Drop all BlueR runtime tasks and close their bus connection before exit.
    runtime.shutdown_timeout(std::time::Duration::from_secs(1));
    match result {
        Ok(()) => {
            println!(
                "phase=complete transport_success=true authenticated=false elapsed_ms={}",
                started.elapsed().as_millis()
            );
            ExitCode::SUCCESS
        }
        Err(failure) => {
            eprintln!(
                "phase={} error={} transport_success=false elapsed_ms={}",
                failure.phase,
                failure.reason,
                started.elapsed().as_millis()
            );
            ExitCode::FAILURE
        }
    }
}

/// @brief Dispatch validated session commands and bound cancellation before resource acquisition.
fn session_command(command: Command) -> ExitCode {
    let data = match &command {
        Command::Send { text } => Some((worker::Op::Send, text.as_str())),
        Command::Sendkeep { text } => Some((worker::Op::Keep, text.as_str())),
        Command::Closekeep => Some((worker::Op::Close, "")),
        _ => None,
    };
    if let Some((op, text)) = data {
        if let Err(e) = worker::validate(op, text) {
            eprintln!("phase=input error={e}");
            return ExitCode::FAILURE;
        }
        if op != worker::Op::Close && !worker::provisioned() {
            eprintln!("phase=provisioning error=pair_not_provisioned");
            return ExitCode::FAILURE;
        }
    }
    let owner = if matches!(command, Command::Worker) {
        match ownership::ProbeLock::acquire_as(&worker::Paths::production().lock, 0) {
            Ok(l) => Some(l),
            Err(_) => return ExitCode::FAILURE,
        }
    } else {
        None
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(_) => return ExitCode::FAILURE,
    };
    let result = runtime.block_on(async {
        let stop = worker::cancellation()?;
        if matches!(command, Command::Worker) {
            worker::serve(
                worker::Paths::production(),
                worker::pair()?,
                worker::Ble::new(),
                stop,
                owner.as_ref().unwrap(),
            )
            .await
            .map(|_| worker::Outcome::Closed)
        } else {
            let (op, text) = data.ok_or("invalid_command")?;
            tokio::select! {biased;_=stop=>Err("cancelled"),r=worker::client(op,text)=>r}
        }
    });
    runtime.shutdown_timeout(std::time::Duration::ZERO);
    match result {
        Ok(o) => {
            println!("{}", o.report());
            if !o.succeeded() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("phase=session error={e} delivery=unconfirmed");
            ExitCode::FAILURE
        }
    }
}
