#![expect(clippy::print_stdout)]

use std::{
    env,
    hint::black_box,
    path::Path,
    time::{Duration, Instant},
};

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::{SemanticBuilder, Stats};
use oxc_tasks_common::TestFile;
use oxc_transformer::{TransformOptions, Transformer};

const COMPLICATED_URLS: &[&str] = &[
    "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/checker.ts",
    "https://cdn.jsdelivr.net/gh/excalidraw/excalidraw@f6d85bc80fe328e8f472636eb0d541f7bb891aa0/packages/excalidraw/components/App.tsx",
    "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@cd3bc3d431452b640f5dfcabbc22a8d8a388f393/RadixUIAdoptionSection.jsx",
    "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs",
    "https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js",
    "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/binder.ts",
    "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@cd3bc3d431452b640f5dfcabbc22a8d8a388f393/kitchen-sink.tsx",
];

fn main() {
    #[cfg(target_os = "macos")]
    // SAFETY: `pthread_set_qos_class_self_np` is a standard macOS libsystem symbol;
    // `0x21` (`QOS_CLASS_USER_INTERACTIVE`) pins the benchmark thread to a P-core
    // so XNU does not schedule A/B rounds across heterogeneous P-cores and E-cores.
    unsafe {
        unsafe extern "C" {
            fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
        }
        pthread_set_qos_class_self_np(0x21, 0);
    }

    let rounds: usize = env::var("ROUNDS").ok().and_then(|s| s.parse().ok()).unwrap_or(5);
    let target_ms: u64 = env::var("TARGET_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(15);
    let filter = env::var("FILTER").unwrap_or_default();

    let std_opts = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };
    let nohash_opts = ParseOptions {
        parse_regular_expression: true,
        enable_ident_hashes: false,
        ..ParseOptions::default()
    };
    let transform_options = TransformOptions::from_target("esnext").unwrap();

    // Spin briefly (15ms) at process start so the OS scheduler migrates this thread
    // to a performance core (P-core) and ramps clock frequency to steady-state turbo
    // before calibrating or timing sub-millisecond fixtures.
    {
        let warmup_start = Instant::now();
        let mut acc = 0u64;
        while warmup_start.elapsed() < Duration::from_millis(15) {
            for i in 0..10_000u64 {
                acc = acc.wrapping_add(black_box(i));
            }
        }
        black_box(acc);
    }

    for &url in COMPLICATED_URLS {
        let short_name = url.rsplit('/').next().unwrap_or(url);
        if !filter.is_empty() && short_name != filter && !short_name.contains(&filter) {
            continue;
        }
        let file = TestFile::new(url);

        let source_text = file.source_text.as_str();
        let source_type = file.source_type;
        let source_path = Path::new(&file.file_name);
        let mut allocator = Allocator::default();

        // Precompute `Stats` outside the timed semantic loop (see oxc-project/oxc#28467).
        let stats = {
            let parsed =
                Parser::new(&allocator, source_text, source_type).with_options(std_opts).parse();
            assert!(
                parsed.diagnostics.is_empty(),
                "unexpected parse error in benchmark fixture {}",
                file.file_name
            );
            let stats = Stats::count(&parsed.program);
            drop(parsed);
            allocator.reset();
            stats
        };

        // Run two warmup passes across all stages so arena chunks and allocator freelists
        // are warm, and calibrate each stage's iteration count from its own duration.
        let mut parse_single_ns = 100u128;
        let mut sem_single_ns = 100u128;
        let mut pipe_single_ns = 100u128;
        for _ in 0..2 {
            let t_parse = Instant::now();
            let parsed = Parser::new(&allocator, black_box(source_text), source_type)
                .with_options(std_opts)
                .parse();
            parse_single_ns = t_parse.elapsed().as_nanos().max(100);

            let t_sem = Instant::now();
            let ret = SemanticBuilder::new_compiler().with_stats(stats).build(&parsed.program);
            black_box(&ret.semantic);
            sem_single_ns = t_sem.elapsed().as_nanos().max(100);
            drop(ret);
            drop(parsed);
            allocator.reset();

            let t_pipe = Instant::now();
            let mut parsed = Parser::new(&allocator, black_box(source_text), source_type)
                .with_options(std_opts)
                .parse();
            let scoping = SemanticBuilder::new_compiler()
                .with_excess_capacity(2.0)
                .with_enum_eval(true)
                .build(&parsed.program)
                .semantic
                .into_scoping();
            let _ = Transformer::new(&allocator, source_path, &transform_options)
                .build_with_scoping(scoping, &mut parsed.program);
            let code = Codegen::new().build(&parsed.program);
            black_box(&code);
            drop(code);
            drop(parsed);
            allocator.reset();
            pipe_single_ns = t_pipe.elapsed().as_nanos().max(100);
        }
        let target_ns = u128::from(target_ms) * 1_000_000;
        let parse_iters = usize::try_from((target_ns / parse_single_ns).clamp(2, 200)).unwrap_or(2);
        let sem_iters =
            usize::try_from((target_ns / (parse_single_ns + sem_single_ns)).clamp(2, 200))
                .unwrap_or(2);
        let pipe_iters = usize::try_from((target_ns / pipe_single_ns).clamp(1, 100)).unwrap_or(1);

        let mut parse_std_samples = Vec::with_capacity(rounds);
        let mut parse_nohash_samples = Vec::with_capacity(rounds);
        let mut sem_with_stats_samples = Vec::with_capacity(rounds);
        let mut bundler_pipeline_samples = Vec::with_capacity(rounds);

        let parse_iters_u32 = u32::try_from(parse_iters).unwrap_or(1);
        let sem_iters_u32 = u32::try_from(sem_iters).unwrap_or(1);
        let pipe_iters_u32 = u32::try_from(pipe_iters).unwrap_or(1);

        for _ in 0..rounds {
            // 1. Standard parser (`parse_regular_expression: true, enable_ident_hashes: true`)
            let s = Instant::now();
            for _ in 0..parse_iters {
                let ret = Parser::new(&allocator, black_box(source_text), source_type)
                    .with_options(std_opts)
                    .parse();
                black_box(&ret);
                drop(ret);
                allocator.reset();
            }
            parse_std_samples.push(s.elapsed() / parse_iters_u32);

            // 2. Parser without identifier hash precomputation (`enable_ident_hashes: false`)
            let s = Instant::now();
            for _ in 0..parse_iters {
                let ret = Parser::new(&allocator, black_box(source_text), source_type)
                    .with_options(nohash_opts)
                    .parse();
                black_box(&ret);
                drop(ret);
                allocator.reset();
            }
            parse_nohash_samples.push(s.elapsed() / parse_iters_u32);

            // 3. Semantic analysis with precomputed `Stats` on a fresh AST every iteration.
            // Matches `tasks/benchmark/benches/semantic.rs` (`oxc-project/oxc#28467`):
            // - Fresh `Program` per iteration (since `SemanticBuilder` mutates AST `Cell`s)
            // - `Stats::count` excluded from timed region via `.with_stats(stats)`
            // - `Semantic` dropped inside timed region; `diagnostics` dropped outside
            let mut sem_elapsed = Duration::ZERO;
            for _ in 0..sem_iters {
                allocator.reset();
                let program = Parser::new(&allocator, source_text, source_type)
                    .with_options(std_opts)
                    .parse()
                    .program;
                let program = black_box(program);
                let s = Instant::now();
                let ret = SemanticBuilder::new_compiler().with_stats(stats).build(&program);
                let ret = black_box(ret);
                let diagnostics = ret.diagnostics;
                sem_elapsed += s.elapsed();
                drop(diagnostics);
            }
            allocator.reset();
            sem_with_stats_samples.push(sem_elapsed / sem_iters_u32);

            // 4. End-to-end per-module bundler pipeline (`parse -> semantic(excess 2.0) -> transform -> codegen`).
            // Models downstream bundler/HMR workloads (e.g. `developer-bundler` 10k-module & HMR updates)
            // where L1 instruction cache is shared across parser, semantic, transformer, and codegen,
            // and `SemanticBuilder` runs without precomputed `Stats` and with `with_excess_capacity(2.0)`.
            let s = Instant::now();
            for _ in 0..pipe_iters {
                let mut parsed = Parser::new(&allocator, black_box(source_text), source_type)
                    .with_options(std_opts)
                    .parse();
                let scoping = SemanticBuilder::new_compiler()
                    .with_excess_capacity(2.0)
                    .with_enum_eval(true)
                    .build(&parsed.program)
                    .semantic
                    .into_scoping();
                let _ = Transformer::new(&allocator, source_path, &transform_options)
                    .build_with_scoping(scoping, &mut parsed.program);
                let code = Codegen::new().build(&parsed.program);
                black_box(&code);
                drop(code);
                drop(parsed);
                allocator.reset();
            }
            bundler_pipeline_samples.push(s.elapsed() / pipe_iters_u32);
        }

        parse_std_samples.sort_unstable();
        parse_nohash_samples.sort_unstable();
        sem_with_stats_samples.sort_unstable();
        bundler_pipeline_samples.sort_unstable();

        println!(
            "RESULT|{}|{}|{}|{}|{}|{}",
            file.file_name,
            source_text.len(),
            parse_std_samples[0].as_nanos(),
            parse_nohash_samples[0].as_nanos(),
            sem_with_stats_samples[0].as_nanos(),
            bundler_pipeline_samples[0].as_nanos(),
        );
    }
}
