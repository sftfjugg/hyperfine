# unreleased

## Features

## Changes

- When `--export-*` commands are used, result files are created before benchmark execution
  to fail early in case of, e.g., wrong permissions. See #306 (@s1ck). 
- When `--export-*` options are used, result files are written after each individual 
  benchmark command instead of writing after all benchmarks have finished. See #306 (@s1ck).

## Bugfixes

## Other

## Packaging



# v1.11.0

## Features

- The `-L`/`--parameter-list` option can now be specified multiple times to
  evaluate all possible combinations of the listed parameters:

  ``` bash
  hyperfine -L number 1,2 -L letter a,b,c \
      "echo {number}{letter}" \
      "printf '%s\n' {number}{letter}"
  # runs 12 benchmarks: 2 commands (echo and printf) times 6 combinations of
  # the "letter" and "number" parameters
  ```

  See: #253, #318 (@wchargin)

- Add CLI option to identify a command with a custom name, see #326 (@scampi)

## Changes

- When parameters are used with `--parameter-list` or `--parameter-scan`, the JSON export format
  now contains a dictionary `parameters` instead of a single key `parameter`. See #253, #318.
- The `plot_parametrized.py` script now infers the parameter name, and its `--parameter-name`
  argument has been deprecated. See #253, #318.

## Bugfixes

- Fix a bug in the outlier detection which would only detect "slow outliers" but not the fast
  ones (runs that are much faster than the rest of the benchmarking runs), see #329
- Better error messages for very fast commands that would lead to inf/nan results in the relative
  speed comparison, see #319
- Show error message if `--warmup` or `--*runs` arguments can not be parsed, see #337
- Keep output colorized when the output is not interactive and `--style=full` or `--style=color` is used.


# v1.10.0

## Features

- Hyperfine now comes with shell completion files for Bash, Zsh, Fish
  and PowerShell, see #290 (@four0000four).
- Hyperfine now comes with a basic man page, see #257 (@cadeef)
- During execution of benchmarks, hyperfine will now set a `HYPERFINE_RANDOMIZED_ENVIRONMENT_OFFSET` environment variable in order to randomize the memory layout. See #235 and #241 for references and details.
- A few enhancements for the histogram plotting scripts and the
  advanced statistics script
- Updates for the `plot_whisker.py` script, see #275 (@ghaiklor)

## Bugfixes

- Fix Spin Icon on Windows, see #229
- A few typos have been fixed, see #292 (@McMartin)

## Packaging

- `hyperfine` is now available on MacPorts for macOS, see #281 (@herbygillot)
- `hyperfine` is now available on OpenBSD, see #289 (@minusf)

Package authors: note that Hyperfine now comes with a set of shell completion files and a man page (see above)

# v1.9.0

## Features

- The new `--parameter-list <VAR> <VALUES>` option can be used to run
  a parametrized benchmark on a user-specified list of values.
  This is similar to `--parameter-scan <VAR> <MIN> <MAX>`, but doesn't
  necessarily required numeric arguments.

  ``` bash
  hyperfine --parameter-list compiler "gcc,clang" \
      "{compiler} -O2 main.cpp"
  ```

  See: #227, #234 (@JuanPotato)

- Added `none` as a possible choice for the `--style` option to
  run `hyperfine` without any output, see #193 (@knidarkness)

- Added a few new scripts for plotting various types of benchmark
  results (https://github.com/sharkdp/hyperfine/tree/master/scripts)

## Changes

- The `--prepare` command is now also run during the warmup
  phase, see #182 (@sseemayer)

- Better estimation of the remaining benchmark time due to an update
  of the `indicatif` crate.

## Other

- `hyperfine` is now available on NixOS, see #240 (@tuxinaut)

# v1.8.0

## Features

- The `--prepare <CMD>` option can now be specified multiple times to
  run specific preparation commands for each of the benchmarked programs:

  ``` bash
  hyperfine --prepare "make clean; git checkout master"  "make" \
            --prepare "make clean; git checkout feature" "make"
  ```

  See: #216, #218 (@iamsauravsharma)

- Added a new [`welch_ttest.py`](https://github.com/sharkdp/hyperfine/blob/master/scripts/welch_ttest.py) script to test whether or not the two benchmark
  results are the same, see #222 (@uetchy)

- The Markdown export has been improved. The relative speed is now exported
  with a higher precision (see #208) and includes the standard deviation
  (see #225).

## Other

- Improved documentation for [`scripts`](https://github.com/sharkdp/hyperfine/tree/master/scripts) folder (@matthieusb)

# v1.7.0

## Features

- Added a new `-D`,`--parameter-step-size` option that can be used to control
  the step size for `--parameter-scan` benchmarks. In addition, decimal numbers
  are now allowed for parameter scans. For example, the following command runs
  `sleep 0.3`, `sleep 0.5` and `sleep 0.7`:
  ``` bash
  hyperfine --parameter-scan delay 0.3 0.7 -D 0.2 'sleep {delay}'
  ```
  For more details, see #184 (@piyushrungta25)

## Other

- hyperfine is now in the official Alpine repositories, see #177 (@maxice8, @5paceToast)
- hyperfine is now in the official Fedora repositories, see #196 (@ignatenkobrain)
- hyperfine is now in the official Arch Linux repositories
- hyperfine can be installed on FreeBSD, see #204 (@0mp)
- Enabled LTO for slightly smaller binary sizes, see #179 (@Calinou)
- Various small improvements all over the code base, see #194 (@phimuemue)

# v1.6.0

## Features

- Added a `-c, --cleanup <CMD>` option to execute `CMD` after the completion of all benchmarking runs for a given command. This is useful if the commands to be benchmarked produce artifacts that need to be cleaned up. See #91 (@RalfJung and @colinwahl)
- Add parameter values (for `--parameter-scan` benchmarks) to exported CSV and JSON files. See #131 (@bbannier)
- Added AsciiDoc export option, see #137 (@5paceToast)
- The relative speed is now part of the Markdown export, see #127 (@mathiasrw and @sharkdp).
- The *median* run time is now exported via CSV and JSON, see #171 (@hosewiejacke and @sharkdp).

## Other

- Hyperfine has been updated to Rust 2018 (@AnderEnder). The minimum supported Rust version is now 1.31.

# v1.5.0

## Features

- Show the number of runs in `hyperfine`s output (@tcmal)
- Added two Python scripts to post-process exported benchmark results (see [`scripts/`](https://github.com/sharkdp/hyperfine/tree/master/scripts) folder)

## Other

- Refined `--help` text for the `--export-*` flags (@psteinb)
- Added Snapcraft file (@popey)
- Small improvements in the progress bar "experience".

# v1.4.0

## Features

- Added `-S`/`--shell` option to override the default shell, see #61 (@mqudsi and @jasonpeacock)
- Added `-u`/`--time-unit` option to change the unit of time (`second` or `millisecond`), see #80 (@jasonpeacock)
- Markdown export auto-selects time unit, see #71 (@jasonpeacock)

# v1.3.0

## Feature

- Compute and print standard deviation of the speed ratio, see #83 (@Shnatsel)
- More compact output format, see #70  (@jasonpeacock)
- Added `--style=color`, see #70 (@jasonpeacock)
- Added options to specify the max/exact numbers of runs, see #77 (@orium)

## Bugfixes

- Change Windows `cmd` interpreter to `cmd.exe` to prevent accidentally calling other programs, see #74 (@tathanhdinh)

## Other

- Binary releases for Windows are now available, see #87 

# v1.2.0

- Support parameters in preparation commands, see #68 (@siiptuo)
- Updated dependencies, see #69. The minimum required Rust version is now 1.24.

# v1.1.0

* Added `--show-output` option (@chrisduerr and @sevagh)
* Refactoring work (@stevepentland)

# v1.0.0

## Features

* Support for various export-formats like CSV, JSON and Markdown - see #38, #44, #49, #42 (@stevepentland)
* Summary output that compares the different benchmarks, see #6 (@stevepentland)
* Parameterized benchmarks via `-P`, `--parameter-scan <VAR> <MIN> <MAX>`, see #19

## Thanks

I'd like to say a big THANK YOU to @stevepentland for implementing new features,
for reviewing pull requests and for giving very valuable feedback.

# v0.5.0

* Proper Windows support (@stevepentland)
* Added `--style auto/basic/nocolor/full` option (@stevepentland)
* Correctly estimate the full execution time, see #27 (@rleungx)
* Added Void Linux install instructions (@wpbirney)

# v0.4.0

- New `--style` option to disable output coloring and interactive CLI features, see #24 (@stevepentland)
- Statistical outlier detection, see #23 #18 

# v0.3.0

## Features

- In addition to 'real' (wall clock) time, Hyperfine can now also measure 'user' and 'system' time (see #5).
- Added `--prepare` option that can be used to clear up disk caches before timing runs, for example (see #8).

## Other

- [Arch Linux package](https://aur.archlinux.org/packages/hyperfine) for Hyperfine (@jD91mZM2).
- Ubuntu/Debian packages are now are available.

# v0.2.0

Initial public release
