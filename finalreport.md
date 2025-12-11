# Personal Finance Tracker - Final Report

## ECE1724 Fall 2025 - Rust Programming Project

---

## Team Members

| Name | Student Number | Email |
|------|----------------|-------|
| Muchen Liu | 1006732145 | muchen.liu@mail.utoronto.ca |
| Ping Shu | 1010506365 | pings.shu@mail.utoronto.ca |
| Ziang Wang | 1006912370 | ziang.wang@mail.utoronto.ca |

---

## 1. Motivation

### Why Personal Finance Management?

Managing daily finances is a universal challenge faced by students, professionals, and families alike. While numerous financial management tools exist, most are limited by:
- **Paywalls** that restrict access to essential features
- **Mobile-only platforms** that don't integrate with terminal workflows
- **Feature bloat** that makes simple tasks unnecessarily complex
- **Privacy concerns** with cloud-based services handling sensitive financial data

### The Rust Ecosystem Gap

Our research revealed a significant gap in the Rust ecosystem: while Rust excels in systems programming and web services, there is a notable absence of comprehensive personal finance tools. Most available finance utilities are written in Python, Java, or JavaScript. This presented an opportunity to:

1. **Fill a community need** by creating a terminal-based finance tracker
2. **Demonstrate Rust's capabilities** in practical, user-facing applications
3. **Gain experience** with Rust's database, async, and TUI ecosystems

### Personal Motivation

Our team was motivated by real-world problems we face daily:
- Logging recurring expenses (rent, subscriptions, bills)
- Understanding spending patterns across categories
- Managing accounts across multiple banks and currencies
- Converting currencies when traveling internationally

Building a tool that solves our own problems while contributing to the Rust ecosystem made this project both personally satisfying and technically rewarding.

---

## 2. Objectives

The primary objective of this project was to develop a **complete, user-friendly, terminal-based personal finance tracker** that leverages Rust's memory safety, strict type system, and efficient concurrency.

### Specific Goals

1. **Transaction Management**: Enable users to log income and expenses with categorization
2. **Multi-Account Support**: Manage multiple bank accounts, credit cards, and savings accounts
3. **Multi-Currency Support**: Handle transactions in different currencies with real-time exchange rates
4. **Recurring Transactions**: Automate regular expenses like rent, salary, and subscriptions
5. **Financial Insights**: Provide spending analysis and category breakdowns
6. **Data Export**: Allow CSV/JSON export for tax preparation and external analysis
7. **Text-Based Interface**: Build an intuitive TUI using ratatui for terminal users
8. **RESTful API**: Provide a complete API for programmatic access

---

## 3. Features

### 3.1 Core Features

| Feature | Description |
|---------|-------------|
| **User Management** | Create, switch between, and delete user profiles |
| **Account Management** | Add/delete bank accounts with custom currencies (USD, EUR, CAD, etc.) |
| **Transaction Logging** | Record income and expenses with descriptions and categories |
| **Category System** | User-defined categories for organizing transactions |
| **Balance Tracking** | Automatic balance updates on transaction create/delete |

### 3.2 Advanced Features

| Feature | Description |
|---------|-------------|
| **Recurring Transactions** | Set up automatic monthly/weekly/daily/yearly transactions |
| **Multi-Currency** | Support for 100+ currencies with real-time exchange rates |
| **Currency Conversion** | Convert amounts between currencies using live rates |
| **Exchange Rate Scraping** | Fetch latest rates from X-Rates website |
| **Spending Analytics** | Top spending categories with visual bar charts |
| **Data Export** | Export to CSV and JSON formats |

### 3.3 User Interface Features

| Feature | Description |
|---------|-------------|
| **8-Tab Navigation** | Dashboard, Accounts, Transactions, Categories, Recurring, FX Rates, Reports, Export |
| **Keyboard Shortcuts** | Efficient navigation with single-key commands |
| **Scrollable Lists** | Full scrolling with Page Up/Down, Home/End support |
| **Interactive Forms** | Tab-based field navigation for data entry |
| **Real-time Filtering** | Filter transactions by currency |
| **Position Indicators** | Always know your position in lists (e.g., [5/100]) |

---

## 4. User's Guide

### 4.1 Starting the Application

**Launch the TUI:**
```bash
cargo run tui
```

**Start the API server:**
```bash
cargo run serve
```

### 4.2 User Selection Screen

When you launch the TUI, you'll see the user selection screen:

| Key | Action |
|-----|--------|
| `↑/↓` | Navigate between users |
| `Enter` | Login as selected user |
| `a` | Add new user |
| `d` | Delete selected user |
| `q` | Quit application |

### 4.3 Main Navigation (After Login)

Use number keys or arrow keys to switch between tabs:

| Tab | Key | Screen |
|-----|-----|--------|
| 1 | `1` or `←/→` | Dashboard |
| 2 | `2` | Accounts |
| 3 | `3` | Transactions |
| 4 | `4` | Categories |
| 5 | `5` | Recurring |
| 6 | `6` | FX Rates |
| 7 | `7` | Reports |
| 8 | `8` | Export |

### 4.4 Common Actions

| Action | Key | Available On |
|--------|-----|--------------|
| Add item | `a` | Accounts, Transactions, Categories, Recurring, FX Rates |
| Delete item | `d` | Accounts, Transactions, Categories, Recurring, FX Rates |
| View details | `Enter` | Accounts, Transactions, Recurring, FX Rates |
| Refresh data | `r` | All screens |
| Switch user | `u` | All screens |
| Quit | `q` | All screens |
| Cancel/Back | `Esc` | All forms and dialogs |

### 4.5 Adding a Transaction

1. Navigate to **Transactions** (Tab 3)
2. Press `a` to open the add form
3. Fill in fields using `Tab` to move between them:
   - **Account ID**: Select from the list on the right (grouped by currency)
   - **Amount**: Enter the transaction amount
   - **Type**: Enter `i` for income or `e` for expense
   - **Description**: Brief description of the transaction
   - **Category ID**: Select a category
4. Press `Enter` to submit

### 4.6 Managing Recurring Transactions

1. Navigate to **Recurring** (Tab 5)
2. Available actions:
   - `a` - Add new recurring transaction
   - `p` - Process all due recurring transactions (creates actual transactions)
   - `t` - Toggle active/paused status
   - `d` - Delete recurring transaction

### 4.7 Currency Conversion

1. Navigate to **FX Rates** (Tab 6)
2. Press `c` to open conversion dialog
3. Enter:
   - From currency (e.g., `USD`)
   - To currency (e.g., `EUR`)
   - Amount to convert
4. Press `Enter` to see the converted amount

### 4.8 Exporting Data

1. Navigate to **Export** (Tab 8)
2. Press `e` to open export dialog
3. Select format:
   - `1` - Transactions CSV
   - `2` - Transactions JSON
   - `3` - Accounts CSV
   - `4` - Full Summary JSON

---

## 5. Reproducibility Guide

### 5.1 Prerequisites

Ensure you have the following installed:
- **Rust** (1.70 or later) - https://rustup.rs/
- **SQLite3** - `apt install sqlite3` (Ubuntu) or `brew install sqlite3` (macOS)
- **Git** - `apt install git` (Ubuntu) or `brew install git` (macOS)

### 5.2 Clone the Repository

```bash
git clone https://github.com/pingshu-liu/ECE1724RustProject_PersonalFinanceTracker.git
cd ECE1724RustProject_PersonalFinanceTracker
```

### 5.3 Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Edit .env if needed (default values work for local development)
# DATABASE_URL=sqlite:./personal-finance-tracker.db
# BIND_ADDRESS=127.0.0.1:8080
```

### 5.4 Database Setup

```bash
# Install SQLx CLI (for database migrations)
cargo install sqlx-cli --features sqlite

# Create the database file
sqlx database create

# Run migrations to create all tables
sqlx migrate run

# Verify tables were created (optional)
sqlite3 personal-finance-tracker.db ".tables"
# Expected: accounts categories exchange_rates recurring_transactions 
#           transaction_categories transactions users
```

### 5.5 Seed Initial Data

```bash
# Populate database with sample data (users, accounts, transactions)
cargo run db_seed

# Fetch latest exchange rates
cargo run scrape_rates
```

### 5.6 Build and Run

**Option 1: Run the TUI (Text User Interface)**
```bash
cargo run tui
```

**Option 2: Run the REST API Server**
```bash
cargo run serve
# Server starts at http://127.0.0.1:8080
```

**Option 3: Run both (in separate terminals)**
```bash
# Terminal 1: Start API server
cargo run serve

# Terminal 2: Start TUI
cargo run tui
```

### 5.7 Verify Installation

After running `cargo run tui`:
1. You should see a user selection screen with 3 sample users
2. Select a user and press Enter
3. Navigate through all 8 tabs to verify functionality
4. Try adding a transaction (Tab 3 → press `a`)

### 5.8 Other Useful Commands

```bash
# Clear all data from database
cargo run db_clear

# Clear and reseed database
cargo run db_reseed

# Scrape exchange rates for specific currency
cargo run scrape_rates CAD

# Show help
cargo run help

# Check database status
cargo run db_status
```

---

## 6. Contributions by Team Member

### Ping Shu

**Primary Responsibilities:**
- Project initialization and Rust project setup
- Database schema design and SQLite integration
- SQLx migrations for all core tables
- RESTful API development using Actix-web:
  - User CRUD endpoints
  - Account management endpoints
  - Transaction endpoints
  - Category endpoints
- Exchange rate API integration and web scraping
- Currency conversion logic
- TUI framework setup with ratatui
- Main navigation and menu system
- User documentation and setup guide
- API documentation

### Muchen Liu

**Primary Responsibilities:**
- Migration file creation using sqlx-cli
- Seed data creation for testing
- Error handling and validation implementation
- API testing with Postman
- Recurring transaction endpoints
- CSV/JSON export functionality
- TUI screens implementation:
  - Transaction entry and viewing
  - Account management screens
  - Reports and spending analysis
  - Settings and configuration
- Developer documentation
- Video presentation and demo creation

### Ziang Wang

**Primary Responsibilities:**
- Database structure documentation
- Automated migration testing
- API authentication and security
- Data integrity and error handling validation
- API endpoint documentation
- Currency conversion testing
- Recurring transaction automation testing
- Export format testing
- TUI to backend API connection
- Final project submission preparation

---

## 7. Lessons Learned and Concluding Remarks

### 7.1 Technical Lessons

**Rust Ownership and Borrowing:**
Working with ratatui and async database operations taught us to carefully manage ownership. For example, when rendering lists, we learned to use references and cloning strategically to avoid borrow checker conflicts while maintaining performance.

**Async Programming:**
SQLx's async nature required understanding of Tokio runtime and proper async/await patterns. We learned to structure our code to maximize concurrency while handling database connections efficiently.

**Error Handling:**
Rust's Result type forced us to handle errors explicitly. This led to more robust code where database failures, invalid user input, and network errors are all handled gracefully with informative messages.

**TUI Development:**
Building with ratatui taught us about:
- Stateful vs stateless widget rendering
- Efficient terminal redrawing
- Keyboard event handling
- Layout systems and responsive design in terminals

### 7.2 Project Management Lessons

**Iterative Development:**
Our tentative plan evolved significantly as we progressed. Features we thought would be simple (like multi-currency support) required more work, while others (like the TUI navigation) came together faster than expected.

**Testing Throughout:**
We learned the value of testing each component as it was built rather than waiting until the end. This caught issues early and made debugging much easier.

**Documentation During Development:**
Writing documentation alongside code development (rather than at the end) helped clarify our thinking and made the codebase more maintainable.

### 7.3 Rust Ecosystem Observations

**Strengths:**
- SQLx provided excellent type-safe database queries
- ratatui is a mature and well-documented TUI framework
- Actix-web made RESTful API development straightforward
- Cargo's dependency management is excellent

**Challenges:**
- Some libraries have steep learning curves
- Compile times can be long with many dependencies
- The Rust community's finance tools are less mature than Python/JavaScript

### 7.4 Concluding Remarks

This project successfully demonstrates that Rust is a viable choice for building practical, user-facing financial applications. Our Personal Finance Tracker provides:

- **Complete functionality** for personal finance management
- **Multiple interfaces** (TUI and REST API) for different use cases
- **Multi-currency support** for global users
- **Data portability** through CSV/JSON exports

The project fills a gap in the Rust ecosystem by providing a comprehensive, terminal-based finance tracker that prioritizes:
- Privacy (local SQLite database)
- Speed (Rust's performance)
- Reliability (type safety and error handling)
- Usability (intuitive TUI with keyboard shortcuts)

We hope this project inspires other Rust developers to build practical applications in domains traditionally dominated by Python and JavaScript.

---

## Appendix: Project Statistics

| Metric | Value |
|--------|-------|
| Lines of Rust Code | ~4,500 |
| Number of Source Files | 6 |
| Database Tables | 7 |
| API Endpoints | 25+ |
| TUI Screens | 8 |
| Supported Currencies | 100+ |

---

*Final Report - ECE1724 Fall 2025*
*Personal Finance Tracker Team*

