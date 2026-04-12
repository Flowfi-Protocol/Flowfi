# Contributing to FlowFi Protocol

Thank you for considering a contribution to FlowFi. This document covers everything you need to know to go from "interested" to "merged PR."

---

## Table of Contents

1. [Project Philosophy](#project-philosophy)
2. [Code of Conduct](#code-of-conduct)
3. [How to Find Something to Work On](#how-to-find-something-to-work-on)
4. [Local Environment Setup](#local-environment-setup)
5. [Running Tests](#running-tests)
6. [Code Standards](#code-standards)
7. [Branching Strategy](#branching-strategy)
8. [Pull Request Guidelines](#pull-request-guidelines)
9. [Issue Labels](#issue-labels)
10. [Design Discussions](#design-discussions)
11. [Onboarding Path for New Contributors](#onboarding-path-for-new-contributors)

---

## Project Philosophy

FlowFi is built on three principles:

**1. Openness over completeness.**
We prefer shipping a working but incomplete protocol and inviting contributors to build it out, rather than building everything behind closed doors and open-sourcing a finished product. This means the codebase is intentionally incomplete in well-labeled places.

**2. Education as a feature.**
Every TODO, every comment, every architectural decision should be legible to someone learning Soroban. Code that is clever but opaque is a bug. Code that is slightly verbose but clearly explained is preferred.

**3. Protocol correctness over gas optimization.**
This is not an Ethereum mainnet protocol. Soroban fees are low. We will not sacrifice safety or clarity for micro-optimizations. When in doubt, add a check.

---

## Code of Conduct

FlowFi follows the [Contributor Covenant](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). In short: be respectful, be constructive, be patient. Aggressive, dismissive, or exclusionary behavior will result in removal from the project.

---

## How to Find Something to Work On

### 1. Browse labeled issues

Issues are labeled by difficulty. Start here:

- [`good-first-issue`](../../issues?q=is%3Aopen+label%3Agood-first-issue): Focused, well-scoped tasks ideal for first contributions
- [`intermediate`](../../issues?q=is%3Aopen+label%3Aintermediate): Require understanding multiple parts of the codebase
- [`advanced`](../../issues?q=is%3Aopen+label%3Aadvanced): Protocol design, security hardening, or significant new features
- [`protocol-design`](../../issues?q=is%3Aopen+label%3Aprotocol-design): Architectural decisions — start with a discussion before coding
- [`security`](../../issues?q=is%3Aopen+label%3Asecurity): Require careful analysis; please post your threat model as a comment before submitting a fix

### 2. Search for TODOs

The codebase has TODO comments throughout. Each one is a potential contribution:

```bash
# Find all TODOs
grep -rn "TODO" contracts/ sdk/
```

If a TODO doesn't have a corresponding GitHub issue yet, open one! This itself is a valuable contribution.

### 3. Improve tests

Test coverage is incomplete by design. Adding tests to any contract is always welcome and a great way to understand the codebase.

### 4. Improve documentation

Clear documentation has the highest leverage of any contribution. If you read something and had to re-read it twice to understand, that's a documentation bug.

---

## Local Environment Setup

### Requirements

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable | `rustup install stable` |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | >= 20.x | `cargo install stellar-cli --features opt` |
| Node.js | >= 18 | [nodejs.org](https://nodejs.org) |
| npm | >= 9 | Comes with Node.js |

### Setup Steps

```bash
# 1. Fork the repository on GitHub, then clone your fork
git clone https://github.com/<your-username>/flowfi.git
cd flowfi

# 2. Add the upstream remote
git remote add upstream https://github.com/flowfi-protocol/flowfi.git

# 3. Build contracts
cargo build --target wasm32-unknown-unknown --release --workspace

# 4. Set up the SDK
cd sdk
npm install
npm run build
cd ..

# 5. Copy environment config
cp .env.example .env
# Edit .env with your testnet details (only needed for deployment/integration tests)
```

---

## Running Tests

### Smart contract unit tests

```bash
# Run all contract tests
cargo test --workspace

# Run tests for a specific contract
cargo test -p flowfi-vault

# Run a specific test by name
cargo test -p flowfi-vault test_deposit_and_withdraw

# Run with output (useful for debugging)
cargo test --workspace -- --nocapture
```

### SDK tests

```bash
cd sdk
npm test

# Watch mode during development
npm test -- --watch
```

### Integration tests

```bash
# Integration tests require a running Soroban environment
# See tests/README.md for setup instructions

cd tests
npm install
npm test
```

---

## Code Standards

### Rust (Smart Contracts)

**Formatting:** All Rust code must be formatted with `rustfmt`. Run before committing:

```bash
cargo fmt --all
```

**Linting:** Code must pass `clippy` with no warnings:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

**Style guidelines:**
- Use descriptive variable names. `shares_to_mint` over `s`.
- Every public function must have a doc comment (`///`).
- Every `panic!()` must have a clear message explaining what invariant was violated.
- Use `checked_mul` / `saturating_mul` for arithmetic. Never assume no overflow.
- New storage keys must be added to the `DataKey` enum with a comment explaining their purpose.
- Tests should follow the `Arrange → Act → Assert` pattern.
- Test function names should be `test_<function>_<scenario>`, e.g. `test_deposit_zero_amount_panics`.

**What we don't do:**
- No `unwrap()` in production paths (only in tests). Use `.expect("descriptive message")`.
- No `unsafe` code.
- No `println!` in contract code (use events).

### TypeScript (SDK)

**Formatting:** Use Prettier. Run before committing:

```bash
cd sdk && npm run format
```

**Linting:**

```bash
cd sdk && npm run lint
```

**Style guidelines:**
- All public methods and types must have JSDoc comments.
- Use `bigint` for all token amounts. Never `number` for on-chain values.
- Error messages should be actionable: `"deposit amount must be positive"` not `"invalid input"`.
- All RPC calls must have try/catch with typed error handling.

---

## Branching Strategy

We use a simple trunk-based development model:

| Branch | Purpose |
|--------|---------|
| `main` | Stable, reviewed code. Direct pushes not allowed. |
| `develop` | Integration branch for upcoming release |
| `feat/<name>` | Feature branches (your PRs target `develop`) |
| `fix/<name>` | Bug fix branches |
| `docs/<name>` | Documentation-only changes |
| `security/<name>` | Security fixes (coordinate with maintainers first) |

**Branch naming examples:**
- `feat/multi-token-rewards`
- `fix/vault-share-price-precision`
- `docs/sdk-jsdoc-coverage`
- `test/vault-edge-cases`

---

## Pull Request Guidelines

### Before Opening a PR

- [ ] Your branch is based on the latest `develop`
- [ ] `cargo fmt --all` passes with no changes
- [ ] `cargo clippy --workspace` passes with no warnings
- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] New functionality has tests
- [ ] Public functions have doc comments
- [ ] If your PR changes protocol behavior, it has a corresponding note in `docs/`

### PR Description Template

```markdown
## Summary
What does this PR do? (2–3 sentences)

## Motivation
Why is this change needed? Link to the relevant issue: Closes #<issue-number>

## Changes
- Contract X: Added Y
- SDK: Updated Z
- Tests: Added coverage for W

## Testing
How did you verify this works? What edge cases did you test?

## Open Questions
Anything you're unsure about or want reviewers to focus on?
```

### Review Process

1. Open a draft PR early if you want early feedback
2. At least one maintainer review is required to merge
3. Security-related PRs require two reviews
4. We aim to provide initial feedback within 5 business days
5. PRs that sit without updates for 3 weeks may be closed (but can be reopened)

### What Makes a Great PR

- **Small and focused**: One concern per PR. If you find yourself writing "also..." in the summary, it should probably be a separate PR.
- **Test-first thinking**: Show us you understand the edge cases, not just the happy path.
- **Explains the why**: Comments in code, description in PR — we want to understand your reasoning.
- **Iterates gracefully**: Responds to review feedback constructively and promptly.

---

## Issue Labels

| Label | Meaning |
|-------|---------|
| `good-first-issue` | Self-contained, well-scoped, ideal for first PRs |
| `intermediate` | Requires understanding multiple modules |
| `advanced` | Significant new feature or protocol change |
| `security` | Potential security impact — handle with care |
| `protocol-design` | Architectural question — start with discussion |
| `testing` | Improving test coverage |
| `documentation` | Docs, comments, examples |
| `sdk` | TypeScript SDK changes |
| `contracts` | Smart contract changes |
| `dx` | Developer experience improvements |
| `bug` | Something is broken |
| `research` | Exploratory — output is a design doc, not code |
| `help-wanted` | Maintainers want community input |

---

## Design Discussions

For `protocol-design` or `advanced` issues, **please do not open a code PR before the design is agreed upon**. The process is:

1. Open a GitHub Discussion (or comment on the existing issue) with your proposed design
2. Include: motivation, proposed approach, alternatives considered, risks
3. Wait for maintainer sign-off (or meaningful community consensus) before writing code
4. Reference the discussion in your eventual PR

This saves everyone time. A 200-line design comment is much cheaper to revise than a 2000-line PR.

---

## Onboarding Path for New Contributors

If you're new to Soroban, Web3, or open source in general, here's a recommended path:

### Week 1 — Orient
- Read [docs/architecture.md](docs/architecture.md) in full
- Read [docs/contracts.md](docs/contracts.md)
- Clone the repo and get the build working locally
- Run `cargo test --workspace` and see everything green
- Skim the four contract files, reading all comments

### Week 2 — First Touch
- Pick a `good-first-issue` that interests you
- Comment on the issue: "I'd like to work on this"
- Set up a branch and start exploring the relevant code
- Open a draft PR early — don't wait until it's perfect

### Week 3 — First PR
- Address any review feedback
- Get your PR merged 🎉
- Leave a comment on the issue you'd want to tackle next

### Helpful Resources

- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Soroban by Example](https://soroban.stellar.org/docs/learn/migrate)
- [Stellar Developer Discord](https://discord.gg/stellardev)
- [Rust Book](https://doc.rust-lang.org/book/) — especially chapters on ownership and enums
- [FlowFi Architecture Doc](docs/architecture.md)

---

## Questions?

- **GitHub Discussions**: For design questions and longer conversations
- **GitHub Issues**: For bug reports and feature requests
- **Discord**: [Join the FlowFi server](#) for real-time chat

We're glad you're here. Let's build something great for the Stellar ecosystem together.
