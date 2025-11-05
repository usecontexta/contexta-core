// Performance benchmarks for Contexta indexing operations
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Mock implementations for benchmarking
// (Replace with actual imports when running against real code)

fn create_test_python_file(dir: &TempDir, size: usize) -> PathBuf {
    let path = dir.path().join(format!("test_{}.py", size));
    let content = format!(
        r#"
# Test Python file for benchmarking
import os
import sys

def example_function_{}():
    """Example function for parsing benchmark"""
    result = []
    for i in range({}):
        result.append(i * 2)
    return result

class ExampleClass{}:
    """Example class for parsing benchmark"""
    def __init__(self):
        self.value = {}

    def method(self):
        return self.value * 2
"#,
        size, size, size, size
    );

    fs::write(&path, content).unwrap();
    path
}

fn benchmark_file_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_discovery");

    for num_files in [10, 100, 1000].iter() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        for i in 0..*num_files {
            create_test_python_file(&temp_dir, i);
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(num_files),
            num_files,
            |b, _| {
                b.iter(|| {
                    // Benchmark file discovery
                    let files: Vec<PathBuf> = fs::read_dir(temp_dir.path())
                        .unwrap()
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .collect();

                    black_box(files);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_file_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_parsing");

    for file_size in [100, 500, 1000].iter() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_python_file(&temp_dir, *file_size);
        let content = fs::read_to_string(&file_path).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(file_size),
            file_size,
            |b, _| {
                b.iter(|| {
                    // Benchmark parsing operation
                    // In real benchmarks, this would use tree-sitter parsing
                    let lines: Vec<&str> = content.lines().collect();
                    black_box(lines);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_metadata_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_extraction");

    for num_symbols in [10, 50, 100].iter() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_python_file(&temp_dir, *num_symbols);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_symbols),
            num_symbols,
            |b, _| {
                b.iter(|| {
                    // Benchmark metadata extraction
                    let metadata = fs::metadata(&file_path).unwrap();
                    black_box(metadata);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_file_discovery,
    benchmark_file_parsing,
    benchmark_metadata_extraction
);
criterion_main!(benches);
