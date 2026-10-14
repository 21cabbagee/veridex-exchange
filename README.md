# ⟐ VERIDEX — Next-Gen Event Derivatives & Prediction Exchange

> **Institutional-grade binary prediction markets powered by a sub-millisecond Rust matching engine.**
>
> 🌐 **Live Web Terminal (GitHub Pages)**: [https://21cabbagee.github.io/veridex-exchange/](https://21cabbagee.github.io/veridex-exchange/)  
> ⚡ **Rust Matching Engine**: `sub-400μs` FIFO matching latency, synthetic parity, and real-time WebSocket order stream.

---

## 🏛️ Executive Architecture

VERIDEX unbundles the legacy duopoly of prediction markets by pairing an institutional Chicago-style central limit order book (CLOB) with decentralized cryptographic oracle resolution and mathematical arbitrage parity ($P_{\text{Yes}} + P_{\text{No}} \equiv \$1.00$).

```
                      ┌────────────────────────────────────────┐
                      │    VERIDEX Frontend / Web Terminal     │
                      │   (Canvas Probability Curve + CLOB)    │
                      └──────────────────┬─────────────────────┘
                                         │ WebSocket / REST
                                         ▼
                      ┌────────────────────────────────────────┐
                      │        veridex-api (Axum 0.7)          │
                      │  REST Handlers + WS Event Multiplexer  │
                      └──────────────────┬─────────────────────┘
                                         │
                 ┌───────────────────────┴───────────────────────┐
                 ▼                                               ▼
┌─────────────────────────────────┐             ┌─────────────────────────────────┐
│    veridex-orderbook (Rust)     │             │      veridex-engine (Risk)      │
│  • BTreeMap Price-Time Priority │             │  • Margin & Collateral Vault    │
│  • Microsecond FIFO Queue       │             │  • Oracle Resolution Ingestion  │
│  • Sub-penny Tick Precision     │             │  • Mark-to-Market Settlement    │
└─────────────────────────────────┘             └─────────────────────────────────┘
```

---

## ⚡ Core Features

- **Central Limit Order Book (CLOB)**: Price-time priority matching engine implemented from scratch in Rust without bloated third-party dependencies.
- **Zero-Arbitrage Synthetic Parity**: Guaranteed mathematical parity across binary outcome tokens ($P_{\text{Yes}} + P_{\text{No}} = \$1.00$).
- **Sub-Millisecond Execution**: In-memory `BTreeMap` depth indexing designed for HFT liquidity providers.
- **Micro-Canvas Probability Engine**: Dynamic client-side spline curve renderer with custom gradient shading.
- **Multi-Order Support**: Limit, Market, Fill-or-Kill (FOK), and Immediate-or-Cancel (IOC) strategies.
- **Oracle Resolution Pipeline**: Transparent verification with automated contract settlement.
- **Zero Barrier Sandbox**: Instant demo trading sandbox loaded with $10,000 virtual USD collateral.

---

## 📂 Repository Layout

```
.
├── Cargo.toml                  # Cargo workspace manifest
├── crates/
│   ├── core/                   # Domain types, Market, Order, Trade, DomainEvent
│   ├── orderbook/              # Price-time matching engine (BTreeMap, LevelEntry)
│   ├── api/                    # Axum 0.7 HTTP REST + WebSocket server
│   ├── engine/                 # Risk engine, settlement logic, oracle dispatch
│   ├── ws/                     # Standalone WebSocket connection multiplexer
│   └── db/                     # SQLx / Redis connection manager
├── frontend/                   # High-velocity static web exchange terminal
│   ├── index.html              # Main exchange UI (Markets, Depth, Portfolio, Leaderboard)
│   ├── css/app.css             # High-frequency dark mode design system
│   └── js/                     # API client, canvas chart, orderbook ladder, app state
└── .github/workflows/
    └── deploy.yml              # Automated GitHub Pages CI/CD workflow
```

---

## 🚀 Quickstart

### 1. Run the Rust API Server & Matching Engine

```bash
# Ensure Rust stable is installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run the API server on port 8080
cargo run -p predix-api
```

Test the health endpoint:
```bash
curl http://localhost:8080/health
# {"markets":4,"status":"ok","uptime_ms":0,"version":"0.1.0"}
```

Submit an order to the matching engine:
```bash
curl -X POST http://localhost:8080/api/v1/orders \
  -H "Content-Type: application/json" \
  -d '{"market_id":"00000000-0000-0000-0000-000000000001","outcome":"YES","side":"BUY","order_type":"LIMIT","price":0.73,"quantity":"100"}'
```

### 2. Run the Web Terminal

```bash
# Serve the static frontend
python3 -m http.server 5178 --directory frontend
# Open http://localhost:5178 in your browser
```

---

## 🛡️ License

MIT License © 2026 VERIDEX Exchange Authors.
