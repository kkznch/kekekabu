# keketrade (kabu) task runner

# Build the project
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format + lint + test
ci: fmt-check lint test

# Install the binary
install:
    cargo install --path .

# Clean build artifacts
clean:
    cargo clean
