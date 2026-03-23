<p align="center">
  <img src="helixir-logo.jpeg" alt="Helixir" width="320"/>
</p>

<h1 align="center">Helixir</h1>

<p align="center">
  Graph-based persistent memory for LLM agents.<br/>
  Associative recall, causal reasoning, ontology classification — out of the box.
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#how-it-works">How It Works</a> &middot;
  <a href="#mcp-tools">MCP Tools</a> &middot;
  <a href="#configuration">Configuration</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.83+-orange?logo=rust" alt="Rust 1.83+"/>
  <img src="https://img.shields.io/badge/MCP-compatible-4c8bf5?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiPjwvc3ZnPg==" alt="MCP"/>
  <img src="https://img.shields.io/badge/license-MIT-green" alt="MIT License"/>
  <img src="https://img.shields.io/badge/HelixDB-graph%20%2B%20vector-blueviolet" alt="HelixDB"/>
</p>

---

## What is Helixir?

Helixir gives AI agents **memory that persists between sessions**. When an agent powered by Helixir starts a new conversation, it recalls past decisions, preferences, goals, and reasoning chains — not from a flat log, but from a **graph of interconnected facts**.

Every piece of information is LLM-extracted into atomic facts, classified by ontology (skill, preference, goal, fact, opinion, experience, achievement, action), linked to named entities, and stored with vector embeddings for semantic search. Duplicate detection, contradiction tracking, and supersession happen automatically.

Built on [HelixDB](https://github.com/HelixDB/helix-db) (graph + vector database) with native [MCP](https://modelcontextprotocol.io/) support for Cursor, Claude Desktop, and any MCP-compatible client.

### Why this approach?

| Flat memory (key-value, embeddings only) | Helixir (graph + vector + ontology) |
|:-----------------------------------------|:------------------------------------|
| Retrieves similar text chunks | Retrieves facts **and their connections** |
| No deduplication — grows forever | Smart dedup: ADD / UPDATE / SUPERSEDE / NOOP |
| No reasoning trail | Causal chains: A BECAUSE B, A IMPLIES C |
| All memories equal | Ontology: skills vs preferences vs goals |
| Single-user | Cross-user: shared facts, conflict detection |

### By the numbers

| Metric | Value |
|:-------|:------|
| Memory node types | 15 (Memory, Entity, Concept, Context, ...) |
| Edge types | 33 (24 active in pipeline, 9 reserved for future features) |
| Ontology types | 8 (fact, preference, skill, goal, opinion, experience, achievement, action) |
| MCP tools | 14 |
| Search modes | 4 (recent/4h, contextual/30d, deep/90d, full) |
| Reasoning relations | 4 (IMPLIES, BECAUSE, CONTRADICTS, SUPPORTS) |
| Startup time | ~50ms |
| Memory footprint | ~15MB |
| Test coverage | 48 unit tests passing |

---

## Quick Start

### One-command install

```bash
curl -fsSL https://raw.githubusercontent.com/nickorulenko/helixir/main/install.sh | bash
```

The script will:
1. Check prerequisites (Rust, Docker)
2. Clone the repo and build from source
3. Start HelixDB via Docker
4. Deploy the graph schema
5. Generate MCP config for your IDE

Or install manually:

```bash
# Clone
git clone https://github.com/nickorulenko/helixir.git
cd helixir

# Build
make build

# Start HelixDB + deploy schema
make setup

# Show MCP config to paste into your IDE
make config
```

### Prerequisites

- **Rust 1.83+** — [rustup.rs](https://rustup.rs)
- **Docker** — for HelixDB ([install](https://docs.docker.com/get-docker/))
- **API key** — at least one LLM provider:
  - [Cerebras](https://cloud.cerebras.ai) (free tier, ~3000 tok/s)
  - [OpenAI](https://platform.openai.com/api-keys)
  - [Ollama](https://ollama.com) (local, no key needed)

---

## How It Works

```
               Input: "I deployed the server to AWS and prefer using Terraform"
                                          |
                                    LLM Extraction
                                          |
                          +---------------+---------------+
                          |                               |
                  Memory: "I deployed         Memory: "I prefer
                  the server to AWS"          using Terraform"
                  type: action                type: preference
                          |                               |
                    +-----+-----+                   +-----+-----+
                    |           |                   |           |
                Entity:     Entity:            Entity:      Concept:
                "AWS"       "server"           "Terraform"  Preference
                          |
                    Phase 1: Personal search (dedup check)
                    Phase 2: Cross-user search (shared facts)
                          |
                    Decision: ADD / UPDATE / SUPERSEDE / NOOP
                          |
                    Store in HelixDB (graph + vector)
```

### Architecture

```
MCP Server (stdio)                        IDE (Cursor / Claude Desktop)
       |                                           |
  HelixirClient                               MCP Protocol
       |
  ToolingManager ──── FastThinkManager
       |                    |
  +----+----+----+     petgraph (in-memory)
  |    |    |    |          |
Extract Decision Entity  commit to DB
  |    Engine  Manager       |
Search    |    Ontology      |
Engine  Reasoning Manager    |
  |    Engine    |           |
  +----+----+----+-----------+
       |
  HelixDB Client (HTTP)
       |
  HelixDB (graph + vector database)
```

See full architectural diagrams in [`helixir/diagrams/`](helixir/diagrams/).

---

## MCP Tools

### Memory

| Tool | What it does |
|:-----|:-------------|
| `add_memory` | Extract atomic facts from text, deduplicate, store with entities and relations |
| `search_memory` | Semantic search with temporal modes: `recent` (4h), `contextual` (30d), `deep` (90d), `full` |
| `search_by_concept` | Filter by ontology type: skill, preference, goal, fact, opinion, experience, achievement, action |
| `search_reasoning_chain` | Traverse causal/logical connections: IMPLIES, BECAUSE, CONTRADICTS, SUPPORTS |
| `get_memory_graph` | Return memory as a graph of nodes and edges |
| `update_memory` | Modify existing memory content |
| `search_incomplete_thoughts` | Find auto-saved incomplete FastThink sessions |

### FastThink (working memory)

Isolated scratchpad for complex reasoning. Nothing pollutes long-term memory until you explicitly commit.

| Tool | What it does |
|:-----|:-------------|
| `think_start` | Open a new thinking session |
| `think_add` | Add a reasoning step (types: reasoning, hypothesis, observation, question) |
| `think_recall` | Pull facts from long-term memory into the session (read-only) |
| `think_conclude` | Mark a conclusion |
| `think_commit` | Save the conclusion to long-term memory |
| `think_discard` | Discard the session without saving |
| `think_status` | Check session state: thought count, depth, elapsed time |

**Flow:** `think_start` &#8594; `think_add` (repeat) &#8594; `think_recall` (optional) &#8594; `think_conclude` &#8594; `think_commit`

If a session times out, partial thoughts are auto-saved with an `[INCOMPLETE]` tag and recoverable via `search_incomplete_thoughts`.

---

## Integration

### Cursor

Add to `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "helixir": {
      "command": "/path/to/helixir-mcp",
      "env": {
        "HELIX_HOST": "localhost",
        "HELIX_PORT": "6969",
        "HELIX_LLM_PROVIDER": "cerebras",
        "HELIX_LLM_MODEL": "gpt-oss-120b",
        "HELIX_LLM_API_KEY": "YOUR_KEY",
        "HELIX_EMBEDDING_PROVIDER": "openai",
        "HELIX_EMBEDDING_MODEL": "nomic-embed-text-v1.5",
        "HELIX_EMBEDDING_URL": "https://openrouter.ai/api/v1",
        "HELIX_EMBEDDING_API_KEY": "YOUR_KEY"
      }
    }
  }
}
```

### Claude Desktop

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

Same JSON structure as above.

### Cursor Rules (recommended)

Add to **Cursor Settings > Rules** so the agent actually uses its memory:

```
# Core Memory Behavior
- At conversation start, call search_memory to recall relevant context
- After completing tasks, save key outcomes with add_memory
- Use search_by_concept for skill/preference/goal queries
- Use search_reasoning_chain for "why" questions

# FastThink for Complex Reasoning
- Before major decisions, use FastThink to structure your reasoning
- Flow: think_start -> think_add (repeat) -> think_recall -> think_conclude -> think_commit

# What to Save
- ALWAYS save: decisions, outcomes, architecture changes, error fixes, preferences
- NEVER save: grep results, lint output, file contents, temporary data
```

---

## Configuration

All settings are passed as environment variables.

### Required

| Variable | Description |
|:---------|:------------|
| `HELIX_HOST` | HelixDB address (default: `localhost`) |
| `HELIX_PORT` | HelixDB port (default: `6969`) |
| `HELIX_LLM_API_KEY` | API key for the LLM provider |
| `HELIX_EMBEDDING_API_KEY` | API key for the embedding provider |

### Optional

| Variable | Default | Description |
|:---------|:--------|:------------|
| `HELIX_LLM_PROVIDER` | `cerebras` | `cerebras`, `openai`, `ollama` |
| `HELIX_LLM_MODEL` | `gpt-oss-120b` | Model name |
| `HELIX_LLM_BASE_URL` | — | Custom endpoint (for Ollama) |
| `HELIX_EMBEDDING_PROVIDER` | `openai` | `openai`, `ollama` |
| `HELIX_EMBEDDING_URL` | `https://openrouter.ai/api/v1` | Embedding API URL |
| `HELIX_EMBEDDING_MODEL` | `nomic-embed-text-v1.5` | Embedding model |
| `RUST_LOG` | `helixir=warn` | Log level |

### Provider presets

<details>
<summary><b>Cerebras + OpenRouter</b> (recommended — fast inference, cheap embeddings)</summary>

```bash
HELIX_LLM_PROVIDER=cerebras
HELIX_LLM_MODEL=gpt-oss-120b
HELIX_LLM_API_KEY=csk-xxx           # https://cloud.cerebras.ai

HELIX_EMBEDDING_PROVIDER=openai
HELIX_EMBEDDING_URL=https://openrouter.ai/api/v1
HELIX_EMBEDDING_MODEL=nomic-embed-text-v1.5
HELIX_EMBEDDING_API_KEY=sk-or-xxx   # https://openrouter.ai/keys
```

</details>

<details>
<summary><b>Fully local with Ollama</b> (no API keys, fully private)</summary>

```bash
# Install Ollama: https://ollama.com
ollama pull llama3:8b
ollama pull nomic-embed-text

HELIX_LLM_PROVIDER=ollama
HELIX_LLM_MODEL=llama3:8b
HELIX_LLM_BASE_URL=http://localhost:11434

HELIX_EMBEDDING_PROVIDER=ollama
HELIX_EMBEDDING_URL=http://localhost:11434
HELIX_EMBEDDING_MODEL=nomic-embed-text
```

</details>

<details>
<summary><b>OpenAI only</b> (simple, one API key)</summary>

```bash
HELIX_LLM_PROVIDER=openai
HELIX_LLM_MODEL=gpt-4o-mini
HELIX_LLM_API_KEY=sk-xxx

HELIX_EMBEDDING_PROVIDER=openai
HELIX_EMBEDDING_MODEL=text-embedding-3-small
HELIX_EMBEDDING_API_KEY=sk-xxx
```

</details>

---

## Development

```bash
make build          # Build release binary
make test           # Run all tests
make check          # cargo check + clippy
make run            # Run MCP server locally (debug)
make deploy-schema  # Deploy schema to running HelixDB
make docker-up      # Start full stack via Docker Compose
make docker-down    # Stop Docker stack
```

Or directly:

```bash
cargo build --release                          # Build
cargo test                                     # Test (48 tests)
cargo clippy                                   # Lint
RUST_LOG=helixir=debug cargo run --bin helixir-mcp  # Run with debug logs
```

### Project structure

```
helixir-rs/
  helixir/
    src/
      bin/
        helixir_mcp.rs          # MCP server entry point
        helixir_deploy.rs       # Schema deployment CLI
      core/                     # Config, client, search modes
      db/                       # HelixDB client
      llm/                      # LLM providers, extractor, decision engine
      mcp/                      # MCP server, params, prompts
      toolkit/
        tooling_manager/        # Main pipeline (add, search, CRUD)
        mind_toolbox/           # Search engine, entity, ontology, reasoning
        fast_think/             # Working memory (petgraph-based)
    schema/
      schema.hx                 # HelixDB node/edge definitions (33 edge types, 15 node types)
      queries.hx                # HQL queries (100+)
    diagrams/                   # Architecture diagrams (D2 + PNG)
    Dockerfile
    docker-compose.yml
```

---

## License

[MIT](LICENSE) &copy; 2025-2026 Nikita Rulenko

## Links

- [HelixDB](https://github.com/HelixDB/helix-db) — graph + vector database
- [MCP Specification](https://modelcontextprotocol.io/) — Model Context Protocol
- [Cerebras](https://cloud.cerebras.ai) — fast LLM inference (free tier)
- [OpenRouter](https://openrouter.ai) — unified LLM/embedding API
