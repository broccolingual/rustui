# Contributing to rustui

Thank you for your interest in contributing to rustui! We welcome contributions of all kinds, from bug reports to feature implementations.

## Code of Conduct

This project adheres to a simple principle: be respectful and constructive in all interactions.

## Development Guidelines

### Core Principles

1. **Simplicity First**: Keep APIs simple and intuitive
2. **Safety**: No unsafe code unless absolutely necessary
3. **Backward Compatibility**: Maintain compatibility whenever possible
4. **Performance**: Efficient rendering with minimal overhead

### Code Standards

#### Line Length and Formatting
- **Maximum line length: 100**
- Use `cargo fmt` before submitting
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

#### API Design
- Keep public APIs minimal and focused
- Prefer composition over inheritance
- Use Result types for fallible operations
- Document all public APIs with examples

#### Safety Requirements
- **No unsafe code** without explicit justification and review
- Use Rust's type system to prevent runtime errors
- Prefer compile-time checks over runtime validation

#### Backward Compatibility
- Breaking changes require major version bump
- Deprecate features before removal (use `#[deprecated]`)
- Provide migration paths in documentation

### Code Organization

#### File Structure
- Keep modules focused and cohesive
- Maximum 500 lines per file (exceptions require justification)
- Use clear, descriptive naming for modules and files

#### Function Guidelines
- Maximum 50 lines per function
- Single responsibility principle
- Clear, descriptive names
- Comprehensive documentation for public functions

#### Testing
- Unit tests for all public APIs

## Contributing Process

### 1. Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/broccolingual/rustui.git
cd rustui

# Install dependencies
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --check
```

### 2. Making Changes

#### For Bug Fixes
1. Create an issue describing the bug
2. Create a feature branch: `git checkout -b fix/issue-description`
3. Write a failing test that reproduces the bug
4. Fix the bug
5. Ensure all tests pass
6. Submit a pull request

#### For New Features
1. Create an issue for discussion
2. Get approval from maintainers
3. Create a feature branch: `git checkout -b feature/feature-name`
4. Implement the feature following guidelines
5. Add comprehensive tests
6. Update documentation
7. Submit a pull request

### 3. Pull Request Guidelines

#### Requirements
- [ ] Code follows formatting standards (`cargo fmt`)
- [ ] All tests pass (`cargo test`)
- [ ] New features have tests
- [ ] Documentation is updated
- [ ] Backward compatibility is maintained

#### PR Description Template
```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix (non-breaking change)
- [ ] New feature (non-breaking change)
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Backward Compatibility
- [ ] No breaking changes
- [ ] Breaking changes documented
- [ ] Migration guide provided

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
```

### 4. Code Review Process

#### Review Criteria
- Functionality and correctness
- Code style and organization
- Documentation completeness
- Performance implications
- Backward compatibility impact

## Specific Areas for Contribution

### High Priority
- Bug fixes in core functionality
- Performance improvements
- Documentation improvements

### Medium Priority
- New rendering features
- Input handling improvements
- Cross-platform compatibility
- Example applications
- Testing infrastructure

### Low Priority
- Code refactoring
- Developer tooling
- CI/CD improvements

## Documentation Standards

### Code Documentation
- All public APIs must have rustdoc comments
- Document panics, errors, and safety considerations
- Use proper markdown formatting

### Examples
- Focus on clarity over complexity
- Include comments explaining key concepts

## Release Process

### Version Numbering
- Follow semantic versioning (SemVer)
- Major: Breaking changes
- Minor: New features (backward compatible)
- Patch: Bug fixes (backward compatible)

### Deprecation Policy
- Features marked as deprecated will be removed in the next major version
- Deprecation warnings must include replacement suggestions

## Getting Help

### Communication Channels
- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: General questions and ideas
- Pull Request comments: Code-specific discussions

## License

By contributing to rustui, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to rustui! Your efforts help make terminal UI development in Rust more accessible and enjoyable.
