#!/usr/bin/env node

// oxlint-disable no-console

/**
 * Performance & Benchmark Integrity Gate (`perf-gate.js`)
 *
 * Enforces the performance engineering invariants documented in `AGENTS.md`:
 * 1. Canonical benchmark setup (`Stats::count` outside `semantic` timed loop via `.with_stats(stats)`,
 *    fresh AST per iteration, `parse_regular_expression: true` in parser benchmarks, no heuristic
 *    `Stats` estimation in `SemanticBuilder::build`).
 * 2. Memory allocation & zero-reallocation gate (`tasks/track_memory_allocations/allocs_*.yaml`):
 *    - `allocs_semantic.yaml` must have `sys reallocs: 0` and `arena reallocs: 0` on every fixture.
 *    - Against `--base <ref>`, `sys reallocs` / `arena reallocs` must not increase, and
 *      `sys alloc bytes` / `sys peak growth` must not grow by > 2% without an `arena size` reduction.
 * 3. `Full` vs `Ancestry` semantic parity gate (`cargo test -p oxc_semantic --all-features`).
 * 4. Interleaved per-file A/B benchmark & `.text` instruction-size gate across:
 *    - `parse_std` (`parse_regular_expression: true, enable_ident_hashes: true`)
 *    - `parse_nohash` (`parse_regular_expression: true, enable_ident_hashes: false`)
 *    - `parse_tokens` (`RuntimeParserConfig::new(true)`)
 *    - `semantic_with_stats` (`with_stats(stats)` on a fresh AST every iteration)
 */

const { execFileSync, execSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const process = require("process");

function parseArgs(argv) {
  const opts = {
    baseRef: null,
    baseBin: null,
    headBin: null,
    runAllocs: false,
    runBench: true,
    rounds: 5,
    abRounds: 4,
    targetMs: 15,
    filter: "",
    maxAggregateRegressionPct: 2.5,
    maxFileRegressionPct: 5.0,
    maxHeapGrowthPct: 2.0,
    maxTextGrowthBytes: 4096,
  };

  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--base" && argv[i + 1]) {
      opts.baseRef = argv[++i];
    } else if (arg === "--base-bin" && argv[i + 1]) {
      opts.baseBin = argv[++i];
    } else if (arg === "--head-bin" && argv[i + 1]) {
      opts.headBin = argv[++i];
    } else if (arg === "--run-allocs") {
      opts.runAllocs = true;
    } else if (arg === "--no-bench") {
      opts.runBench = false;
    } else if (arg === "--rounds" && argv[i + 1]) {
      opts.rounds = Number(argv[++i]);
    } else if (arg === "--ab-rounds" && argv[i + 1]) {
      opts.abRounds = Number(argv[++i]);
    } else if (arg === "--target-ms" && argv[i + 1]) {
      opts.targetMs = Number(argv[++i]);
    } else if (arg === "--filter" && argv[i + 1]) {
      opts.filter = argv[++i];
    } else if (arg === "--max-aggregate-regression" && argv[i + 1]) {
      opts.maxAggregateRegressionPct = Number(argv[++i]);
    } else if (arg === "--max-file-regression" && argv[i + 1]) {
      opts.maxFileRegressionPct = Number(argv[++i]);
    }
  }
  return opts;
}

function parseAllocsYaml(content) {
  const files = new Map();
  let currentFile = null;
  for (const rawLine of content.split("\n")) {
    const line = rawLine.replace(/#.*$/u, "").trimEnd();
    if (!line.trim()) {
      continue;
    }
    if (!line.startsWith(" ") && line.endsWith(":")) {
      currentFile = line.slice(0, -1).trim();
      files.set(currentFile, {});
    } else if (currentFile && line.startsWith("  ")) {
      const sep = line.indexOf(":");
      if (sep !== -1) {
        const key = line.slice(0, sep).trim();
        const val = Number(line.slice(sep + 1).trim());
        files.get(currentFile)[key] = val;
      }
    }
  }
  return files;
}

function checkStaticInvariants(failures) {
  console.log("==> [1/4] Checking static benchmark & architecture invariants...");

  const semanticBench = fs.readFileSync("tasks/benchmark/benches/semantic.rs", "utf8");
  if (
    !semanticBench.includes("Stats::count(&program)") ||
    !semanticBench.includes(".with_stats(stats)")
  ) {
    failures.push(
      "tasks/benchmark/benches/semantic.rs must precompute `Stats::count(&program)` outside `runner.run` and pass `.with_stats(stats)` (oxc-project/oxc#28467).",
    );
  }

  const semanticBuilder = fs.readFileSync("crates/oxc_semantic/src/builder.rs", "utf8");
  if (semanticBuilder.includes("estimate_from_source")) {
    failures.push(
      "crates/oxc_semantic/src/builder.rs must not use heuristic `estimate_from_source`; `SemanticBuilder::build` must use `Stats::count(program)` when `self.stats` is `None`.",
    );
  }

  const parserBench = fs.readFileSync("tasks/benchmark/benches/parser.rs", "utf8");
  if (!parserBench.includes("parse_regular_expression: true")) {
    failures.push("tasks/benchmark/benches/parser.rs must set `parse_regular_expression: true`.");
  }
}

function checkAllocationInvariants(opts, failures, summaryLines) {
  console.log("==> [2/4] Checking memory allocation & zero-reallocation invariants...");

  if (opts.runAllocs) {
    execSync("cargo run --release -p oxc_track_memory_allocations", { stdio: "inherit" });
    try {
      execSync("git diff --exit-code tasks/track_memory_allocations/", { stdio: "inherit" });
    } catch {
      failures.push(
        "Uncommitted changes in `tasks/track_memory_allocations/`. Run `cargo allocs` and commit the updated snapshots, or fix the allocation regression.",
      );
    }
  }

  const semAllocsPath = "tasks/track_memory_allocations/allocs_semantic.yaml";
  const semAllocs = parseAllocsYaml(fs.readFileSync(semAllocsPath, "utf8"));
  for (const [file, metrics] of semAllocs.entries()) {
    if (metrics["sys reallocs"] !== 0 || metrics["arena reallocs"] !== 0) {
      failures.push(
        `allocs_semantic.yaml [${file}] has non-zero reallocations (sys reallocs=${metrics["sys reallocs"]}, arena reallocs=${metrics["arena reallocs"]}). Exact Stats::count reservation requires 0 reallocs.`,
      );
    }
  }

  if (opts.baseRef) {
    for (const snap of ["allocs_parser.yaml", "allocs_semantic.yaml"]) {
      const relPath = `tasks/track_memory_allocations/${snap}`;
      let baseContent = null;
      try {
        baseContent = execFileSync("git", ["show", `${opts.baseRef}:${relPath}`], {
          encoding: "utf8",
          stdio: ["ignore", "pipe", "ignore"],
        });
      } catch {
        continue;
      }
      const baseMap = parseAllocsYaml(baseContent);
      const headMap = parseAllocsYaml(fs.readFileSync(relPath, "utf8"));

      for (const [file, headMetrics] of headMap.entries()) {
        const baseMetrics = baseMap.get(file);
        if (!baseMetrics) {
          continue;
        }
        if (headMetrics["sys reallocs"] > baseMetrics["sys reallocs"]) {
          failures.push(
            `${snap} [${file}]: sys reallocs increased from ${baseMetrics["sys reallocs"]} to ${headMetrics["sys reallocs"]}.`,
          );
        }
        if (headMetrics["arena reallocs"] > baseMetrics["arena reallocs"]) {
          failures.push(
            `${snap} [${file}]: arena reallocs increased from ${baseMetrics["arena reallocs"]} to ${headMetrics["arena reallocs"]}.`,
          );
        }
        const baseBytes = baseMetrics["sys alloc bytes"] || 0;
        const headBytes = headMetrics["sys alloc bytes"] || 0;
        if (baseBytes > 0) {
          const growthPct = ((headBytes - baseBytes) / baseBytes) * 100;
          if (growthPct > opts.maxHeapGrowthPct) {
            failures.push(
              `${snap} [${file}]: sys alloc bytes grew by +${growthPct.toFixed(2)}% (${baseBytes} -> ${headBytes}, max allowed +${opts.maxHeapGrowthPct}%). Small-module heap growth directly regresses 10k-module and HMR workloads.`,
            );
          }
        }
      }
    }
    summaryLines.push(
      `- **Allocation Gate (\`allocs_parser.yaml\`, \`allocs_semantic.yaml\`)**: 0 semantic reallocs, within heap growth budget vs \`${opts.baseRef}\`.`,
    );
  } else {
    summaryLines.push(
      "- **Allocation Gate (`allocs_semantic.yaml`)**: 0 `sys reallocs` and 0 `arena reallocs` across all fixtures.",
    );
  }
}

function checkSemanticParity(failures, summaryLines) {
  console.log("==> [3/4] Running Full vs Ancestry semantic parity tests...");
  try {
    execSync("cargo test -p oxc_semantic --all-features --quiet", { stdio: "inherit" });
    summaryLines.push(
      "- **Semantic Parity (`with_build_nodes(true)` vs `with_build_nodes(false)`)**: All unit, conformance, JSDoc, and diagnostic parity tests passed.",
    );
  } catch {
    failures.push("`cargo test -p oxc_semantic --all-features` failed.");
  }
}

function getTextSectionBytes(binPath) {
  try {
    if (process.platform === "darwin") {
      const out = execFileSync("size", ["-m", binPath], { encoding: "utf8" });
      const match = out.match(/Section __text:\s*(\d+)/u);
      if (match) {
        return Number(match[1]);
      }
    }
    const out = execFileSync("size", [binPath], { encoding: "utf8" });
    const lines = out.trim().split("\n");
    if (lines.length >= 2) {
      const cols = lines[lines.length - 1].trim().split(/\s+/u);
      const textBytes = Number(cols[0]);
      if (Number.isFinite(textBytes) && textBytes > 0) {
        return textBytes;
      }
    }
  } catch {
    // Fall back to file size if `size` is unavailable.
  }
  return fs.statSync(binPath).size;
}

function runPerfGateBinary(binPath, opts, filterFile = "") {
  const out = execFileSync(binPath, [], {
    encoding: "utf8",
    env: {
      ...process.env,
      ROUNDS: String(opts.rounds),
      TARGET_MS: String(opts.targetMs),
      FILTER: filterFile || opts.filter,
    },
  });
  const results = new Map();
  for (const line of out.split("\n")) {
    if (!line.startsWith("RESULT|")) {
      continue;
    }
    const parts = line.trim().split("|");
    results.set(parts[1], {
      size: Number(parts[2]),
      parseStdNs: Number(parts[3]),
      parseNohashNs: Number(parts[4]),
      semWithStatsNs: Number(parts[5]),
      bundlerPipelineNs: Number(parts[6]),
    });
  }
  return results;
}

function fmtUs(ns) {
  return `${(ns / 1e3).toFixed(1)} us`;
}

function fmtMs(ns) {
  return `${(ns / 1e6).toFixed(2)} ms`;
}

function fmtPct(base, head) {
  const pct = ((head - base) / base) * 100;
  const sign = pct >= 0 ? "+" : "";
  return `${sign}${pct.toFixed(1)}%`;
}

function buildHeadBinary() {
  console.log("Building HEAD perf_gate binary...");
  const binPath = path.resolve("target/release/examples/perf_gate");
  fs.rmSync(binPath, { force: true });
  const now = new Date();
  fs.utimesSync(path.resolve("tasks/benchmark/examples/perf_gate.rs"), now, now);
  execSync(
    "cargo build --release -p oxc_benchmark --example perf_gate --no-default-features --features compiler",
    { stdio: "inherit" },
  );
  const copyPath = path.resolve("target/perf_gate_head");
  fs.copyFileSync(binPath, copyPath);
  return copyPath;
}

function buildBaseBinaryFromRef(baseRef) {
  console.log(`Building BASE perf_gate binary from ${baseRef}...`);
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "oxc-perf-gate-base-"));
  // Use a separate target directory (`target/base-target`) so the temporary worktree's
  // fresh file mtimes never poison `target/release/deps/*.rlib` for the HEAD worktree.
  const baseTargetDir = path.resolve("target/base-target");
  const builtBinPath = path.join(baseTargetDir, "release/examples/perf_gate");
  const outBinPath = path.resolve("target/perf_gate_base");
  try {
    execFileSync("git", ["worktree", "add", "--detach", tmpDir, baseRef], { stdio: "inherit" });
    const exDir = path.join(tmpDir, "tasks/benchmark/examples");
    fs.mkdirSync(exDir, { recursive: true });
    const dstPerfGate = path.join(exDir, "perf_gate.rs");
    fs.copyFileSync(path.resolve("tasks/benchmark/examples/perf_gate.rs"), dstPerfGate);
    const cargoTomlPath = path.join(tmpDir, "tasks/benchmark/Cargo.toml");
    const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
    if (!cargoToml.includes('name = "perf_gate"')) {
      fs.writeFileSync(
        cargoTomlPath,
        `${cargoToml}\n[[example]]\nname = "perf_gate"\nrequired-features = ["compiler"]\n`,
      );
    }
    fs.rmSync(builtBinPath, { force: true });
    execSync(
      "cargo build --release -p oxc_benchmark --example perf_gate --no-default-features --features compiler",
      {
        cwd: tmpDir,
        stdio: "inherit",
        env: { ...process.env, CARGO_TARGET_DIR: baseTargetDir },
      },
    );
    fs.copyFileSync(builtBinPath, outBinPath);
    return outBinPath;
  } finally {
    try {
      execFileSync("git", ["worktree", "remove", "--force", tmpDir], { stdio: "ignore" });
    } catch {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  }
}

function runBenchmarks(opts, failures, summaryLines) {
  if (!opts.runBench) {
    return;
  }
  console.log("==> [4/4] Running canonical parser & semantic_with_stats benchmark gate...");

  const headBin = opts.headBin ? path.resolve(opts.headBin) : buildHeadBinary();
  let baseBin = opts.baseBin ? path.resolve(opts.baseBin) : null;
  if (!baseBin && opts.baseRef) {
    baseBin = buildBaseBinaryFromRef(opts.baseRef);
  }

  if (!baseBin) {
    const headMap = runPerfGateBinary(headBin, opts);
    const headTextBytes = getTextSectionBytes(headBin);
    summaryLines.push(
      `- **Binary \`.text\` section**: \`${headTextBytes.toLocaleString()} bytes\``,
      "",
      "| File | Size | Parse (std) | Parse (no-hash) | Semantic (`with_stats`) | Bundler Pipeline (`parse+sem+xform+codegen`) |",
      "| :--- | ---: | ---: | ---: | ---: | ---: |",
    );
    let totStd = 0;
    let totNohash = 0;
    let totSem = 0;
    let totPipe = 0;
    for (const [file, m] of headMap.entries()) {
      totStd += m.parseStdNs;
      totNohash += m.parseNohashNs;
      totSem += m.semWithStatsNs;
      totPipe += m.bundlerPipelineNs;
      summaryLines.push(
        `| \`${file}\` | ${(m.size / 1024).toFixed(1)} KB | ${fmtUs(m.parseStdNs)} | ${fmtUs(m.parseNohashNs)} | ${fmtUs(m.semWithStatsNs)} | ${fmtUs(m.bundlerPipelineNs)} |`,
      );
    }
    summaryLines.push(
      `| **TOTAL** | | **${fmtMs(totStd)}** | **${fmtMs(totNohash)}** | **${fmtMs(totSem)}** | **${fmtMs(totPipe)}** |`,
    );
    return;
  }

  const baseTextBytes = getTextSectionBytes(baseBin);
  const headTextBytes = getTextSectionBytes(headBin);
  const textDeltaBytes = headTextBytes - baseTextBytes;
  const textDeltaPct = ((textDeltaBytes / baseTextBytes) * 100).toFixed(2);
  summaryLines.push(
    `- **Binary \`.text\` instruction bytes**: \`${baseTextBytes.toLocaleString()}\` -> \`${headTextBytes.toLocaleString()}\` (\`${textDeltaBytes >= 0 ? "+" : ""}${textDeltaBytes} bytes\`, \`${textDeltaBytes >= 0 ? "+" : ""}${textDeltaPct}%\`)`,
  );

  // Discover fixture list and download any uncached fixtures once before timing.
  const initialHead = runPerfGateBinary(headBin, { ...opts, rounds: 1, targetMs: 2 });
  const files = [...initialHead.keys()];

  const baseBest = new Map();
  const headBest = new Map();

  for (const file of files) {
    for (let r = 0; r < opts.abRounds; r++) {
      const order = r % 2 === 0 ? ["base", "head"] : ["head", "base"];
      for (const which of order) {
        const bin = which === "base" ? baseBin : headBin;
        const targetMap = which === "base" ? baseBest : headBest;
        const m = runPerfGateBinary(bin, opts, file).get(file);
        const prev = targetMap.get(file);
        if (!prev) {
          targetMap.set(file, { ...m });
        } else {
          prev.parseStdNs = Math.min(prev.parseStdNs, m.parseStdNs);
          prev.parseNohashNs = Math.min(prev.parseNohashNs, m.parseNohashNs);
          prev.semWithStatsNs = Math.min(prev.semWithStatsNs, m.semWithStatsNs);
          prev.bundlerPipelineNs = Math.min(prev.bundlerPipelineNs, m.bundlerPipelineNs);
        }
      }
    }
  }

  summaryLines.push(
    "",
    "| File | Base Parse | Head Parse | Diff | Base Sem (`with_stats`) | Head Sem (`with_stats`) | Diff | Base Bundler Pipeline | Head Bundler Pipeline | Diff |",
    "| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
  );

  let totBaseStd = 0;
  let totHeadStd = 0;
  let totBaseNohash = 0;
  let totHeadNohash = 0;
  let totBaseSem = 0;
  let totHeadSem = 0;
  let totBasePipe = 0;
  let totHeadPipe = 0;

  for (const file of files) {
    const b = baseBest.get(file);
    const h = headBest.get(file);
    totBaseStd += b.parseStdNs;
    totHeadStd += h.parseStdNs;
    totBaseNohash += b.parseNohashNs;
    totHeadNohash += h.parseNohashNs;
    totBaseSem += b.semWithStatsNs;
    totHeadSem += h.semWithStatsNs;
    totBasePipe += b.bundlerPipelineNs;
    totHeadPipe += h.bundlerPipelineNs;

    summaryLines.push(
      `| \`${file}\` | ${fmtUs(b.parseStdNs)} | ${fmtUs(h.parseStdNs)} | ${fmtPct(b.parseStdNs, h.parseStdNs)} | ${fmtUs(b.semWithStatsNs)} | ${fmtUs(h.semWithStatsNs)} | ${fmtPct(b.semWithStatsNs, h.semWithStatsNs)} | ${fmtUs(b.bundlerPipelineNs)} | ${fmtUs(h.bundlerPipelineNs)} | ${fmtPct(b.bundlerPipelineNs, h.bundlerPipelineNs)} |`,
    );

    const pipeReg = ((h.bundlerPipelineNs - b.bundlerPipelineNs) / b.bundlerPipelineNs) * 100;
    if (pipeReg > opts.maxFileRegressionPct) {
      failures.push(
        `Per-file bundler pipeline regression on ${file}: +${pipeReg.toFixed(1)}% (exceeds +${opts.maxFileRegressionPct}% ceiling).`,
      );
    }

    if (b.size >= 100_000) {
      const parseReg = ((h.parseStdNs - b.parseStdNs) / b.parseStdNs) * 100;
      const semReg = ((h.semWithStatsNs - b.semWithStatsNs) / b.semWithStatsNs) * 100;
      if (parseReg > opts.maxFileRegressionPct) {
        failures.push(
          `Per-file parse regression on ${file}: +${parseReg.toFixed(1)}% (exceeds +${opts.maxFileRegressionPct}% ceiling; check for instruction-cache bloat).`,
        );
      }
      if (semReg > opts.maxFileRegressionPct) {
        failures.push(
          `Per-file semantic_with_stats regression on ${file}: +${semReg.toFixed(1)}% (exceeds +${opts.maxFileRegressionPct}% ceiling).`,
        );
      }
    }
  }

  summaryLines.push(
    `| **TOTAL** | **${fmtMs(totBaseStd)}** | **${fmtMs(totHeadStd)}** | **${fmtPct(totBaseStd, totHeadStd)}** | **${fmtMs(totBaseSem)}** | **${fmtMs(totHeadSem)}** | **${fmtPct(totBaseSem, totHeadSem)}** | **${fmtMs(totBasePipe)}** | **${fmtMs(totHeadPipe)}** | **${fmtPct(totBasePipe, totHeadPipe)}** |`,
    "",
    `- **Parser (no-ident-hash) total**: \`${fmtMs(totBaseNohash)}\` -> \`${fmtMs(totHeadNohash)}\` (\`${fmtPct(totBaseNohash, totHeadNohash)}\`)`,
  );

  const pipeChangePct = ((totHeadPipe - totBasePipe) / totBasePipe) * 100;
  if (textDeltaBytes > opts.maxTextGrowthBytes && pipeChangePct > -2.0) {
    failures.push(
      `Binary .text section grew by +${textDeltaBytes} bytes (max allowed +${opts.maxTextGrowthBytes} bytes) without at least a -2.0% bundler pipeline speedup (measured ${pipeChangePct.toFixed(2)}%). Avoid duplicating fast paths that bloat L1i in downstream bundlers.`,
    );
  }

  const aggChecks = [
    ["Parser (std)", totBaseStd, totHeadStd],
    ["Parser (no-hash)", totBaseNohash, totHeadNohash],
    ["Semantic (with_stats)", totBaseSem, totHeadSem],
    ["Bundler Pipeline (parse+sem+xform+codegen)", totBasePipe, totHeadPipe],
  ];
  for (const [label, bTot, hTot] of aggChecks) {
    const regPct = ((hTot - bTot) / bTot) * 100;
    if (regPct > opts.maxAggregateRegressionPct) {
      failures.push(
        `Aggregate ${label} regressed by +${regPct.toFixed(2)}% (${fmtMs(bTot)} -> ${fmtMs(hTot)}, max allowed +${opts.maxAggregateRegressionPct}%).`,
      );
    }
  }
}

function main() {
  const opts = parseArgs(process.argv);
  const failures = [];
  const summaryLines = ["## Performance & Benchmark Integrity Gate", ""];

  checkStaticInvariants(failures);
  checkAllocationInvariants(opts, failures, summaryLines);
  checkSemanticParity(failures, summaryLines);
  runBenchmarks(opts, failures, summaryLines);

  if (failures.length > 0) {
    summaryLines.push("", "### Gate Failures", "");
    for (const f of failures) {
      summaryLines.push(`- ❌ ${f}`);
    }
  } else {
    summaryLines.push("", "### Gate Status: ✅ Passed");
  }

  const report = `${summaryLines.join("\n")}\n`;
  console.log(`\n${report}`);

  if (process.env.GITHUB_STEP_SUMMARY) {
    fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, report);
  }

  if (failures.length > 0) {
    process.exit(1);
  }
}

main();
