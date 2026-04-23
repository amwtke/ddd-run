# ddd-run

> A tiny Rust CLI that bootstraps a **DDD + Superpowers harness** for Claude Code projects in one command.

`ddd-run init` installs three Claude Code skills (`ddd-storm`, `ddd-model`, `ddd-spec`) plus two anchor documents (`CLAUDE.md`, `DOMAIN.md`) into your project, giving you a structured workflow from **fuzzy business description → domain model → spec → TDD implementation**.

```
   业务需求
       ↓
  /ddd-storm  ──→ docs/ddd/01-event-storming-*.md     (event storming)
       ↓
  /ddd-model  ──→ DOMAIN.md                            (Single Source of Truth)
       ↓
  /ddd-spec   ──→ docs/specs/spec-*.md                 (Superpowers-ready spec)
       ↓
  Superpowers ──→ tests + implementation               (TDD)
```

## Why this exists

When you let an AI coding assistant (Claude Code, Cursor, Codex) write a backend system from scratch, it usually produces:
- anemic domain models (all logic in Services, entities are dumb data bags)
- inconsistent naming (one concept, three different class names)
- cross-aggregate transactions and broken invariants
- "one-shot" megafiles that are impossible to iterate on

This CLI doesn't solve that by rewriting what the AI produces. It solves it **by constraining the AI _before_ it starts writing code**. The three skills force the AI through a strategic modeling phase (DDD), and the two anchor docs (`CLAUDE.md` + `DOMAIN.md`) keep every later decision consistent.

It's a thin tool — the real value is in the templates it ships.

## Install

### From source (requires Rust)

```bash
git clone https://github.com/amwtke/ddd-run
cd ddd-run
cargo install --path .
```

After install, `ddd-run` is on your `$PATH`.

### Pre-built binary (if available)

```bash
# macOS / Linux
curl -L https://github.com/amwtke/ddd-run/releases/latest/download/ddd-run-$(uname -s)-$(uname -m) -o /usr/local/bin/ddd-run
chmod +x /usr/local/bin/ddd-run
```

## Usage

### Bootstrap a project

```bash
cd your-new-project/
ddd-run init
```

This creates:

```
your-new-project/
├── .claude/
│   └── skills/
│       ├── ddd-storm/SKILL.md    # 🔍 event storming
│       ├── ddd-model/SKILL.md    # 🏗 domain modeling
│       └── ddd-spec/SKILL.md     # 📝 spec generation (→ Superpowers)
├── CLAUDE.md                     # 🛡 project-level Claude Code rules
├── DOMAIN.md                     # 📘 domain model Single Source of Truth
├── docs/
│   ├── ddd/                      # event-storming notes
│   └── specs/                    # spec outputs
└── README-DDD-HARNESS.md         # in-project guide
```

### Flags

```bash
ddd-run init --force       # overwrite existing files
ddd-run init --minimal     # install only skills, no CLAUDE.md / DOMAIN.md
ddd-run init --dir ./api   # initialize a subdirectory
```

### Check harness status

```bash
ddd-run status
```

Prints a green/red checklist of every required asset.

## The three skills

### 🔍 `/ddd-storm <business description>`
Event storming. Extracts **actors, commands, domain events, external systems, aggregate candidates** from a fuzzy requirement. **No code, no technical terms, only business language.** Output: `docs/ddd/01-event-storming-*.md`.

### 🏗 `/ddd-model`
Domain modeling. Reads the event-storming output, applies the Entity/VO/Aggregate decision tree, draws aggregate boundaries, builds the **Ubiquitous Language table**, and **updates `DOMAIN.md`** (the single source of truth for the whole project).

### 📝 `/ddd-spec <use case>`
Spec bridge to Superpowers. Reads `DOMAIN.md`, produces a spec with **Given-When-Then scenarios** that Superpowers can directly consume for TDD. All naming is anchored to `DOMAIN.md`'s ubiquitous language — guaranteeing the implementation uses the same terms as the model.

## The two anchor documents

### 🛡 `CLAUDE.md` — project-level AI rules
Hard rules for Claude Code in this project: layered architecture, rich domain model (no anemia), one-aggregate-per-transaction, Repository only for aggregate roots, TDD cadence, package structure. **Any code the AI generates is checked against these rules.**

### 📘 `DOMAIN.md` — domain model Single Source of Truth
The authoritative model: bounded context, **ubiquitous language table**, aggregate definitions, invariants, domain events, ADRs. **Every class name, method name, and test description in the codebase must reference this document.** Managed exclusively by `/ddd-model`.

## Who is this for

- Architects / senior engineers who use Claude Code / Cursor daily and are tired of steering the AI away from anemic code every five minutes
- Teams adopting DDD who want AI assistance without losing modeling discipline
- People preparing for **AI4SE / harness-engineering-style interviews** (the setup demonstrates exactly the AI-as-constrained-execution-engine mindset those interviews look for)

## How it relates to Superpowers

`ddd-run` and [Superpowers](https://github.com/obra/superpowers) are complementary:

| Phase | Owner |
|---|---|
| Strategic modeling (What is this domain?) | **ddd-run** (`/ddd-storm` → `/ddd-model`) |
| Use case decomposition (What's the next unit?) | **ddd-run** (`/ddd-spec`) |
| Tactical implementation (Write tests, write code) | **Superpowers** (TDD loop) |

`/ddd-spec`'s output format is designed to drop straight into Superpowers' spec workflow.

## Project structure

```
ddd-run/
├── Cargo.toml
├── README.md                        # this file
└── src/
    ├── main.rs                      # CLI entrypoint (clap)
    ├── commands/
    │   ├── mod.rs
    │   ├── init.rs                  # `ddd-run init`
    │   └── status.rs                # `ddd-run status`
    └── templates/                   # embedded via include_str!
        ├── skills/
        │   ├── ddd-storm.md
        │   ├── ddd-model.md
        │   └── ddd-spec.md
        └── root/
            ├── CLAUDE.md
            ├── DOMAIN.md
            └── README-DDD-HARNESS.md
```

All templates are compiled into the binary (`include_str!`), so the CLI has **zero runtime file dependencies** — you can copy the binary anywhere and it just works.

## License

MIT

## Credits

Inspired by Anthropic's [harness-design-long-running-apps](https://www.anthropic.com/engineering/harness-design-long-running-apps) and the general "AI as a constrained execution engine" mindset.
