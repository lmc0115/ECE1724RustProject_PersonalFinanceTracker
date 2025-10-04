# ECE1724RustProject_PersonalFinanceTracker
Design and build a new command-line utility that implements a personal finance tracker, assisting users to manage their income and expenses in various categories.


## 2. Objective and Key Features

### **Project Objectives**
Our goal is to build a practical and meaningful command-line personal finance tracker that helps users manage their daily finances across multiple currencies and bank accounts. The tool will allow users to log transactions, organize them by category, and view their spending patterns over time. By using a backend server for data storage and integrating real-time exchange rate APIs, we aim to create a system that's both **accessible and secure**, while providing **accurate financial insights** regardless of which currencies users work with.


### **Key Features**

#### **1. Multi-Currency Support with Real-Time Exchange Rates**

Many people deal with multiple currencies—whether from travel, online purchases-everyday. Our personal finance tracker supports multiple bank accounts in different currencies and can fetch real-time exchange rates via API. This means the currency conversion functionality automatically retrieves current exchange rates to provide accurate conversions. Users can also see their complete financial picture across all accounts without calculating conversions between currencies manually.

#### **2. Recurring Transactions**
Expenses and income sometimes follow predictable patterns. Users can set up recurring expenses such as rent, salary, or subscriptions. When creating a recurring transaction, users specify the amount, category, frequency (monthly most of time), and start date. Our finance tracker will also automatically create these transactions at the appropriate intervals.

This feature provides two practical benefits:

- **Reduced manual entry**: Once set up, recurring transactions are logged automatically, saving time and reducing the chance of forgetting to record regular expenses or income.
- **Better financial planning**: Users can view their scheduled recurring transactions to understand expected cash flow. This helps with anticipating upcoming expenses and planning spending accordingly.

Users maintain full control over recurring transactions—they can modify amounts, pause, or cancel them as needed when circumstances change. This flexibility ensures the tool adapts to real-life situations like salary increases, subscription cancellations, or rent adjustments.

#### **3. Expense Analysis and Insights**

Our finance tracker will provide simple, text-based summaries and charts to help users understand their spending patterns.

Users can query their transaction data to generate various reports and insights:

- **Spending comparisons**: See how spending in specific categories has changed over time (e.g., "This month you spent 35% more on groceries compared to last month")
- **Category breakdowns**: View top spending categories for a given period (e.g., "Your top three spending categories are: Food, Transportation, Entertainment")
- **Custom queries**: Users can filter transactions by date range, category, account, or amount to analyze specific aspects of their finances

These insights are presented in a straightforward, text-based format that focuses on clarity. The goal is to surface useful information that helps users identify trends and make informed decisions about their spending habits.

#### **4. Data Export (CSV/JSON)**
Users can export their transaction data in standard CSV or JSON formats, which supports other use cases:

- **Tax preparation**: Export annual transaction data to share with accountants or import into tax software
- **External analysis**: Open exported CSV files in spreadsheet applications like Excel or Google Sheets for custom charts, pivot tables, or detailed analysis
- **Data portability**: Keep backup copies of financial records or migrate data to other financial management tools

The export function allows users to specify date ranges and select which accounts or categories to include, so they can export exactly the data they need rather than everything at once. Both CSV and JSON formats are supported here.


## 3. Tentative Plan

**1. Project Setup & Database Design**

- Initialize Rust project with necessary dependencies (axum, sqlx, tokio, etc.)
- Set up SQLite database
- Design database schema for accounts, transactions, categories, and recurring transactions
- Create initial migration files using sqlx-cli
- Document the database structure and relationships

**2. Database Implementation & Migration**
- Implement database migrations for all core tables
- Set up connection pooling and database configuration
- Create seed data for testing (sample accounts, categories, transactions)
- Verify database operations and constraints
- Set up automated migration testing

**3. Backend API Development - Core Features**
- Implement RESTful API endpoints for CRUD operations:
  - Account management (create, read, update, delete accounts)
  - Transaction logging (add, edit, delete transactions)
  - Category management
  - Basic querying and filtering
- Set up proper error handling and validation
- Implement API authentication and security

**4. API Testing & Validation**
- Write unit tests for database operations
- Create integration tests for API endpoints
- Test with tools like Postman
- Validate data integrity and error handling
- Document API endpoints and usage

**5. Advanced Features - Multi-Currency & Recurring Transactions**
- Integrate exchange rate API (e.g., exchangerate-api.io or similar)
- Implement currency conversion logic
- Create endpoints for managing recurring transactions
- Test currency conversion accuracy and recurring transaction automation

**6. Data Analysis & Export Features**
- Implement spending analysis algorithms
- Create endpoints for generating reports and insights
- Build CSV/JSON export functionality
- Add query filters for custom date ranges and categories
- Test export formats and analysis accuracy

**7. TUI Development with Ratatui**
- Set up ratatui framework and basic UI structure
- Implement main navigation and menu system
- Create screens for:
  - Transaction entry and viewing
  - Account management
  - Spending analysis and reports
  - Settings and configuration
- Connect TUI to backend API endpoints

**8. Documentation & Finalization**
- Write user documentation and setup guide
- Document API specifications
- Create developer documentation for future maintenance
- Make Video Slide Presentation and Project Video Demo
- Prepare final project submission

**Note: Agile Approach**.   
This plan is tentative and will be adjusted as we progress. We expect to iterate on earlier components (database, API) as we learn from implementing later features (TUI, analysis).