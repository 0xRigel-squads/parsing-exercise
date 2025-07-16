# Indexer Performance Optimization Exercise

A simplified Solana transaction indexer focused on token program parsing performance optimization.

## Overview

This exercise extracts a critical component from our production indexer - the token program parser. Your goal is to optimize it from its current performance to **achieve at least 10,000 TPS**.

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Mainnet TX    │───▶│  Token Program  │───▶│   Queue Entry   │
│   Data Feed     │    │     Parser      │    │  (Filtered)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                               │
                               ▼
                    ┌─────────────────┐
                    │ Redis Cache     │
                    │ (Owner Lookup)  │
                    └─────────────────┘
```

## Architecture

### Core Components

1. **Token Program Parser** (`core/src/parsing/token_program/parser.rs`)

   - Processes token program instructions from Solana transactions
   - Creates `TokenAccountChange` objects for each registered token program instruction
   - Filters changes by relevance using batch Redis calls to avoid multiple round-trip latency

2. **Redis Emulator** (`core/src/redis/local_emulator.rs`)

   - Simulates production Redis with 125k token owners + 25k vault addresses
   - Adds realistic network latency (0.5ms per batch call)
   - Supports batch operations for performance testing

3. **Queue Entry System** (`core/src/queue_entry.rs`)
   - Aggregates token account changes from parsing
   - Only relevant changes (based on owner cache) are queued for downstream processing

### Data Flow

```
Transaction → Parse Instructions → Create All Changes → Check Owner Relevance → Queue Entry
     │              │                      │                    │             │
     │              │                      │                    ▼             │
     │              │                      │          ┌─────────────────┐     │
     │              │                      │          │ Redis Cache     │     │
     │              │                      │          │ Batch Lookup    │     │
     │              │                      │          └─────────────────┘     │
     │              │                      │                                  │
     ▼              ▼                      ▼                                  ▼
  Raw Solana    Token Program          Token Changes                    Filtered
 Transaction    Instructions           (All Created)                    Results
```

## Performance Bottlenecks

The current implementation shows performance limitations that need to be addressed to reach the target TPS.

### Processing Requirements

- **Sequential Processing**: Transactions must be processed in order since transaction A may introduce new relevant owners that are referenced in transaction B. You cannot batch multiple transactions together.
- **Redis Latency Bottleneck**: Each Redis operation adds 0.5ms latency. With sequential processing, this creates a theoretical maximum of ~2,000 TPS. Consider how to work within or around this constraint.
- **Cache Architecture**: Redis serves as the central source of truth, but consider when and how the cache is accessed.

## Getting Started

### Prerequisites

- Rust 1.70+
- Cargo

### Running the Benchmark

```bash
# Run with default settings (processes all transactions)
cargo run

```

### Current Performance

Initial runs typically show ~800-900 TPS. Your target: **10,000+ TPS**.

**Performance Milestones:**
- **Level 1**: ~10,000 TPS - Address the primary bottleneck
- **Level 2**: ~50,000+ TPS - Identify and optimize secondary performance issues

**Hint**: The Redis latency constraint suggests the bottleneck isn't CPU-bound. What does this tell you about where optimizations should focus?

## Production Context

In production, our indexer:

- Runs 5 parsers in sequence on each txn (this exercise focuses on 1)
- Uses Redis as a central source of truth for account owner cache and task queue
- Maintains data consistency across multiple processing streams

## Exercise Rules

### What You Can Modify

- **Token Program Parser** (`core/src/parsing/token_program/parser.rs`) - Primary optimization target & any of its associated data structures
- **Redis Emulator** (`core/src/redis/local_emulator.rs`) - Add new functions if needed
- **Parser initialization and caching strategy** - Consider when and how the Redis cache is accessed
- **Data structures** - Optimize allocations, string handling, and data flow

### What You Must NOT Modify

- **Main benchmark harness** (`src/main.rs`) - Keep benchmark logic intact
- **Transaction data structures** (`core/src/transaction/`) - Core transaction types
- **Queue entry core logic** (`core/src/queue_entry.rs`) - Preserve existing functionality
- **Parser trait interfaces** (`core/src/parsing/parser_trait.rs`) - Keep API contracts

## Success Criteria

1. **Performance**: Achieve 10,000+ TPS consistently
2. **Correctness**: Same filtering and parsing behavior as baseline
3. **Code Quality**: Clean, maintainable optimizations
4. **Strategic Thinking**: Demonstrate understanding of system bottlenecks and optimization trade-offs

**Evaluation focuses on:**
- Identifying the root cause of performance bottlenecks
- Systems-level thinking about cache architecture
- Understanding the relationship between network latency and throughput
- Clean implementation of performance optimizations

## Key Files to Examine

- `core/src/parsing/token_program/parser.rs` - **Primary optimization target**
- `core/src/redis/local_emulator.rs` - Redis simulation (can be extended)
- `src/main.rs` - Benchmark harness (understand but don't modify)
