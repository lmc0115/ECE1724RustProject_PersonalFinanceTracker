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
| **Multi-Currency** | Support for 50+ currencies with real-time exchange rates |
| **Currency Conversion Calculator** | Convert amounts between currencies using live rates |
| **View in Currency (Transactions)** | View all transactions converted to any target currency |
| **View in Currency (Account)** | View account details with all amounts in chosen currency |
| **Intelligent Rate Fallback** | Direct rate → Inverse rate → Multi-currency triangulation (USD/EUR/CAD/GBP) |
| **Exchange Rate Scraping** | Fetch latest rates from X-Rates (4 base currencies: USD, EUR, CAD, GBP × 50+ targets) |
| **Spending Analytics** | Top spending categories with visual bar charts |
| **Data Export** | Export to CSV and JSON formats |
| **Account Transaction View** | View all transactions under each account with full details |

### 3.3 User Interface Features

| Feature | Description |
|---------|-------------|
| **8-Tab Navigation** | Dashboard, Accounts, Transactions, Categories, Recurring, FX Rates, Reports, Export |
| **Cross-Platform Shortcuts** | Works on Windows, Mac, and Linux with alternative key bindings |
| **Scrollable Lists** | Full scrolling with `↑`/`↓`, `[`/`]` (jump 10) |
| **Scrollable Dialogs** | Currency selection shows all 50+ currencies with scroll support |
| **Interactive Forms** | Tab-based field navigation for data entry |
| **Currency Filtering** | Filter transactions by specific currency |
| **View in Currency** | Convert and view all amounts in any target currency (Enter to select) |
| **Position Indicators** | Always know your position in lists (e.g., [22-32/56]) |
| **Account Details** | View all transactions under an account with independent currency conversion |
| **Visual Highlights** | `►` marker and `✓ ACTIVE` indicators in selection dialogs |

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
| Filter by currency | `f` | Transactions |
| View in currency | `v` | Transactions, Account Details |
| Refresh data | `r` | All screens |
| Switch user | `u` | All screens |
| Quit | `q` | All screens |
| Cancel/Back | `Esc` | All forms and dialogs |

### 4.5 List Navigation (Cross-Platform)

| Action | Windows/Linux | Mac Alternative |
|--------|--------------|-----------------|
| Move up | `↑` | `↑` |
| Move down | `↓` | `↓` |
| Jump up 10 items | `Page Up` | `[` |
| Jump down 10 items | `Page Down` | `]` |
| Jump to first item | `Home` | `g` |
| Jump to last item | `End` | `G` (Shift+g) |

### 4.6 Adding a Transaction

1. Navigate to **Transactions** (Tab 3)
2. Press `a` to open the add form
3. Fill in fields using `Tab` to move between them:
   - **Account ID**: Select from the list on the right (shows account name and currency)
   - **Amount**: Enter the transaction amount
   - **Type**: Enter `i` for income or `e` for expense
   - **Description**: Brief description of the transaction
   - **Category ID**: Select a category
4. Press `Enter` to submit

### 4.7 Managing Recurring Transactions

1. Navigate to **Recurring** (Tab 5)
2. Available actions:
   - `a` - Add new recurring transaction
   - `p` - Process all due recurring transactions (creates actual transactions)
   - `t` - Toggle active/paused status
   - `d` - Delete recurring transaction

### 4.8 View in Currency (Currency Conversion for Display)

This feature allows you to view all transaction amounts converted to a single currency. The currency selection dialog is **fully scrollable** and shows all 50+ currencies from the exchange rates database.

**On Transactions Screen:**
1. Navigate to **Transactions** (Tab 3)
2. Press `v` to open the View in Currency dialog
3. Use `↑`/`↓` to scroll through currencies, `[`/`]` to jump 10
4. Press `Enter` to select the highlighted currency
5. All transactions will display amounts in the selected currency
6. Original amounts are shown in parentheses

**On Account Details:**
1. Navigate to **Accounts** (Tab 2)
2. Press `Enter` on an account to view details
3. You'll see all transactions for that account listed
4. Press `v` to open the currency selection dialog
5. Scroll with `↑`/`↓` and press `Enter` to select
6. Both balance and transaction amounts will be converted
7. Press `Esc` to exit details (currency resets automatically)

**Note:** Transactions screen and Account Details have **separate currency states** - changing one does not affect the other.

**Currency Selection Dialog Controls:**
| Action | Key |
|--------|-----|
| Scroll up | `↑` |
| Scroll down | `↓` |
| Jump up 10 | `[` |
| Jump down 10 | `]` |
| Select currency | `Enter` |
| Cancel | `Esc` |

The system uses exchange rates from the database, with intelligent fallback:
- Direct rate if available (e.g., USD → CZK)
- Inverse rate (1/rate) if reverse exists (e.g., CZK → USD inverted)
- Multi-currency triangulation via USD, EUR, CAD, or GBP as intermediates

**Technical Details:**
- Exchange rates are loaded with deduplication (only latest rate per currency pair)
- Currency codes are extracted from full names (e.g., "Czech Koruna (CZK)" → "CZK")
- This ensures "USD" matches "US Dollar (USD)" for reliable rate lookup

### 4.9 Currency Conversion Calculator

1. Navigate to **FX Rates** (Tab 6)
2. Press `c` to open conversion dialog
3. Enter:
   - From currency (e.g., `USD`)
   - To currency (e.g., `EUR`)
   - Amount to convert
4. Press `Enter` to see the converted amount

### 4.10 Exporting Data

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

# Fetch latest exchange rates (scrapes USD, EUR, CAD, GBP as base currencies)
# This creates ~200 exchange rate pairs for currency conversion
cargo run scrape_rates

# Optional: Scrape additional base currency (e.g., JPY)
cargo run scrape_rates JPY
```

**Note:** Exchange rates are required for the "View in Currency" feature to work correctly. Running `scrape_rates` fetches rates for 4 base currencies, enabling conversion between any of the 50+ supported currencies through direct rates or triangulation.

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
- Stateful vs stateless widget rendering (ListState for scrollable lists)
- Efficient terminal redrawing and layout management
- Keyboard event handling across different platforms (Mac lacks Home/End/PageUp/PageDown keys)
- Providing alternative keybindings (e.g., `[`/`]` for PageUp/PageDown)
- Building scrollable dialogs with Enter-to-select pattern for better UX
- Managing separate state for different contexts (e.g., account view currency vs transaction view currency)

**Multi-Currency Handling:**
Implementing currency conversion across 50+ currencies taught us:
- The importance of normalizing data formats (currency codes vs full names like "Czech Koruna (CZK)")
- How to extract currency codes from various string formats using pattern matching
- Triangulation strategies when direct exchange rates don't exist
- Smart database queries that deduplicate and get only the latest rates per currency pair
- The value of trying multiple intermediate currencies (USD, EUR, CAD, GBP) for robust conversion

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
- **Multi-currency support** for global users with real-time conversion
- **View in Currency** feature for unified financial overview across currencies
- **Cross-platform compatibility** with Mac, Linux, and Windows support
- **Data portability** through CSV/JSON exports

The project fills a gap in the Rust ecosystem by providing a comprehensive, terminal-based finance tracker that prioritizes:
- Privacy (local SQLite database)
- Speed (Rust's performance)
- Reliability (type safety and error handling)
- Usability (intuitive TUI with scrollable dialogs and Enter-to-select pattern)
- Accessibility (cross-platform keyboard shortcuts for Mac/Linux/Windows)

We hope this project inspires other Rust developers to build practical applications in domains traditionally dominated by Python and JavaScript.

---

## Appendix: Project Statistics

| Metric | Value |
|--------|-------|
| Lines of Rust Code | ~5,500+ |
| Number of Source Files | 6 |
| Database Tables | 7 |
| API Endpoints | 25+ |
| TUI Screens | 8 |
| TUI Modes | 14 (Normal, AddTransaction, ViewDetails, SelectViewCurrency, etc.) |
| Supported Currencies | 50+ (from FX rates database) |
| Exchange Rate Pairs | ~200 (4 base currencies × 50+ targets, deduplicated) |
| Triangulation Intermediates | 4 (USD, EUR, CAD, GBP) |
| Keyboard Shortcuts | 35+ (cross-platform compatible) |
| Scrollable Dialogs | 2 (Currency Filter, View in Currency) |

### Exchange Rate System Architecture

The exchange rate system is designed for reliability and completeness:

1. **Data Collection**: Scrapes X-Rates.com for 4 base currencies (USD, EUR, CAD, GBP), each providing rates to ~50 target currencies
2. **Smart Loading**: Uses SQL query with GROUP BY to load only the latest rate per currency pair, avoiding duplicates from multiple scrapes
3. **Currency Normalization**: Extracts 3-letter codes from full names (e.g., "Czech Koruna (CZK)" → "CZK") for reliable matching
4. **Flexible Matching**: Compares currencies by code, not string, so "USD" matches "US Dollar (USD)"
5. **Multi-Level Fallback**:
   - Direct rate lookup
   - Inverse rate calculation (1/rate)
   - Triangulation through USD, EUR, CAD, or GBP as intermediates

---

*Final Report - ECE1724 Fall 2025*
*Personal Finance Tracker Team*

