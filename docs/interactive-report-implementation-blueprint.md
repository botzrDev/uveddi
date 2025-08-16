# Uveddi Interactive Reporting: Local‑First Implementation Blueprint

## Executive Summary

This blueprint combines the provided research with concrete, local‑first implementation guidance to transform Uveddi’s static reports into a modern, interactive experience — without any cloud hosting. The approach leverages a decoupled architecture: a high‑performance Rust backend serving a React + TypeScript Single‑Page Application (SPA), packaged for local use as a Progressive Web App (PWA), an embedded localhost app, or an optional Tauri desktop app. Visualization is polyglot: Mermaid (wrapped), Cytoscape.js for dependency graphs, Chart.js for metrics, and D3 for bespoke visuals.

Key outcomes
- Fully local, offline-capable interactive reports (no cloud required)
- Consistent UX built around the developer loop: Discover → Understand → Act
- Rich, scalable visualizations with strong performance characteristics
- Clean API contract and CI/CD integration to sustain quality and velocity

Non‑goals
- Multi‑tenant SaaS or hosted backend
- Vendor‑specific cloud integrations (can be added later via adapters)

---

## Architecture Overview (Local‑First)

- Decoupled model: Rust backend (Axum/Warp) + React/TypeScript SPA
- SPA assets served from the Rust binary on localhost, or packaged via Tauri
- Offline/PWA support for report viewing without internet access
- API exposes versioned JSON contracts for analysis results

Deployment modes (no cloud):
1) Localhost SPA served by Rust (recommended start)
   - Rust serves /api/v1 and the SPA static bundle on 127.0.0.1
   - Optionally embed assets with `rust-embed` or `include_dir` to ship a single binary
2) Tauri desktop app (optional)
   - Native installer, system webview, Rust backend, fully offline
3) Portable report bundle (optional)
   - Self-contained folder: index.html + assets + data.json + small runner script
   - Double‑click to open locally (no internet)

---

## Frontend Stack (Evidence‑based)

- Framework: React 18+ with TypeScript (ecosystem maturity, visualization wrappers)
- Build tool: Vite (fast dev cycle, small bundles)
- State:
  - Server state: React Query (TanStack Query)
  - Minimal global state: Redux Toolkit or React Context (theme, prefs)
- UI library: MUI (Material UI) or Ant Design (accessibility, velocity)
- Routing: React Router
- Visualization:
  - Network graphs: Cytoscape.js (via react-cytoscapejs)
  - Standard diagrams: Mermaid.js via a custom React wrapper
  - Charts: Chart.js (via react-chartjs-2)
  - Bespoke visuals: D3 (encapsulated in React components)
- PWA: manifest + service worker (Vite PWA plugin)
- Styling: MUI theme + CSS modules or Emotion (or Tailwind if preferred)

Why React + TS now
- Largest ecosystem and visualization wrapper support (Cytoscape, D3, Mermaid wrappers)
- Wide talent pool and abundant examples/documentation
- Aligns with industry leaders (Kibana, Datadog, Grafana teams hire for React/TS)

---

## Visualization Strategy (Polyglot)

- Mermaid.js for standard diagrams (flow, sequence, etc.)
  - Use a React wrapper to manage lifecycle and avoid Virtual DOM conflicts
- Cytoscape.js for dependency/call graphs
  - Efficient layouts, interactions, large graph handling
- Chart.js for metrics cards and distributions
- D3 for signature, bespoke architecture visuals

---

## Local‑Only Delivery Plan

- All assets are bundled; no CDNs; strict Content‑Security‑Policy enforcing `self`
- Service worker caches app shell and report JSON for offline re‑visits
- Optional Tauri packaging for a native, single‑install experience
- CLI ergonomics:
  - `uveddi analyze <path>` → writes report JSON locally (e.g., `~/.uveddi/reports/<id>.json`)
  - `uveddi ui serve` → starts localhost server and opens SPA
  - `uveddi report open <id>` → deep‑links into `/reports/<id>`
  - `uveddi report export <id> --portable` → writes a self‑contained viewer folder

Offline/Air‑gapped mode
- Config flag `offline_mode = true` disables any network egress
- CSP locks down external origins; fonts/images/scripts/styles from self only
- AI integrations default to local providers (e.g., local Ollama) or disabled

---

## Mermaid in Markdown (Render Options)

Goal: Make Markdown reports “show diagrams” in common offline viewers.

Three compatible strategies:
1) Mermaid code fences (zero work)
   - Works out‑of‑the‑box in VS Code preview, GitHub, Obsidian, mdBook (with plugin)
   - Example:
     ```
     ```mermaid
     graph TD
     A --> B
     ```
     ```
2) Pre‑render to inline SVG (universal, offline)
   - Use existing `SvgGenerator` (Mermaid CLI `mmdc`) to render SVG
   - Embed raw `<svg>` in Markdown (Markdown allows inline HTML)
3) Pre‑render to linked files (portable)
   - Save `diagrams/diagram-1.svg` (or `.png`) next to the `.md`
   - Reference with `![title](diagrams/diagram-1.svg)`

Proposed CLI flag
- `--md-diagrams mermaid|inline-svg|linked-svg|linked-png` (default: mermaid)
- If `mmdc` is missing, gracefully fall back to `mermaid` fences

Implementation sketch
- Extend `MarkdownReportGenerator` to accept a `DiagramRenderMode`
- Reuse `src/report/svg_generator.rs` for rendering and fallbacks
- Add `output_dir` param to write linked assets when requested

---

## Backend API (Versioned, Local)

REST endpoints (served by Warp/Axum):
- `GET /api/v1/reports/:id` → Report JSON (versioned contract)
- `GET /api/v1/reports/:id/graphs/dependency` → Graph JSON (nodes/edges)
- `GET /health` and `/metrics` (already present in GraphQL server; unify under one server)
- Static asset serving: `/app/*` serves SPA build; SPA fallback to `index.html`

Report v1 schema (example)
```json
{
  "schemaVersion": "1.0",
  "project": { "id": "uuid", "name": "repo", "commit": "sha", "branch": "main", "repoUrl": "https://…" },
  "summary": { "coverage": 84.2, "issuesTotal": 37, "issuesBySeverity": {"critical": 2, "high": 7, "medium": 15, "low": 13}, "timeGenerated": "2025-08-16T…Z" },
  "findings": [
    { "id": "f-1", "type": "LongMethod", "severity": "high", "title": "Long method in X", "message": "…", "file": "src/x.rs", "startLine": 42, "endLine": 101, "codeSnippet": "…", "tags": ["maintainability"] }
  ],
  "dependencyGraph": { "nodes": [{"id": "n1", "label": "src/x.rs", "path": "src/x.rs", "type": "module"}], "edges": [{"source": "n1", "target": "n2"}] },
  "diagrams": [ { "id": "d1", "kind": "mermaid", "title": "System", "source": "graph TD; A-->B;" } ]
}
```

Notes
- Serialize with Serde; store JSON on disk; keep contract stable and versioned
- Optional: document with OpenAPI in `docs/08-api/openapi.yaml`

---

## SPA UX: Discover → Understand → Act

- Dashboard: KPIs + distribution charts + quick filters
- Findings: searchable/sortable list; severity/category/file filters
- Dependencies: interactive graph (select → detail panel → navigate)
- Diagrams: Mermaid and custom views with export (PNG/SVG)
- Deep‑links: link to SCM file/line from finding detail (constructed from repo metadata)

---

## Implementation Roadmap (Lean, 4–6 weeks)

Phase 0: Groundwork (Week 1)
- Define Report v1 structs + serde
- Implement `GET /api/v1/reports/demo` (mocked JSON)
- Add static serving and SPA fallback to existing server

Phase 1: SPA foundation (Week 1–2)
- Scaffold Vite + React + TS (+ MUI, Router, React Query)
- Fetch and render demo report (dashboard + findings table)
- Add CSP headers; verify no external egress

Phase 2: Visualizations (Week 2–3)
- Mermaid React wrapper (render + rerender + export)
- Cytoscape dependency graph with interactions and layouts
- Chart.js cards for metrics

Phase 3: PWA + Drill‑downs (Week 3–4)
- Add manifest+SW; cache app shell and viewed reports
- Implement drill‑down UX (summary → detail → code link)
- CLI: `uveddi report open <id>`

Phase 4: Exports + Desktop (Week 4–6)
- Markdown diagram render modes (inline/linked) via `SvgGenerator`
- `uveddi report export <id> --portable`
- Optional Tauri desktop packaging

Parallel fix (critical): Tree‑sitter UTF‑8 bounds crash
- Add bounds checks before `node.utf8_text()` and graceful error paths to protect the pipeline

---

## CI/CD & Packaging

- GitHub Actions
  - Frontend build (Node 18, npm ci, vite build), upload `dist/` artifact
  - Backend build/tests; embed SPA assets or package alongside
  - Coverage + security audit jobs remain (existing workflows)
- Docker (optional): multi‑stage build to bundle SPA + Rust server
- Installers: Tauri for native packages (optional deliverable)

---

## Security & Privacy (Local‑Only)

- CSP: `default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self'; font-src 'self' data:`
- No CDNs; bundle all assets; sanitize diagram sources; disable eval
- JWT optional for multi‑user local setups; otherwise localhost only
- Offline mode blocks external HTTP clients by default

---

## Risks & Mitigations

- Tree‑sitter panic on UTF‑8 extraction → Add bounds checks + robust error handling (no panics)
- Large graphs performance → Tune Cytoscape layouts, style throttling, virtualize lists; consider WebGL plugin
- Mermaid rendering inconsistencies → Use React wrapper and/or server‑side SVG pre‑render as fallback
- PWA cache staleness → Version assets and cache keys; add “Refresh data” UX

---

## Acceptance Criteria & Success Metrics

- Analysis runs produce Report v1 JSON and render fully in SPA locally (no internet)
- Mermaid diagrams render reliably in SPA; Markdown diagrams render via chosen mode
- Dependency graphs are interactive and performant for medium projects
- PWA installable; previously viewed reports available offline
- CI passes; no external network egress under offline mode

---

## Immediate Next Steps

1) Define and commit Report v1 structs + demo endpoint (`/api/v1/reports/demo`)
2) Scaffold SPA (Vite + React + TS) and render demo data
3) Implement Mermaid wrapper and Cytoscape dependency view
4) Add `--md-diagrams` modes to Markdown generator using `SvgGenerator`
5) Enforce CSP and offline mode; verify fully local operation

---

## Appendix: Example Endpoint Stubs (Rust)

```rust
// GET /api/v1/reports/:id
async fn get_report(id: String) -> impl warp::Reply {
    // Load from ~/.uveddi/reports/{id}.json
    let report = load_report(id)?; // serde_json::from_reader(...)
    warp::reply::json(&report)
}

// Static SPA serving (dist/ embedded via rust-embed)
let spa = warp::path("app").and(warp::fs::dir("frontend/dist"));
let spa_fallback = warp::get().and(warp::path::end()).map(|| {
    warp::reply::html(include_str!("../static/index.html"))
});
```

---

This document unifies the strategic research with a focused, local‑first execution plan so Uveddi can deliver a best‑in‑class, fully offline interactive reporting experience.
