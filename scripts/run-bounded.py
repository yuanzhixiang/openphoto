#!/usr/bin/env python3
"""Run a command with a deadline and terminate its process group on interruption."""

import argparse
import os
import signal
import subprocess
import sys


def stop(process):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=float, required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command
    if command and command[0] == "--":
        command = command[1:]
    if not command or args.timeout <= 0:
        parser.error("a positive timeout and a command are required")
    process = subprocess.Popen(command, start_new_session=True)
    print(f"PID={process.pid} deadline={args.timeout:g}s", file=sys.stderr, flush=True)
    try:
        return process.wait(timeout=args.timeout)
    except subprocess.TimeoutExpired:
        print("Deadline exceeded; stopping the process group", file=sys.stderr)
        stop(process)
        return 124
    except KeyboardInterrupt:
        stop(process)
        return 130


if __name__ == "__main__":
    sys.exit(main())
