#!/usr/bin/env python3
"""Side-by-side byte-for-byte differential scoreboard against GNU ed."""
import argparse, json, pathlib, shutil, subprocess, sys, tempfile

def run_bin(bin_path, args, stdin_bytes, workdir):
    p = subprocess.run(
        [bin_path, *args],
        input=stdin_bytes,
        cwd=workdir,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=10,
    )
    files = {}
    for f in sorted(pathlib.Path(workdir).rglob("*")):
        if f.is_file():
            files[str(f.relative_to(workdir))] = f.read_bytes()
    return p.returncode, p.stdout, p.stderr, files

def run_case(case_path, ref_bin, cand_bin):
    spec = json.loads(case_path.read_text(encoding="utf-8"))
    args = spec.get("args", [])
    stdin_bytes = spec.get("stdin", "").encode("utf-8")
    setup_files = spec.get("files", {})
    results = []
    for b in (ref_bin, cand_bin):
        d = tempfile.mkdtemp(prefix="ed-case-")
        try:
            for rel, content in setup_files.items():
                p = pathlib.Path(d) / rel
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_bytes(content.encode("utf-8"))
            results.append(run_bin(b, args, stdin_bytes, d))
        finally:
            shutil.rmtree(d, ignore_errors=True)
    ref_r, cand_r = results
    diffs = []
    if ref_r[0] != cand_r[0]:
        diffs.append(f"exit: ref={ref_r[0]} cand={cand_r[0]}")
    if ref_r[1] != cand_r[1]:
        diffs.append(f"stdout: ref={ref_r[1]!r} cand={cand_r[1]!r}")
    if ref_r[2] != cand_r[2]:
        diffs.append(f"stderr: ref={ref_r[2]!r} cand={cand_r[2]!r}")
    if ref_r[3] != cand_r[3]:
        diffs.append(f"files: ref={sorted(ref_r[3].keys())} cand={sorted(cand_r[3].keys())}")
    return diffs

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ref", default="/usr/bin/ed")
    ap.add_argument("--cand", required=True)
    ap.add_argument("--cases", default=str(pathlib.Path(__file__).parent / "cases"))
    ap.add_argument("--case", default=None)
    ns = ap.parse_args()
    cases = sorted(pathlib.Path(ns.cases).glob("*.json"))
    if ns.case:
        cases = [c for c in cases if c.stem == ns.case]
    passed = 0
    for c in cases:
        diffs = run_case(c, ns.ref, ns.cand)
        if not diffs:
            passed += 1
            print(f"PASS {c.stem}")
        else:
            print(f"FAIL {c.stem}: " + "; ".join(diffs))
    print(f"SCOREBOARD: {passed} / {len(cases)} passing")
    sys.exit(0 if passed == len(cases) else 1)

if __name__ == "__main__":
    main()
