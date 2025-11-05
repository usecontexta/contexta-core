// Parsing performance benchmarks using Criterion
// Measures parsing speed for Python, TypeScript, and Rust files
// Run with: cargo bench --bench parsing

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use tempfile::TempDir;

use analyzer_python::parser::PythonParser;
use analyzer_typescript::parser::TypeScriptParser;
use analyzer_rust::parser::RustParser;
use analyzer_core::indexer::{
    IndexerConfig,
    discover_files,
    index_files_with_progress,
    index_files_with_progress_parallel,
};

// Sample Python code for benchmarking
const PYTHON_CODE: &str = r#"
import os
import sys
from typing import List, Dict, Optional

class DataProcessor:
    """Process data with various methods."""

    def __init__(self, config: Dict[str, any]):
        self.config = config
        self.results = []

    def process_item(self, item: Dict[str, any]) -> Optional[Dict[str, any]]:
        """Process a single item."""
        if not self._validate(item):
            return None

        processed = {
            'id': item.get('id'),
            'value': self._transform(item.get('value')),
            'timestamp': self._get_timestamp(),
        }

        self.results.append(processed)
        return processed

    def _validate(self, item: Dict[str, any]) -> bool:
        """Validate item structure."""
        required_keys = ['id', 'value']
        return all(key in item for key in required_keys)

    def _transform(self, value: any) -> any:
        """Transform value."""
        return str(value).upper()

    def _get_timestamp(self) -> int:
        """Get current timestamp."""
        import time
        return int(time.time())

def main():
    processor = DataProcessor({'debug': True})
    items = [{'id': i, 'value': f'item_{i}'} for i in range(100)]

    for item in items:
        processor.process_item(item)

    print(f"Processed {len(processor.results)} items")

if __name__ == '__main__':
    main()
"#;

// Sample TypeScript code for benchmarking
const TYPESCRIPT_CODE: &str = r#"
import { Request, Response } from 'express';
import { UserService } from './services/UserService';

interface User {
    id: string;
    name: string;
    email: string;
    createdAt: Date;
}

interface CreateUserRequest {
    name: string;
    email: string;
    password: string;
}

export class UserController {
    private userService: UserService;

    constructor(userService: UserService) {
        this.userService = userService;
    }

    async getUser(req: Request, res: Response): Promise<void> {
        try {
            const userId = req.params.id;
            const user = await this.userService.findById(userId);

            if (!user) {
                res.status(404).json({ error: 'User not found' });
                return;
            }

            res.json(user);
        } catch (error) {
            console.error('Error fetching user:', error);
            res.status(500).json({ error: 'Internal server error' });
        }
    }

    async createUser(req: Request, res: Response): Promise<void> {
        try {
            const data: CreateUserRequest = req.body;

            const validation = this.validateUserData(data);
            if (!validation.isValid) {
                res.status(400).json({ error: validation.error });
                return;
            }

            const user = await this.userService.create(data);
            res.status(201).json(user);
        } catch (error) {
            console.error('Error creating user:', error);
            res.status(500).json({ error: 'Internal server error' });
        }
    }

    private validateUserData(data: CreateUserRequest): { isValid: boolean; error?: string } {
        if (!data.name || data.name.length < 2) {
            return { isValid: false, error: 'Name must be at least 2 characters' };
        }

        if (!data.email || !data.email.includes('@')) {
            return { isValid: false, error: 'Invalid email address' };
        }

        if (!data.password || data.password.length < 8) {
            return { isValid: false, error: 'Password must be at least 8 characters' };
        }

        return { isValid: true };
    }
}
"#;

// Sample Rust code for benchmarking
const RUST_CODE: &str = r#"
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Debug)]
pub enum UserError {
    NotFound,
    InvalidInput(String),
    DatabaseError(String),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserError::NotFound => write!(f, "User not found"),
            UserError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            UserError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for UserError {}

pub struct UserRepository {
    users: HashMap<u64, User>,
    next_id: u64,
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, name: String, email: String) -> Result<User, UserError> {
        if name.is_empty() {
            return Err(UserError::InvalidInput("Name cannot be empty".to_string()));
        }

        if !email.contains('@') {
            return Err(UserError::InvalidInput("Invalid email address".to_string()));
        }

        let user = User {
            id: self.next_id,
            name,
            email,
        };

        self.users.insert(self.next_id, user.clone());
        self.next_id += 1;

        Ok(user)
    }

    pub fn find_by_id(&self, id: u64) -> Result<&User, UserError> {
        self.users.get(&id).ok_or(UserError::NotFound)
    }

    pub fn update(&mut self, id: u64, name: String, email: String) -> Result<User, UserError> {
        let user = self.users.get_mut(&id).ok_or(UserError::NotFound)?;

        if !name.is_empty() {
            user.name = name;
        }

        if !email.is_empty() && email.contains('@') {
            user.email = email;
        }

        Ok(user.clone())
    }

    pub fn delete(&mut self, id: u64) -> Result<(), UserError> {
        self.users.remove(&id).ok_or(UserError::NotFound)?;
        Ok(())
    }
}
"#;

/// Benchmark Python parser
fn bench_python_parsing(c: &mut Criterion) {
    let mut parser = PythonParser::new().expect("Failed to create Python parser");

    c.bench_function("parse_python_file", |b| {
        b.iter(|| {
            let tree = parser.parse(black_box(PYTHON_CODE)).expect("Parse failed");
            black_box(tree);
        })
    });
}

/// Benchmark TypeScript parser
fn bench_typescript_parsing(c: &mut Criterion) {
    let mut parser = TypeScriptParser::new().expect("Failed to create TypeScript parser");

    c.bench_function("parse_typescript_file", |b| {
        b.iter(|| {
            let tree = parser.parse(black_box(TYPESCRIPT_CODE)).expect("Parse failed");
            black_box(tree);
        })
    });
}

/// Benchmark Rust parser
fn bench_rust_parsing(c: &mut Criterion) {
    let mut parser = RustParser::new().expect("Failed to create Rust parser");

    c.bench_function("parse_rust_file", |b| {
        b.iter(|| {
            let tree = parser.parse(black_box(RUST_CODE)).expect("Parse failed");
            black_box(tree);
        })
    });
}

/// Benchmark sequential vs parallel indexing
fn bench_indexing_modes(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Create test files
    for i in 0..50 {
        fs::write(
            temp_path.join(format!("test{}.py", i)),
            PYTHON_CODE
        ).expect("Failed to write test file");
    }

    let config = IndexerConfig {
        root_dir: temp_path.to_path_buf(),
        ..Default::default()
    };

    let files = discover_files(&config).expect("Failed to discover files");

    let mut group = c.benchmark_group("indexing_modes");

    group.bench_with_input(
        BenchmarkId::new("sequential", files.len()),
        &files,
        |b, files| {
            b.iter(|| {
                let callback = Box::new(|_: usize, _: usize| {});
                index_files_with_progress(black_box(files), callback)
                    .expect("Indexing failed");
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", files.len()),
        &files,
        |b, files| {
            b.iter(|| {
                let callback = Box::new(|_: usize, _: usize| {});
                index_files_with_progress_parallel(black_box(files), callback)
                    .expect("Indexing failed");
            })
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    bench_python_parsing,
    bench_typescript_parsing,
    bench_rust_parsing,
    bench_indexing_modes
);
criterion_main!(benches);
