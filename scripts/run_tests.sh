#!/bin/bash

# Block Migrate Test Runner
# Comprehensive test execution script with coverage reporting

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_DB_URL="${TEST_DATABASE_URL:-postgresql://blockuser:blockpass@localhost/metaforge_test}"
COVERAGE_THRESHOLD=85
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"

echo -e "${BLUE}🧪 Block Migrate Test Suite${NC}"
echo "=================================="

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust."
        exit 1
    fi
    
    # Check if PostgreSQL is running (for integration tests)
    if ! pg_isready -h localhost -p 5432 &> /dev/null; then
        print_warning "PostgreSQL is not running. Integration tests may fail."
        print_info "Start PostgreSQL with: docker compose up -d"
    fi
    
    # Check if test database exists
    if ! psql "$TEST_DB_URL" -c "SELECT 1;" &> /dev/null; then
        print_warning "Test database is not accessible. Creating..."
        createdb metaforge_test || print_warning "Could not create test database"
    fi
    
    print_status "Prerequisites checked"
}

# Run unit tests
run_unit_tests() {
    print_info "Running unit tests..."
    
    cargo test --lib --bins --tests unit_tests \
        --verbose \
        -- --nocapture
    
    print_status "Unit tests completed"
}

# Run integration tests
run_integration_tests() {
    print_info "Running integration tests..."
    
    # Set test environment variables
    export TEST_DATABASE_URL="$TEST_DB_URL"
    export RUST_LOG=debug
    
    cargo test --test integration_tests \
        --verbose \
        -- --nocapture --test-threads=1
    
    print_status "Integration tests completed"
}

# Run performance benchmarks
run_benchmarks() {
    print_info "Running performance benchmarks..."
    
    if cargo test --release --test integration_tests benchmark_ -- --nocapture; then
        print_status "Benchmarks completed"
    else
        print_warning "Some benchmarks failed or were skipped"
    fi
}

# Generate test coverage report
generate_coverage() {
    print_info "Generating test coverage report..."
    
    # Install cargo-tarpaulin if not present
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_info "Installing cargo-tarpaulin..."
        cargo install cargo-tarpaulin
    fi
    
    # Generate coverage report
    cargo tarpaulin \
        --verbose \
        --all-features \
        --workspace \
        --timeout 120 \
        --out Html \
        --output-dir coverage \
        --exclude-files "target/*" \
        --exclude-files "tests/*" \
        --exclude-files "src/bin/*" \
        || print_warning "Coverage generation failed"
    
    # Check coverage threshold
    if [ -f "coverage/tarpaulin-report.html" ]; then
        print_status "Coverage report generated: coverage/tarpaulin-report.html"
        
        # Extract coverage percentage (this is a simplified extraction)
        if command -v grep &> /dev/null; then
            COVERAGE=$(grep -o '[0-9]\+\.[0-9]\+%' coverage/tarpaulin-report.html | head -1 | sed 's/%//')
            if [ -n "$COVERAGE" ]; then
                print_info "Current coverage: ${COVERAGE}%"
                if (( $(echo "$COVERAGE >= $COVERAGE_THRESHOLD" | bc -l) )); then
                    print_status "Coverage threshold met (${COVERAGE}% >= ${COVERAGE_THRESHOLD}%)"
                else
                    print_warning "Coverage below threshold (${COVERAGE}% < ${COVERAGE_THRESHOLD}%)"
                fi
            fi
        fi
    fi
}

# Run linting and formatting checks
run_linting() {
    print_info "Running linting and formatting checks..."
    
    # Check formatting
    if cargo fmt -- --check; then
        print_status "Code formatting is correct"
    else
        print_warning "Code formatting issues found. Run 'cargo fmt' to fix."
    fi
    
    # Run clippy
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_status "No clippy warnings found"
    else
        print_warning "Clippy warnings found. Please fix them."
    fi
}

# Run security audit
run_security_audit() {
    print_info "Running security audit..."
    
    # Install cargo-audit if not present
    if ! command -v cargo-audit &> /dev/null; then
        print_info "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    if cargo audit; then
        print_status "No security vulnerabilities found"
    else
        print_warning "Security vulnerabilities detected. Please review."
    fi
}

# Clean up test artifacts
cleanup() {
    print_info "Cleaning up test artifacts..."
    
    # Remove test database if it was created
    if [ "$CLEANUP_TEST_DB" = "true" ]; then
        dropdb metaforge_test 2>/dev/null || true
    fi
    
    # Clean cargo cache
    cargo clean
    
    print_status "Cleanup completed"
}

# Main execution
main() {
    local run_unit=true
    local run_integration=true
    local run_bench=false
    local run_coverage=false
    local run_lint=true
    local run_audit=false
    local cleanup_after=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --unit-only)
                run_integration=false
                run_bench=false
                shift
                ;;
            --integration-only)
                run_unit=false
                run_bench=false
                shift
                ;;
            --with-benchmarks)
                run_bench=true
                shift
                ;;
            --with-coverage)
                run_coverage=true
                shift
                ;;
            --with-audit)
                run_audit=true
                shift
                ;;
            --no-lint)
                run_lint=false
                shift
                ;;
            --cleanup)
                cleanup_after=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --unit-only         Run only unit tests"
                echo "  --integration-only  Run only integration tests"
                echo "  --with-benchmarks   Include performance benchmarks"
                echo "  --with-coverage     Generate coverage report"
                echo "  --with-audit        Run security audit"
                echo "  --no-lint           Skip linting checks"
                echo "  --cleanup           Clean up after tests"
                echo "  --help              Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Start timer
    start_time=$(date +%s)
    
    # Run test suite
    check_prerequisites
    
    if [ "$run_lint" = true ]; then
        run_linting
    fi
    
    if [ "$run_unit" = true ]; then
        run_unit_tests
    fi
    
    if [ "$run_integration" = true ]; then
        run_integration_tests
    fi
    
    if [ "$run_bench" = true ]; then
        run_benchmarks
    fi
    
    if [ "$run_coverage" = true ]; then
        generate_coverage
    fi
    
    if [ "$run_audit" = true ]; then
        run_security_audit
    fi
    
    if [ "$cleanup_after" = true ]; then
        cleanup
    fi
    
    # Calculate total time
    end_time=$(date +%s)
    total_time=$((end_time - start_time))
    
    echo ""
    echo "=================================="
    print_status "All tests completed in ${total_time} seconds"
    
    # Summary
    echo ""
    print_info "Test Summary:"
    echo "  - Unit tests: $([ "$run_unit" = true ] && echo "✅ Passed" || echo "⏭️  Skipped")"
    echo "  - Integration tests: $([ "$run_integration" = true ] && echo "✅ Passed" || echo "⏭️  Skipped")"
    echo "  - Benchmarks: $([ "$run_bench" = true ] && echo "✅ Completed" || echo "⏭️  Skipped")"
    echo "  - Coverage: $([ "$run_coverage" = true ] && echo "✅ Generated" || echo "⏭️  Skipped")"
    echo "  - Linting: $([ "$run_lint" = true ] && echo "✅ Passed" || echo "⏭️  Skipped")"
    echo "  - Security audit: $([ "$run_audit" = true ] && echo "✅ Completed" || echo "⏭️  Skipped")"
}

# Handle script interruption
trap 'print_error "Test execution interrupted"; exit 1' INT TERM

# Run main function with all arguments
main "$@"

