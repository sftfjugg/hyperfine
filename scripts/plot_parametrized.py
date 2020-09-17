#!/usr/bin/python

"""This program shows parametrized `hyperfine` benchmark results as an
errorbar plot."""

import argparse
import json
import matplotlib.pyplot as plt
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("file", help="JSON file with benchmark results", nargs="+")
parser.add_argument(
    "--parameter-name",
    metavar="name",
    type=str,
    help="Deprecated; parameter names are now inferred from benchmark files",
)
parser.add_argument(
    "--log-x", help="Use a logarithmic x (parameter) axis", action="store_true"
)
parser.add_argument(
    "--log-time", help="Use a logarithmic time axis", action="store_true"
)
parser.add_argument(
    "--titles", help="Comma-separated list of titles for the plot legend"
)

args = parser.parse_args()
if args.parameter_name is not None:
    sys.stderr.write(
        "warning: --parameter-name is deprecated; names are inferred from "
        "benchmark results\n"
    )


def die(msg):
    sys.stderr.write("fatal: %s\n" % (msg,))
    sys.exit(1)


def extract_parameters(results):
    """Return `(parameter_name: str, parameter_values: List[float])`."""
    if not results:
        die("no benchmark data to plot")
    (names, values) = zip(*(unique_parameter(b) for b in results))
    names = frozenset(names)
    if len(names) != 1:
        die(
            "benchmarks must all have the same parameter name, but found: %s"
            % sorted(names)
        )
    return (next(iter(names)), values)


def unique_parameter(benchmark):
    """Return the unique parameter `(name: str, value: float)`, or dies."""
    params_dict = benchmark.get("parameters", {})
    if not params_dict:
        die("benchmarks must have exactly one parameter, but found none")
    if len(params_dict) > 1:
        die(
            "benchmarks must have exactly one parameter, but found multiple: %s"
            % sorted(params_dict)
        )
    return next(iter(params_dict.items()))


parameter_name = None

for filename in args.file:
    with open(filename) as f:
        results = json.load(f)["results"]

    (this_parameter_name, parameter_values) = extract_parameters(results)
    if parameter_name is not None and this_parameter_name != parameter_name:
        die(
            "files must all have the same parameter name, but found %r vs. %r"
            % (parameter_name, this_parameter_name)
        )
    parameter_name = this_parameter_name

    parameter_values = [float(pv) for pv in parameter_values]
    times_mean = [b["mean"] for b in results]
    times_stddev = [b["stddev"] for b in results]

    plt.errorbar(x=parameter_values, y=times_mean, yerr=times_stddev, capsize=2)

plt.xlabel(parameter_name)
plt.ylabel("Time [s]")

if args.log_time:
    plt.yscale("log")
else:
    plt.ylim(0, None)

if args.log_x:
    plt.xscale("log")

if args.titles:
    plt.legend(args.titles.split(","))

plt.show()
