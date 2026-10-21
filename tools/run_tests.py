"""Runs `cargo test` with a time limit and a summary, for the commit gate.

The HP comparison classes are heavy (about 13 s of CPU per test, profile.test is opt-level
3), so on a loaded machine the suite can take a long time. This runner puts cargo and every
test binary in their own process group and kills the whole group when the limit passes, so
no test process is left running, and it reports a run that stopped early (a test binary
killed or crashed) as a failure rather than as a short pass.

Usage: python3 -I tools/run_tests.py [--minutes 120] [extra cargo test args]
"""
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    args = sys.argv[1:]
    minutes = 120.0
    if args[:1] == ["--minutes"]:
        minutes = float(args[1])
        args = args[2:]
    env = dict(os.environ, TMPDIR=str(ROOT / "target" / "tmp"))
    (ROOT / "target" / "tmp").mkdir(parents=True, exist_ok=True)
    started = time.time()
    proc = subprocess.Popen(
        ["cargo", "test", "-q", *args],
        cwd=ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    try:
        out, _ = proc.communicate(timeout=minutes * 60)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        out, _ = proc.communicate()
        print(f"TIMEOUT after {minutes:g} minutes; every test process was stopped")
        return 2
    passed = failed = ignored = binaries = 0
    for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", out):
        binaries += 1
        passed += int(m.group(1))
        failed += int(m.group(2))
        ignored += int(m.group(3))
    elapsed = (time.time() - started) / 60
    print(f"tests passed {passed} failed {failed} ignored {ignored} in {binaries} test binaries, {elapsed:.1f} min")
    if proc.returncode != 0 or failed:
        tail = [l for l in out.splitlines() if l.startswith(("error", "---- ", "thread ")) or "FAILED" in l]
        print("\n".join(tail[:40]))
        print(f"cargo exited with {proc.returncode}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
