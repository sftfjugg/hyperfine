use std::cmp;
use std::io;
use std::process::{ExitStatus, Stdio};

use colored::*;
use statistical::{mean, median, standard_deviation};

use crate::benchmark_result::BenchmarkResult;
use crate::command::Command;
use crate::format::{format_duration, format_duration_unit};
use crate::min_max::{max, min};
use crate::options::{CmdFailureAction, HyperfineOptions, OutputStyleOption};
use crate::outlier_detection::{modified_zscores, OUTLIER_THRESHOLD};
use crate::progress_bar::get_progress_bar;
use crate::shell::execute_and_time;
use crate::timer::wallclocktimer::WallClockTimer;
use crate::timer::{TimerStart, TimerStop};
use crate::units::Second;
use crate::warnings::Warnings;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Second = 5e-3;

/// Results from timing a single shell command
#[derive(Debug, Default, Copy, Clone)]
pub struct TimingResult {
    /// Wall clock time
    pub time_real: Second,

    /// Time spent in user mode
    pub time_user: Second,

    /// Time spent in kernel mode
    pub time_system: Second,
}

/// Correct for shell spawning time
fn subtract_shell_spawning_time(time: Second, shell_spawning_time: Second) -> Second {
    if time < shell_spawning_time {
        0.0
    } else {
        time - shell_spawning_time
    }
}

/// Run the given shell command and measure the execution time
pub fn time_shell_command(
    shell: &str,
    command: &Command<'_>,
    show_output: bool,
    failure_action: CmdFailureAction,
    shell_spawning_time: Option<TimingResult>,
) -> io::Result<(TimingResult, ExitStatus)> {
    let (stdout, stderr) = if show_output {
        (Stdio::inherit(), Stdio::inherit())
    } else {
        (Stdio::null(), Stdio::null())
    };

    let wallclock_timer = WallClockTimer::start();
    let result = execute_and_time(stdout, stderr, &command.get_shell_command(), shell)?;
    let mut time_real = wallclock_timer.stop();

    let mut time_user = result.user_time;
    let mut time_system = result.system_time;

    if failure_action == CmdFailureAction::RaiseError && !result.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "{}. \
                Use the '-i'/'--ignore-failure' option if you want to ignore this. \
                Alternatively, use the '--show-output' option to debug what went wrong.",
                result.status.code().map_or(
                    "The process has been terminated by a signal".into(),
                    |c| format!("Command terminated with non-zero exit code: {}", c)
                )
            ),
        ));
    }

    // Correct for shell spawning time
    if let Some(spawning_time) = shell_spawning_time {
        time_real = subtract_shell_spawning_time(time_real, spawning_time.time_real);
        time_user = subtract_shell_spawning_time(time_user, spawning_time.time_user);
        time_system = subtract_shell_spawning_time(time_system, spawning_time.time_system);
    }

    Ok((
        TimingResult {
            time_real,
            time_user,
            time_system,
        },
        result.status,
    ))
}

/// Measure the average shell spawning time
pub fn mean_shell_spawning_time(
    shell: &str,
    style: OutputStyleOption,
    show_output: bool,
) -> io::Result<TimingResult> {
    const COUNT: u64 = 50;
    let progress_bar = if style != OutputStyleOption::Disabled {
        Some(get_progress_bar(
            COUNT,
            "Measuring shell spawning time",
            style,
        ))
    } else {
        None
    };

    let mut times_real: Vec<Second> = vec![];
    let mut times_user: Vec<Second> = vec![];
    let mut times_system: Vec<Second> = vec![];

    for _ in 0..COUNT {
        // Just run the shell without any command
        let res = time_shell_command(
            shell,
            &Command::new(None, ""),
            show_output,
            CmdFailureAction::RaiseError,
            None,
        );

        match res {
            Err(_) => {
                let shell_cmd = if cfg!(windows) {
                    format!("{} /C \"\"", shell)
                } else {
                    format!("{} -c \"\"", shell)
                };

                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Could not measure shell execution time. \
                         Make sure you can run '{}'.",
                        shell_cmd
                    ),
                ));
            }
            Ok((r, _)) => {
                times_real.push(r.time_real);
                times_user.push(r.time_user);
                times_system.push(r.time_system);
            }
        }

        if let Some(bar) = progress_bar.as_ref() {
            bar.inc(1)
        }
    }

    if let Some(bar) = progress_bar.as_ref() {
        bar.finish_and_clear()
    }

    Ok(TimingResult {
        time_real: mean(&times_real),
        time_user: mean(&times_user),
        time_system: mean(&times_system),
    })
}

fn run_intermediate_command(
    shell: &str,
    command: &Option<Command<'_>>,
    show_output: bool,
    error_output: &'static str,
) -> io::Result<TimingResult> {
    if let Some(ref cmd) = command {
        let res = time_shell_command(shell, cmd, show_output, CmdFailureAction::RaiseError, None);
        if res.is_err() {
            return Err(io::Error::new(io::ErrorKind::Other, error_output));
        }
        return res.map(|r| r.0);
    }
    Ok(TimingResult {
        ..Default::default()
    })
}

/// Run the command specified by `--prepare`.
fn run_preparation_command(
    shell: &str,
    command: &Option<Command<'_>>,
    show_output: bool,
) -> io::Result<TimingResult> {
    let error_output = "The preparation command terminated with a non-zero exit code. \
                        Append ' || true' to the command if you are sure that this can be ignored.";

    run_intermediate_command(shell, command, show_output, error_output)
}

/// Run the command specified by `--cleanup`.
fn run_cleanup_command(
    shell: &str,
    command: &Option<Command<'_>>,
    show_output: bool,
) -> io::Result<TimingResult> {
    let error_output = "The cleanup command terminated with a non-zero exit code. \
                        Append ' || true' to the command if you are sure that this can be ignored.";

    run_intermediate_command(shell, command, show_output, error_output)
}

#[cfg(unix)]
fn extract_exit_code(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;

    /* From the ExitStatus::code documentation:
       "On Unix, this will return None if the process was terminated by a signal."
       In that case, ExitStatusExt::signal should never return None.
    */
    status.code().or_else(||
        /* To differentiate between "normal" exit codes and signals, we are using
           something similar to bash exit codes (https://tldp.org/LDP/abs/html/exitcodes.html)
           by adding 128 to a signal integer value.
         */
        status.signal().map(|s| 128 + s))
}

#[cfg(not(unix))]
fn extract_exit_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

/// Run the benchmark for a single shell command
pub fn run_benchmark(
    num: usize,
    cmd: &Command<'_>,
    shell_spawning_time: TimingResult,
    options: &HyperfineOptions,
) -> io::Result<BenchmarkResult> {
    let command_name = cmd.get_name();
    if options.output_style != OutputStyleOption::Disabled {
        println!(
            "{}{}: {}",
            "Benchmark ".bold(),
            (num + 1).to_string().bold(),
            command_name,
        );
    }

    let mut times_real: Vec<Second> = vec![];
    let mut times_user: Vec<Second> = vec![];
    let mut times_system: Vec<Second> = vec![];
    let mut exit_codes: Vec<Option<i32>> = vec![];
    let mut all_succeeded = true;

    // Run init command
    let prepare_cmd = options.preparation_command.as_ref().map(|values| {
        let preparation_command = if values.len() == 1 {
            &values[0]
        } else {
            &values[num]
        };
        Command::new_parametrized(None, preparation_command, cmd.get_parameters().clone())
    });

    // Warmup phase
    if options.warmup_count > 0 {
        let progress_bar = if options.output_style != OutputStyleOption::Disabled {
            Some(get_progress_bar(
                options.warmup_count,
                "Performing warmup runs",
                options.output_style,
            ))
        } else {
            None
        };

        for _ in 0..options.warmup_count {
            let _ = run_preparation_command(&options.shell, &prepare_cmd, options.show_output)?;
            let _ = time_shell_command(
                &options.shell,
                cmd,
                options.show_output,
                options.failure_action,
                None,
            )?;
            if let Some(bar) = progress_bar.as_ref() {
                bar.inc(1)
            }
        }
        if let Some(bar) = progress_bar.as_ref() {
            bar.finish_and_clear()
        }
    }

    // Set up progress bar (and spinner for initial measurement)
    let progress_bar = if options.output_style != OutputStyleOption::Disabled {
        Some(get_progress_bar(
            options.runs.min,
            "Initial time measurement",
            options.output_style,
        ))
    } else {
        None
    };

    let prepare_res = run_preparation_command(&options.shell, &prepare_cmd, options.show_output)?;

    // Initial timing run
    let (res, status) = time_shell_command(
        &options.shell,
        cmd,
        options.show_output,
        options.failure_action,
        Some(shell_spawning_time),
    )?;
    let success = status.success();

    // Determine number of benchmark runs
    let runs_in_min_time = (options.min_time_sec
        / (res.time_real + prepare_res.time_real + shell_spawning_time.time_real))
        as u64;

    let count = {
        let min = cmp::max(runs_in_min_time, options.runs.min);

        options
            .runs
            .max
            .as_ref()
            .map(|max| cmp::min(min, *max))
            .unwrap_or(min)
    };

    let count_remaining = count - 1;

    // Save the first result
    times_real.push(res.time_real);
    times_user.push(res.time_user);
    times_system.push(res.time_system);
    exit_codes.push(extract_exit_code(status));

    all_succeeded = all_succeeded && success;

    // Re-configure the progress bar
    if let Some(bar) = progress_bar.as_ref() {
        bar.set_length(count)
    }
    if let Some(bar) = progress_bar.as_ref() {
        bar.inc(1)
    }

    // Gather statistics
    for _ in 0..count_remaining {
        run_preparation_command(&options.shell, &prepare_cmd, options.show_output)?;

        let msg = {
            let mean = format_duration(mean(&times_real), options.time_unit);
            format!("Current estimate: {}", mean.to_string().green())
        };

        if let Some(bar) = progress_bar.as_ref() {
            bar.set_message(msg.to_owned())
        }

        let (res, status) = time_shell_command(
            &options.shell,
            cmd,
            options.show_output,
            options.failure_action,
            Some(shell_spawning_time),
        )?;
        let success = status.success();

        times_real.push(res.time_real);
        times_user.push(res.time_user);
        times_system.push(res.time_system);
        exit_codes.push(extract_exit_code(status));

        all_succeeded = all_succeeded && success;

        if let Some(bar) = progress_bar.as_ref() {
            bar.inc(1)
        }
    }

    if let Some(bar) = progress_bar.as_ref() {
        bar.finish_and_clear()
    }

    // Compute statistical quantities
    let t_num = times_real.len();
    let t_mean = mean(&times_real);
    let t_stddev = standard_deviation(&times_real, Some(t_mean));
    let t_median = median(&times_real);
    let t_min = min(&times_real);
    let t_max = max(&times_real);

    let user_mean = mean(&times_user);
    let system_mean = mean(&times_system);

    // Formatting and console output
    let (mean_str, time_unit) = format_duration_unit(t_mean, options.time_unit);
    let stddev_str = format_duration(t_stddev, Some(time_unit));
    let min_str = format_duration(t_min, Some(time_unit));
    let max_str = format_duration(t_max, Some(time_unit));
    let num_str = format!("{} runs", t_num);

    let user_str = format_duration(user_mean, Some(time_unit));
    let system_str = format_duration(system_mean, Some(time_unit));

    if options.output_style != OutputStyleOption::Disabled {
        println!(
            "  Time ({} ± {}):     {:>8} ± {:>8}    [User: {}, System: {}]",
            "mean".green().bold(),
            "σ".green(),
            mean_str.green().bold(),
            stddev_str.green(),
            user_str.blue(),
            system_str.blue()
        );

        println!(
            "  Range ({} … {}):   {:>8} … {:>8}    {}",
            "min".cyan(),
            "max".purple(),
            min_str.cyan(),
            max_str.purple(),
            num_str.dimmed()
        );
    }

    // Warnings
    let mut warnings = vec![];

    // Check execution time
    if times_real.iter().any(|&t| t < MIN_EXECUTION_TIME) {
        warnings.push(Warnings::FastExecutionTime);
    }

    // Check programm exit codes
    if !all_succeeded {
        warnings.push(Warnings::NonZeroExitCode);
    }

    // Run outlier detection
    let scores = modified_zscores(&times_real);
    if scores[0] > OUTLIER_THRESHOLD {
        warnings.push(Warnings::SlowInitialRun(times_real[0]));
    } else if scores.iter().any(|&s| s.abs() > OUTLIER_THRESHOLD) {
        warnings.push(Warnings::OutliersDetected);
    }

    if !warnings.is_empty() {
        eprintln!(" ");

        for warning in &warnings {
            eprintln!("  {}: {}", "Warning".yellow(), warning);
        }
    }

    if options.output_style != OutputStyleOption::Disabled {
        println!(" ");
    }

    // Run cleanup command
    let cleanup_cmd = options.cleanup_command.as_ref().map(|cleanup_command| {
        Command::new_parametrized(None, cleanup_command, cmd.get_parameters().clone())
    });
    run_cleanup_command(&options.shell, &cleanup_cmd, options.show_output)?;

    Ok(BenchmarkResult::new(
        command_name,
        t_mean,
        t_stddev,
        t_median,
        user_mean,
        system_mean,
        t_min,
        t_max,
        times_real,
        exit_codes,
        cmd.get_parameters()
            .iter()
            .map(|(name, value)| ((*name).to_string(), value.to_string()))
            .collect(),
    ))
}
