[private]
default:
    @just --list
  
# Format all code
format:
    cargo +nightly fmt

# Update generated CLI help (cli/tests/cli-reference@.md.snap)
update-cli-reference:
    cargo insta test --accept --workspace -- test_generate_md_cli_help

# Preview documentation with live reloading
[group('docs')]
serve-docs:
    uv run mkdocs serve

# Build documentation into rendered-docs/ for offline use
[group('docs')]
build-docs:
    uv run mkdocs build -f mkdocs-offline.yml
