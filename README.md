# Personal Finance Tracker - Project Proposal


### **Team Menbers**  
| Name | Student ID | Email |   
|------|------------|-------|  
| Muchen Liu | 1006732145 | muchen.liu@mail.utoronto.ca |
| Ping Shu | 1010506365 | pings.shu@mail.utoronta.ca | 
| Ziang Wang | 1006912370 | ziang.wang@mail.utoronto.ca |


## **1. Motivation**

Our team chose to develop a Personal Finance Tracker because managing daily finances is a relatable challenge for many people. Students, professionals, and families all need to track income, expenses, and savings, but existing tools are mostly limited by paywalls, dependent on mobile apps, or overloaded with features that make them complex to use. Our team wants to create a utility that is simple, accessible, and efficient, which is a lightweight command-line finance tracker that anyone can use directly from their terminal.

Our team found this project idea motivating because it combines both personal need for practical usefulness and the opportunity to expand the Rust ecosystem for technical creativity. What excites us most is that this project is practical and fun. It solves real-world problems we face, like logging recurring expenses, understanding where our money goes each month, converting between currencies when travelling around different countries, and managing accounts across different banks. It is also a challenge for us to explore advanced Rust libraries for databases, APIs, and text-based interfaces. This will make the development process rewarding and educational.

From a community perspective, there is a considerable gap in the Rust ecosystem that, although Rust has excellent support for systems programming and web services, there is a lack of finance-focused command-line tools. Most available utilities are written in Python, Java, or C++. By implementing this project in Rust, our team will gain valuable experience with the Rust ecosystem. Nevertheless, this project will contribute a utility that may inspire future Rust developers to explore beyond traditional systems programming.

Generally, the motivation behind our project comes from three main drivers:
- **Practical need** — addressing a real-world problem in personal finance management.
- **Personal satisfaction** — working on a project that is both fun and directly applicable to people’s lives satisfied our team a lot.
- **Community contribution** — filling a gap in Rust’s ecosystem that potentially inspires future applications. 



## **2. Project Objectives and Key Features**

### **2.1 Objectives**
The Object of the Personal Tracker is to use **Rust’s guarantees of memory safety**, **strict type system**, and **efficient concurrency** to perform a **Complete, user-friendly, terminal-based financial utility**.  

Our goal is to develop a **practical and user-friendly command-line personal finance tracker** that enables users to manage their daily finances across multiple currencies and bank accounts. The tool enables users to log transactions, categorize them, and view their spending patterns over time.  

By using a backend server for data storage and integrating real-time exchange rate APIs, we aim to create a system that's both accessible and secure, while providing accurate financial insights regardless of which currencies users work with.  

In addition to being a practical tool for everyday use, we aim to explore and demonstrate how **Rust** can be applied to real-world financial records, competing with **JavaScript** and **Python** for mature personal finance ecosystems.


### **2.2 Key Features**

To realize this objective, the following components will be developed:

---

#### **Ping Shu is responsible for:**

#### **2.2.1 Transaction Logging and Multi-Account Support**

Our personal finance tracker allows users to register both expenses and income entries, assigning them to user-defined categories. Supporting complex entries where a single transaction is distributed into multiple categories (e.g., groceries split into food, cleaning supplies, and entertainment).  

The utility manages different account types, such as savings, checking, and credit cards. We aim to model realistic financial scenarios where one personal account may encompass several cards or sub-accounts across banks.

---

#### **2.2.2 Multi-Currency Support with Real-Time Exchange Rates**

Many people deal with multiple currencies, whether from travel, daily online purchases. Our personal finance tracker supports multiple bank accounts in different currencies and can fetch real-time exchange rates via API or customize the exchange rate manually.  

This means the currency conversion functionality automatically retrieves current exchange rates to provide accurate conversions. Users can also view their complete financial picture across all accounts without calculating conversions between currencies manually.  

Users could manage different account types, such as savings, checking, and credit cards, in our all-in-one utility, where one personal account may encompass several cards or sub-accounts across banks.

---

#### **Ziang Wang is responsible for:**

#### **2.2.3 Recurring Transactions**

Expenses and income sometimes follow predictable patterns. Users can set up recurring expenses such as rent, salary, or subscriptions. When creating a recurring transaction, users specify the amount, category, frequency (monthly most of the time), and start date.  

Our finance tracker will also automatically create these transactions at the appropriate intervals.

**This feature provides two practical benefits:**

- **Reduced manual entry:** Once set up, recurring transactions are logged automatically, saving time and reducing the chance of forgetting to record regular expenses or income.  
- **Better financial planning:** Users can view their scheduled recurring transactions to understand expected cash flow. This helps with anticipating upcoming expenses and planning spending accordingly.  

Users maintain full control over recurring transactions—they can modify amounts, pause, or cancel them as needed when circumstances change. This flexibility ensures the tool adapts to real-life situations like salary increases, subscription cancellations, or rent adjustments.

---

#### **2.2.4 Expense Analysis and Insights**

Our finance tracker will provide simple, text-based summaries and charts to help users understand their spending patterns.  

Users can query their transaction data to generate various reports and insights:

- **Spending comparisons:** See how spending in specific categories has changed over time (e.g., "This month you spent 35% more on groceries compared to last month")  
- **Category breakdowns:** View top spending categories for a given period (e.g., "Your top three spending categories are: Food, Transportation, Entertainment")  
- **Custom queries:** Users can filter transactions by date range, category, account, or amount to analyze specific aspects of their finances  

These insights are presented in a straightforward, text-based format that focuses on clarity. The goal is to surface useful information that helps users identify trends and make informed decisions about their spending habits.

---

#### **Muchen Liu is responsible for:**

#### **2.2.5 Data Export (CSV/JSON)**

Users can export their transaction data in standard CSV or JSON formats, which supports other use cases:

- **Tax preparation:** Export annual transaction data to share with accountants or import into tax software  
- **External analysis:** Open exported CSV files in spreadsheet applications like Excel or Google Sheets for custom charts, pivot tables, or detailed analysis  
- **Data portability:** Keep backup copies of financial records or migrate data to other financial management tools  

The export function allows users to specify date ranges and select which accounts or categories to include, so they can export exactly the data they need rather than everything at once. Both CSV and JSON formats are supported here.

---

#### **2.2.6 Text-Based Interface**

Build an intuitive command-line interface using **ratatui**. Include tables, graphs, and interactive forms, making the experience accessible to users who prefer terminal workflows.

---

### **Alignment with Rust Ecosystem Gap**

This project directly addresses the lack of a comprehensive personal finance tracker in the Rust ecosystem.  

By incorporating reliable storage, expressive TUI components, and back-end integration with frameworks such as **Axum** or **Actix Web** and **Diesel/SQLx**, this work demonstrates how Rust can support practical domain-specific applications.

---

## **3. Tentative Plan**



### **3.1 Project Setup & Database Design (Week 1: Oct 6 - Oct 12)**

- Initialize Rust project with necessary dependencies (**axum**, **sqlx**, **tokio**, etc.) -- By Ping Shu
- Set up **SQLite** database -- By Ping Shu
- Design database schema for accounts, transactions, categories, and recurring transactions -- By Ping Shu
- Create initial migration files using **sqlx-cli** -- By Muchen Liu
- Document the database structure and relationships -- By Ziang Wang 

---

### **3.2 Database Implementation & Migration (Week 2: Oct 13 - Oct 19)**

- Implement database migrations for all core tables -- By Ping Shu
- Set up connection pooling and database configuration -- By Ping Shu
- Create seed data for testing (sample accounts, categories, transactions) -- By Muchen Liu  
- Verify database operations and constraints -- By Ziang Wang 
- Set up automated migration testing -- By Ziang Wang  

---

### **3.3 Backend API Development - Core Features (Week 3-5: Oct 20 - Nov 9)**

- Implement RESTful API endpoints for CRUD operations: -- By Ping Shu
  - Account management (create, read, update, delete accounts)  
  - Transaction logging (add, edit, delete transactions)  
  - Category management  
  - Basic querying and filtering  
- Set up proper error handling and validation -- By Muchen Liu
- Implement API authentication and security -- By Ziang Wang  

---

### **3.4 API Testing & Validation (Week 6: Nov 10 - Nov 16)**

- Write unit tests for database operations -- By Ping Shu
- Create integration tests for API endpoints -- By Ping Shu
- Test with tools like Postman -- By Muchen Liu
- Validate data integrity and error handling -- By Ziang Wang
- Document API endpoints and usage -- By Ziang Wang  

---

### **3.5 Advanced Features - Multi-Currency & Recurring Transactions (Week 7: Nov 17 - Nov 23)**

- Integrate exchange rate API (e.g., exchangerate-api.io or similar)  -- By Ping Shu 
- Implement currency conversion logic -- By Ping Shu
- Create endpoints for managing recurring transactions -- By Muchen Liu
- Test currency conversion accuracy and recurring transaction automation -- By Ziang Wang  

---

### **3.6 Data Analysis & Export Features (Week 8: Nov 24 - Nov 30)**

- Implement spending analysis algorithms -- By Ping Shu
- Create endpoints for generating reports and insights -- By Ping Shu  
- Build CSV/JSON export functionality -- By Muchen Liu
- Add query filters for custom date ranges and categories -- By Muchen Liu 
- Test export formats and analysis accuracy -- By Ziang Wang  

---

### **3.7 TUI Development with Ratatui (Week 9: Dec 1 - Dec 7)**

- Set up **ratatui** framework and basic UI structure -- By Ping Shu 
- Implement main navigation and menu system -- By Ping Shu
- Create screens for: -- By Muchen Liu
  - Transaction entry and viewing  
  - Account management  
  - Spending analysis and reports  
  - Settings and configuration  
- Connect TUI to backend API endpoints -- By Ziang Wang  

---

### **3.8 Documentation & Finalization (Week 10: Dec 8 - Dec 15)**

- Write user documentation and setup guide -- By Ping Shu
- Document API specifications -- By Ping Shu
- Create developer documentation for future maintenance -- By Muchen Liu
- Make Video Slide Presentation and Project Video Demo -- By Muchen Liu  
- Prepare final project submission -- By Ziang Wang  

---

### **Note: Agile Approach**

This plan is tentative and will be adjusted as we progress.  
We expect to iterate on earlier components (database, API) as we learn from implementing later features (TUI, analysis).



## 4. Project Setup (One-Time Setup Only)
This setup process only needs to be run ONCE when you first clone the repository.
After completing these steps, you can start developing immediately without repeating the setup process. The database and environment configuration will persist for all future development sessions.
---

### Prerequisites
Before starting, ensure you have:
- Rust and Cargo installed
- Git installed
- SQLite3 installed

#### Step 1. Clone repository into local device and create a working branch.
#### Step 2. Create .env file
**Purpose:** The .env file stores environment variables, particularly the database connection string.
```sh
cp .env.example .env
```
Replace XXXXXX placeholders in .env with your actual values if needed.

#### Step 3. Initialize database.
**Purpose:** Set up the SQLite database and apply all migrations to create the table schema.

```sh
# Install SQLx CLI if not already installed
# This tool is used for database migrations and compile-time query verification
cargo install sqlx-cli --features sqlite

# Create the database file
# This creates a new SQLite database file: personal-finance-tracker.db
sqlx database create

# Run all migrations
# This executes all .sql files in migrations/ folder to create tables, indexes, and triggers
sqlx migrate run

## Verify all tables were created
sqlite3 {your-own-db-name}.db ".tables"
# Expected output (all 7 tables should be listed):
# accounts                  recurring_transactions
# categories                transaction_categories
# exchange_rates            transactions
# users

## Populate some original data into db
cargo run db_seed

# Other commands
cargo run db_clear   // Clear all of data in the db
cargo run db_reseed   // Clear and re-seed database
cargo run help    // Print out the help text
```


## 5. Running the API Server

After completing the one-time setup, you can start the REST API server by:
```sh
cargo run serve
```

The server will start at `http://127.0.0.1:8080` by default, and you should see output like:    

Connecting to database...  
Connected to: sqlite:./{your-own-db-name}.db  
Starting web server...   
Server running at http://127.0.0.1:8080   
API Documentation:  
- Users:        GET/POST    /users  
- User:         GET/PUT/DEL /users/{id}   
- Accounts:     GET/POST    /accounts   
- Account:      GET/PUT/DEL /accounts/{id}   
- Categories:   GET/POST    /categories   
- Category:     GET/PUT/DEL /categories/{id}   
- Transactions: GET/POST    /transactions   
- Transaction:  GET/PUT/DEL /transactions/{id}      

