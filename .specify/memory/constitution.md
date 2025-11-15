<!--
SYNC IMPACT REPORT
==================
Version change: N/A (initial) → 1.0.0
Modified principles: N/A (initial creation)
Added sections:
  - Core Principles (4 principles)
  - Quality Gates
  - Development Workflow
  - Governance
Removed sections: None
Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ No changes needed (Constitution Check section is generic)
  - .specify/templates/spec-template.md: ✅ No changes needed (requirements align)
  - .specify/templates/tasks-template.md: ✅ No changes needed (task structure supports principles)
Follow-up TODOs: None
-->

# 3D Assistant Constitution

## Core Principles

### I. Code Quality First

All code MUST meet established quality standards before merge:

- **Static Analysis**: Code MUST pass linting with zero warnings. Unused imports, dead code, and style violations are blockers.
- **Type Safety**: Strong typing MUST be used where the language supports it. Any type coercion MUST be explicit and justified.
- **Code Reviews**: Every pull request MUST receive at least one approval from a team member who did not author the code.
- **Documentation**: Public APIs MUST include docstrings/comments explaining purpose, parameters, return values, and edge cases.
- **Single Responsibility**: Functions MUST do one thing well. Maximum function length is 50 lines; violations require explicit justification in PR description.
- **No Magic Numbers**: All constants MUST be named and documented.

**Rationale**: Code quality is the foundation of maintainability. Technical debt accumulates exponentially when quality is compromised.

### II. Testing Standards (NON-NEGOTIABLE)

Testing is mandatory, not optional:

- **Coverage Threshold**: Minimum 80% code coverage for all new features. Critical paths MUST have 95%+ coverage.
- **Test Types Required**:
  - Unit tests for all business logic
  - Integration tests for API endpoints and service boundaries
  - Contract tests for external dependencies
- **Test-Driven Development**: For new features, tests SHOULD be written before implementation. Red-Green-Refactor cycle is strongly encouraged.
- **Test Naming**: Tests MUST clearly describe the scenario being tested: `test_[unit]_[scenario]_[expected_outcome]`
- **No Mocking Abuse**: Mocks are for external dependencies only. Internal logic MUST NOT be mocked in unit tests.
- **CI/CD Gate**: All tests MUST pass in CI before merge. Flaky tests MUST be fixed or quarantined immediately.

**Rationale**: Tests are the safety net that enables confident refactoring and prevents regressions. Without tests, every change carries unknown risk.

### III. User Experience Consistency

User-facing interfaces MUST provide predictable, coherent experiences:

- **Response Time**: User interactions MUST provide feedback within 200ms. Long operations MUST show progress indicators.
- **Error Messages**: Errors MUST be user-friendly, actionable, and logged with context for debugging. Never expose stack traces to end users.
- **Accessibility**: All UI components MUST meet WCAG 2.1 AA standards minimum.
- **Consistent Patterns**: Similar actions MUST behave similarly across the application. Navigation patterns, button placement, and terminology MUST be uniform.
- **State Management**: Application state MUST be predictable. Users MUST NOT lose work due to unexpected state changes.
- **Graceful Degradation**: Features MUST fail gracefully. Partial failures MUST NOT break the entire application.

**Rationale**: Users judge software by their experience, not by code elegance. Inconsistent UX erodes trust and increases support burden.

### IV. Performance Requirements

Performance is a feature, not an afterthought:

- **Response Time Targets**:
  - API endpoints: p95 < 200ms for reads, p95 < 500ms for writes
  - UI interactions: < 100ms for feedback, < 1s for navigation
  - Background jobs: MUST complete within SLA or provide progress updates
- **Resource Constraints**:
  - Memory usage MUST NOT exceed allocated limits
  - CPU spikes MUST be temporary and justified
  - Database queries MUST be optimized (no N+1 queries, proper indexing)
- **Monitoring**: All performance-critical paths MUST be instrumented with metrics
- **Load Testing**: Features handling concurrent users MUST be load tested before release
- **Performance Budgets**: Bundle sizes, query counts, and memory allocations MUST have defined budgets that block merge if exceeded

**Rationale**: Poor performance directly impacts user satisfaction and system costs. Performance degradation is cumulative and expensive to fix retroactively.

## Quality Gates

All code MUST pass these gates before merge:

1. **Static Analysis**: Zero linting errors, type checking passes
2. **Test Suite**: All tests pass, coverage thresholds met
3. **Performance**: Benchmarks within acceptable ranges
4. **Security**: No high/critical vulnerabilities in dependencies
5. **Documentation**: API documentation updated, README reflects changes
6. **Code Review**: Approved by qualified reviewer

## Development Workflow

1. **Branch Strategy**: Feature branches from main, descriptive names (`feature/`, `fix/`, `refactor/`)
2. **Commit Standards**: Conventional commits enforced (feat:, fix:, refactor:, test:, docs:)
3. **Pull Request Process**:
   - Clear description of changes and motivation
   - Link to relevant issues/specifications
   - Self-review checklist completed
   - All CI checks passing
4. **Definition of Done**:
   - Code implements requirements
   - Tests written and passing
   - Documentation updated
   - Performance validated
   - Peer reviewed and approved

## Governance

This constitution establishes non-negotiable standards for the 3D Assistant project:

- **Supremacy**: Constitution principles supersede individual preferences. Violations MUST be justified and documented.
- **Amendment Process**:
  1. Propose change with rationale
  2. Team review and discussion
  3. Consensus or majority approval
  4. Document migration plan if breaking change
  5. Update version following semantic versioning
- **Compliance Review**: PRs MUST verify alignment with constitution principles. Reviewers MUST check constitution compliance.
- **Versioning Policy**:
  - MAJOR: Principle removal or incompatible redefinition
  - MINOR: New principle or material expansion
  - PATCH: Clarification or wording improvement
- **Exceptions**: Temporary exceptions require documented justification, approval, and remediation plan with timeline.

**Version**: 1.0.0 | **Ratified**: 2025-11-15 | **Last Amended**: 2025-11-15
