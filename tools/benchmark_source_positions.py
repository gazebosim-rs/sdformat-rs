#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Compare exact semantic loader outputs before timing a source-position change."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import tempfile
import tomllib


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def registry_packages(path):
    return {(p['name'], p['version'], p['source'], p.get('checksum'))
            for p in tomllib.loads(path.read_text())['package'] if p.get('source')}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--baseline', required=True, type=Path)
    parser.add_argument('--candidate', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--output', type=Path, default=Path('artifacts/source-position-benchmark'))
    parser.add_argument('--pairs', type=int, default=7)
    parser.add_argument('--cpu', type=int)
    args = parser.parse_args()
    assert args.pairs > 0, 'at least one measured pair is required'
    candidate = args.candidate.resolve()
    baseline = args.baseline.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    affinity = os.sched_getaffinity(0)
    cpu = min(affinity) if args.cpu is None else args.cpu
    assert cpu in affinity, 'requested CPU is unavailable'
    os.sched_setaffinity(0, {cpu})  # Only this driver and its children.
    project_root = Path(tempfile.mkdtemp(prefix='build-', dir=output))
    roots = {'baseline': baseline, 'candidate': candidate}
    assert baseline != candidate, 'two distinct source trees are required'
    assert (baseline / 'Cargo.toml').read_bytes() == (candidate / 'Cargo.toml').read_bytes(), 'manifest changed'
    assert (baseline / 'Cargo.lock').read_bytes() == (candidate / 'Cargo.lock').read_bytes(), 'dependency graph changed'
    assert (baseline / 'rust-toolchain.toml').read_bytes() == (candidate / 'rust-toolchain.toml').read_bytes(), 'toolchain changed'
    harness = candidate / 'tests/support/loader_benchmark.rs'
    harness_hash = sha(harness)
    runner_hash = sha(Path(__file__))
    projects = {}
    binaries = {}
    source_hashes = {}
    env = dict(os.environ)
    env.pop('RUSTFLAGS', None)
    env.pop('CARGO_ENCODED_RUSTFLAGS', None)
    for name, source in roots.items():
        project = project_root / name
        (project / 'src').mkdir(parents=True, exist_ok=True)
        (project / 'src/main.rs').write_bytes(harness.read_bytes())
        manifest = '[package]\nname="sdf-position-benchmark"\nversion="0.0.0"\nedition="2024"\nlicense="Apache-2.0"\n[workspace]\n[dependencies]\nsdformat={path=' + json.dumps(str(source), ensure_ascii=False) + '}\n'
        (project / 'Cargo.toml').write_text(manifest)
        (project / 'rust-toolchain.toml').write_bytes((source / 'rust-toolchain.toml').read_bytes())
        local_env = dict(env, CARGO_TARGET_DIR=str(project / 'target'))
        subprocess.run(['cargo', 'generate-lockfile', '--offline'], cwd=project, env=local_env, check=True)
        assert registry_packages(project / 'Cargo.lock') <= registry_packages(source / 'Cargo.lock'), 'unlocked dependency version in benchmark'
        source_hashes[name] = {str(p.relative_to(source)): sha(p) for p in sorted((source / 'src').rglob('*.rs'))}
        with (project / 'build.log').open('w') as log:
            subprocess.run(['cargo', 'build', '--release', '--locked', '--offline', '--verbose'], cwd=project, env=local_env, check=True, stdout=log, stderr=subprocess.STDOUT)
        binary = project / 'target/release/sdf-position-benchmark'
        binaries[name] = binary
        projects[name] = dict(project=str(project.relative_to(output)), binary_sha256=sha(binary), manifest_sha256=sha(project / 'Cargo.toml'), lock_sha256=sha(project / 'Cargo.lock'), production_sha256=source_hashes[name])
    inputs = []
    for count, iterations in [(10, 100), (100, 10), (1000, 1)]:
        links = ''.join(f"<link name='l{i}'/>" for i in range(count))
        frames = ''.join(f"<frame name='f{i}' attached_to='{'l0' if i == 0 else 'f' + str(i-1)}'><pose>0.01 0 0 0 0 0</pose></frame>" for i in range(count))
        xml = f"<sdf version='1.12'><model name='bench'>{links}{frames}</model></sdf>"
        path = output / f'model-{count}.sdf'
        path.write_text(xml)
        inputs.append((path, count, iterations))
        if count == 10:
            plugin = "<plugin name='p' filename='f'>" + ''.join(f"<v key='{i}'>α😀{i}</v>" for i in range(10000)) + '</plugin>'
            opaque = output / 'model-10-opaque-tail.sdf'
            opaque.write_text(xml.replace('</model>', plugin + '</model>'))
            inputs.append((opaque, count, 2))
    samples = []
    states = {}
    for path, count, iterations in inputs:
        for pair in range(args.pairs + 1):
            measured = {}
            dumps = {}
            order = ['baseline', 'candidate'] if pair % 2 == 0 else ['candidate', 'baseline']
            for name in order:
                state = project_root / name / (path.stem + '-state.txt')
                run = subprocess.run(['taskset', '-c', str(cpu), str(binaries[name]), str(path), str(count), str(iterations), str(state)], env=env, text=True, capture_output=True, check=True)
                measured[name] = json.loads(run.stdout)
                assert abs(measured[name]['checksum'] - .01 * count * iterations) <= 1e-8
                dumps[name] = state.read_bytes()
            assert measured['baseline']['checksum'] == measured['candidate']['checksum']
            assert dumps['baseline'] == dumps['candidate'], f'complete semantic debug state differs: {path.name}'
            states[path.name] = hashlib.sha256(dumps['baseline']).hexdigest()
            samples.append(dict(case=path.name, pair=pair, discarded_warmup=pair == 0, order=order, measurements=measured))
    medians = []
    for path, _, _ in inputs:
        row = dict(case=path.name)
        for name in roots:
            values = [s['measurements'][name]['nanoseconds'] / s['measurements'][name]['iterations'] for s in samples if s['case'] == path.name and not s['discarded_warmup']]
            row[name] = dict(median_ns_per_load=statistics.median(values), min_ns_per_load=min(values), max_ns_per_load=max(values))
        row['baseline_over_candidate_ratio'] = row['baseline']['median_ns_per_load'] / row['candidate']['median_ns_per_load']
        medians.append(row)
    for name, source in roots.items():
        assert source_hashes[name] == {str(p.relative_to(source)): sha(p) for p in sorted((source / 'src').rglob('*.rs'))}, 'source changed during benchmark'
    assert harness_hash == sha(harness) and runner_hash == sha(Path(__file__)), 'benchmark harness changed during measurement'
    compiler = subprocess.check_output(['rustc', '-vV'], cwd=project_root / 'candidate', env=env, text=True)
    report = dict(compiler=compiler, cpu=cpu, fresh_build_targets=True, projects=projects, harness_sha256=harness_hash, runner_sha256=runner_hash, inputs_sha256={p.name: sha(p) for p, _, _ in inputs}, exact_semantic_state_sha256=states, samples=samples, medians=medians,
                  scope='Identical release harness/toolchain/locked registry versions; exact entire Sdf debug bytes checked outside timing, then count/last-frame pose/attachment/checksum asserted on every measured load. Input read before timing; one warm validated load/state dump per batch and an extra discarded alternating pair.',
                  qualifications=['Author-only unless independently replayed; fixed CPU on shared host, no exclusive CPU/frequency control.', 'Opaque plugin-tail counterworkload retained to detect eager-cache overhead.', 'No native full-loader, P2, SDF-02, whole-engine or portable performance acceptance; debug-state equality is a bounded regression check, not a public format contract.'])
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(medians, indent=2))


if __name__ == '__main__':
    main()
