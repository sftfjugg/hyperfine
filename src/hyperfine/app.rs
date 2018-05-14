use clap::{App, AppSettings, Arg, ArgMatches};
use std::ffi::OsString;
use atty;
use atty::Stream;

pub fn get_arg_matches<T>(args: T) -> ArgMatches<'static>
where
    T: Iterator<Item = OsString>,
{
    let app = build_app();
    app.get_matches_from(args)
}

/// Build the clap app for parsing command line arguments
fn build_app() -> App<'static, 'static> {
    let clap_color_setting = if atty::is(Stream::Stdout) {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };

    App::new("hyperfine")
        .version(crate_version!())
        .setting(clap_color_setting)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::NextLineHelp)
        .setting(AppSettings::HidePossibleValuesInHelp)
        .max_term_width(90)
        .about("A command-line benchmarking tool.")
        .arg(
            Arg::with_name("command")
                .help("Command to benchmark")
                .required(true)
                .multiple(true)
                .empty_values(false),
        )
        .arg(
            Arg::with_name("warmup")
                .long("warmup")
                .short("w")
                .takes_value(true)
                .value_name("NUM")
                .help(
                    "Perform NUM warmup runs before the actual benchmark. This can be used \
                     to fill (disk) caches for I/O-heavy programs.",
                ),
        )
        .arg(
            Arg::with_name("min-runs")
                .long("min-runs")
                .short("m")
                .takes_value(true)
                .value_name("NUM")
                .help("Perform at least NUM runs for each command (default: 10)."),
        )
        .arg(
            Arg::with_name("prepare")
                .long("prepare")
                .short("p")
                .takes_value(true)
                .value_name("CMD")
                .help(
                    "Execute CMD before each timing run. This is useful for \
                     clearing disk caches, for example.",
                ),
        )
        .arg(
            Arg::with_name("parameter-scan")
                .long("parameter-scan")
                .short("P")
                .help(
                    "Perform benchmark runs for each value in the range MIN..MAX. Replaces the \
                     string '{VAR}' in each command by the current parameter value.",
                )
                .takes_value(true)
                .allow_hyphen_values(true)
                .value_names(&["VAR", "MIN", "MAX"]),
        )
        .arg(
            Arg::with_name("shell")
                .long("shell")
                .short("S")
                .takes_value(true)
                .value_name("SHELL")
                .help(
                    "Specifies the shell environment to set up under which the benchmarked \
                     commands will bexe executed.",
                )
        )
        .arg(
            Arg::with_name("style")
                .long("style")
                .short("s")
                .takes_value(true)
                .value_name("TYPE")
                .possible_values(&["auto", "basic", "full", "nocolor"])
                .help(
                    "Set output style type (default: auto). Set this to 'basic' to disable output \
                     coloring and interactive elements. Set it to 'full' to enable all effects \
                     even if no interactive terminal was detected. Set this to 'nocolor' to \
                     keep the interactive output without any colors.",
                ),
        )
        .arg(
            Arg::with_name("ignore-failure")
                .long("ignore-failure")
                .short("i")
                .help("Ignore non-zero exit codes."),
        )
        .arg(
            Arg::with_name("export-csv")
                .long("export-csv")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing results as CSV to the given FILE."),
        )
        .arg(
            Arg::with_name("export-json")
                .long("export-json")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing results as JSON to the given FILE."),
        )
        .arg(
            Arg::with_name("export-markdown")
                .long("export-markdown")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing results as a Markdown table to the given FILE."),
        )
        .help_message("Print this help message.")
        .version_message("Show version information.")
}
