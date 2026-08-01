use std::io::{self, IsTerminal};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::get_limits::SourcePlan;
use crate::infra::loader::{
    loader_show_delay, loader_tick, LoaderView, TerminalStatus, TerminalUi,
};
use crate::presentation::ColorConfig;
use crate::types::SourceReport;

use super::args::OutputMode;
use super::render::{failed_source_block, print_source_report};

struct RunningSource {
    label: &'static str,
    started_at: Instant,
    loader_shown: bool,
    loader_frame: usize,
}

struct SourceEvent {
    label: &'static str,
    result: io::Result<SourceReport>,
}

pub(super) fn run_sources_with_terminal_ui(
    ui: &mut TerminalUi,
    plan: &[SourcePlan],
    output_mode: OutputMode,
) -> io::Result<TerminalStatus> {
    if plan.is_empty() {
        return Ok(TerminalStatus::Fail);
    }

    let color = ColorConfig::from_env(io::stdout().is_terminal());
    let (sender, receiver) = mpsc::channel::<SourceEvent>();
    let mut running = Vec::new();

    for target in plan {
        let target = *target;
        let label = target.label();
        let sender = sender.clone();
        running.push(RunningSource {
            label,
            started_at: Instant::now(),
            loader_shown: false,
            loader_frame: 0,
        });
        thread::spawn(move || {
            let result = crate::get_limits::get_source_plan_limits(target);
            let _ = sender.send(SourceEvent { label, result });
        });
    }
    drop(sender);

    let mut successes = 0_usize;
    let mut failures = 0_usize;
    let mut stderr = String::new();

    while !running.is_empty() {
        render_running_loaders(ui, &mut running)?;

        match receiver.recv_timeout(loader_tick()) {
            Ok(event) => {
                if let Some(index) = running
                    .iter()
                    .position(|running| running.label == event.label)
                {
                    running.remove(index);
                }

                match event.result {
                    Ok(report) => {
                        successes += 1;
                        stderr.push_str(&report.data.stderr);
                        print_source_report(ui, &report, output_mode, &color)?;
                    }
                    Err(error) => {
                        failures += 1;
                        let block = failed_source_block(event.label, &error.to_string());
                        ui.print_provider_block(&block.provider_label, &block.body)?;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    ui.finish_loaders()?;

    if !stderr.trim().is_empty() {
        eprint!("{stderr}");
    }

    Ok(match (successes, failures) {
        (_, 0) if successes > 0 => TerminalStatus::Done,
        (0, _) => TerminalStatus::Fail,
        _ => TerminalStatus::Part,
    })
}

fn render_running_loaders(ui: &mut TerminalUi, running: &mut [RunningSource]) -> io::Result<()> {
    for running in running.iter_mut() {
        if running.started_at.elapsed() >= loader_show_delay() {
            running.loader_shown = true;
        }
        if running.loader_shown {
            running.loader_frame = running.loader_frame.wrapping_add(1);
        }
    }

    let loaders = running
        .iter()
        .filter(|running| running.loader_shown)
        .map(|running| LoaderView {
            label: running.label,
            frame: running.loader_frame,
        })
        .collect::<Vec<_>>();

    if loaders.is_empty() {
        return Ok(());
    }

    ui.render_loaders(&loaders)
}
