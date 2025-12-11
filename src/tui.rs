// tui.rs
// Text User Interface for Personal Finance Tracker

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;

use crate::models::*;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Screen {
    UserSelect,
    Dashboard,
    Accounts,
    Transactions,
    Categories,
    RecurringTransactions,
    ExchangeRates,
    Reports,
    Export,
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    AddTransaction,
    AddExchangeRate,
    AddRecurringTransaction,
    AddAccount,
    AddCategory,
    AddUser,
    DeleteConfirm,
    ViewDetails,
    ConvertCurrency,
    ExportData,
    SelectCurrencyFilter,
    SelectViewCurrency,
}

pub struct App {
    pool: SqlitePool,
    current_screen: Screen,
    selected_tab: usize,
    should_quit: bool,
    mode: Mode,

    // User selection
    current_user_id: Option<i64>,

    // Cached data
    accounts: Vec<Account>,
    transactions: Vec<Transaction>,
    categories: Vec<Category>,
    users: Vec<User>,
    exchange_rates: Vec<ExchangeRate>,
    recurring_transactions: Vec<RecurringTransaction>,
    category_spending: Vec<CategorySpendingSummary>,

    // Selection state
    selected_index: usize,
    #[allow(dead_code)]
    list_state: ListState,

    // Form data for adding transaction
    form_account_id: String,
    form_amount: String,
    form_type: String,
    form_description: String,
    form_category_id: String,
    form_field_index: usize,

    // Form data for adding exchange rate
    form_from_currency: String,
    form_to_currency: String,
    form_rate: String,
    form_source: String,

    // Form data for currency conversion
    form_convert_from: String,
    form_convert_to: String,
    form_convert_amount: String,
    form_converted_result: String,

    // Form data for recurring transaction
    form_recurring_frequency: String,

    // Form data for adding account
    form_account_name: String,
    form_account_bank: String,
    form_account_type: String,
    form_account_currency: String,
    form_account_balance: String,

    // Form data for adding category
    form_category_name: String,
    form_category_type: String,  // income or expense

    // Form data for adding user
    form_user_username: String,
    form_user_email: String,

    // Export options
    #[allow(dead_code)]
    export_format: String,
    export_message: String,

    // Currency filter for transactions view
    currency_filter: Option<String>,
    filter_currencies: Vec<String>,  // Only currencies from user's transactions (for filter)
    available_currencies: Vec<String>,  // All currencies from FX rates (for view in currency)
    
    // View in currency conversion
    view_in_currency: Option<String>,  // For Transactions screen - convert all amounts
    account_view_currency: Option<String>,  // For Account Details only - separate from transactions
    currency_scroll_offset: usize,  // Scroll offset for currency selection dialogs

    // Status message
    status_message: String,
}

impl App {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            current_screen: Screen::UserSelect,
            selected_tab: 0,
            should_quit: false,
            mode: Mode::Normal,
            current_user_id: None,
            accounts: Vec::new(),
            transactions: Vec::new(),
            categories: Vec::new(),
            users: Vec::new(),
            exchange_rates: Vec::new(),
            recurring_transactions: Vec::new(),
            category_spending: Vec::new(),
            selected_index: 0,
            list_state: ListState::default(),
            form_account_id: String::new(),
            form_amount: String::new(),
            form_type: String::from("expense"),
            form_description: String::new(),
            form_category_id: String::new(),
            form_field_index: 0,
            form_from_currency: String::new(),
            form_to_currency: String::new(),
            form_rate: String::new(),
            form_source: String::from("manual"),
            form_convert_from: String::new(),
            form_convert_to: String::new(),
            form_convert_amount: String::new(),
            form_converted_result: String::new(),
            form_recurring_frequency: String::from("monthly"),
            form_account_name: String::new(),
            form_account_bank: String::new(),
            form_account_type: String::from("checking"),
            form_account_currency: String::from("USD"),
            form_account_balance: String::from("0"),
            form_category_name: String::new(),
            form_category_type: String::from("expense"),
            form_user_username: String::new(),
            form_user_email: String::new(),
            export_format: String::from("csv"),
            export_message: String::new(),
            currency_filter: None,
            filter_currencies: Vec::new(),
            available_currencies: Vec::new(),
            view_in_currency: None,
            account_view_currency: None,
            currency_scroll_offset: 0,
            status_message: String::new(),
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        // Load users first
        self.load_users().await;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        while !self.should_quit {
            terminal.draw(|f| self.ui(f))?;
            self.handle_events().await?;
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    async fn load_users(&mut self) {
        if let Ok(users) = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
        {
            self.users = users;
        }
    }

    async fn load_data(&mut self) {
        if self.current_user_id.is_none() {
            return;
        }

        let user_id = self.current_user_id.unwrap();

        // Load accounts for current user
        if let Ok(accounts) = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        {
            self.accounts = accounts;
        }

        // Get account IDs for this user
        let account_ids: Vec<i64> = self.accounts.iter().map(|a| a.id).collect();

        // Load transactions for user's accounts
        if !account_ids.is_empty() {
            let placeholders = account_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",");
            let query = format!(
                "SELECT * FROM transactions WHERE account_id IN ({}) ORDER BY transaction_date DESC LIMIT 100",
                placeholders
            );

            let mut q = sqlx::query_as::<_, Transaction>(&query);
            for id in &account_ids {
                q = q.bind(*id);
            }

            if let Ok(transactions) = q.fetch_all(&self.pool).await {
                self.transactions = transactions;
            }
        } else {
            self.transactions.clear();
        }

        // Load categories for current user
        if let Ok(categories) = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories WHERE user_id = ? ORDER BY name",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        {
            self.categories = categories;
        }

        // Load exchange rates - get the most recent rate for each currency pair
        // Use subquery to get only the latest rate per pair to avoid duplicates
        if let Ok(rates) = sqlx::query_as::<_, ExchangeRate>(
            "SELECT e1.* FROM exchange_rates e1
             INNER JOIN (
                 SELECT from_currency, to_currency, MAX(rate_date) as max_date
                 FROM exchange_rates
                 GROUP BY from_currency, to_currency
             ) e2 ON e1.from_currency = e2.from_currency 
                  AND e1.to_currency = e2.to_currency 
                  AND e1.rate_date = e2.max_date
             ORDER BY e1.from_currency, e1.to_currency",
        )
        .fetch_all(&self.pool)
        .await
        {
            self.exchange_rates = rates;
        }

        // Load recurring transactions for user's accounts
        if !account_ids.is_empty() {
            let placeholders = account_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",");
            let query = format!(
                "SELECT * FROM recurring_transactions WHERE account_id IN ({}) ORDER BY next_occurrence ASC",
                placeholders
            );

            let mut q = sqlx::query_as::<_, RecurringTransaction>(&query);
            for id in &account_ids {
                q = q.bind(*id);
            }

            if let Ok(recurring) = q.fetch_all(&self.pool).await {
                self.recurring_transactions = recurring;
            }
        } else {
            self.recurring_transactions.clear();
        }

        // Load category spending summary
        self.load_category_spending().await;
    }

    async fn load_category_spending(&mut self) {
        if self.current_user_id.is_none() {
            return;
        }

        let user_id = self.current_user_id.unwrap();

        // Query that only shows categories with actual spending (INNER JOIN instead of LEFT JOIN)
        let query = format!(
            "SELECT c.id as category_id, c.name as category_name,
                    SUM(ABS(tc.amount)) as total_amount,
                    COUNT(DISTINCT t.id) as transaction_count
             FROM transaction_categories tc
             INNER JOIN categories c ON tc.category_id = c.id
             INNER JOIN transactions t ON tc.transaction_id = t.id
             INNER JOIN accounts a ON t.account_id = a.id
             WHERE a.user_id = {} AND t.transaction_type = 'expense'
             GROUP BY c.id, c.name
             HAVING total_amount > 0
             ORDER BY total_amount DESC",
            user_id
        );

        if let Ok(spending) = sqlx::query_as::<_, CategorySpendingSummary>(&query)
            .fetch_all(&self.pool)
            .await
        {
            self.category_spending = spending;
        }

        // Build filter_currencies: only currencies from accounts that have transactions
        let mut filter_currency_codes: std::collections::HashSet<String> = std::collections::HashSet::new();
        for t in &self.transactions {
            if let Some(account) = self.accounts.iter().find(|a| a.id == t.account_id) {
                filter_currency_codes.insert(account.currency.clone());
            }
        }
        self.filter_currencies = filter_currency_codes.into_iter().collect();
        self.filter_currencies.sort();
        
        // Collect all available currencies from accounts AND exchange rates (for View in Currency)
        // Use HashSet to deduplicate, but store as (code, display_name) pairs
        let mut currency_codes: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut code_to_display: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        
        // Add currencies from user's accounts
        for a in &self.accounts {
            let code = Self::extract_currency_code(&a.currency);
            currency_codes.insert(code.clone());
            // Account currencies are usually just codes, so display as-is
            code_to_display.entry(code).or_insert(a.currency.clone());
        }
        
        // Add currencies from exchange rates (both from and to)
        for r in &self.exchange_rates {
            let from_code = Self::extract_currency_code(&r.from_currency);
            let to_code = Self::extract_currency_code(&r.to_currency);
            
            currency_codes.insert(from_code.clone());
            currency_codes.insert(to_code.clone());
            
            // Prefer full names with codes like "Argentine Peso (ARS)" over just "ARS"
            if r.from_currency.contains('(') {
                code_to_display.insert(from_code, r.from_currency.clone());
            }
            if r.to_currency.contains('(') {
                code_to_display.insert(to_code, r.to_currency.clone());
            }
        }
        
        // Build available currencies list with nice display names (for View in Currency)
        self.available_currencies = currency_codes.into_iter()
            .map(|code| code_to_display.get(&code).cloned().unwrap_or(code))
            .collect();
        self.available_currencies.sort();
    }

    fn ui(&self, frame: &mut Frame) {
        if self.current_screen == Screen::UserSelect {
            if self.mode == Mode::AddUser {
                self.render_add_user_form(frame);
            } else if self.mode == Mode::DeleteConfirm {
                self.render_delete_user_confirm(frame);
            } else {
                self.render_user_select(frame);
            }
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer/Status
            ])
            .split(frame.area());

        // Header
        self.render_header(frame, chunks[0]);

        // Tabs
        self.render_tabs(frame, chunks[1]);

        // Content based on mode
        match self.mode {
            Mode::Normal => match self.current_screen {
                Screen::Dashboard => self.render_dashboard(frame, chunks[2]),
                Screen::Accounts => self.render_accounts(frame, chunks[2]),
                Screen::Transactions => self.render_transactions(frame, chunks[2]),
                Screen::Categories => self.render_categories(frame, chunks[2]),
                Screen::RecurringTransactions => self.render_recurring_transactions(frame, chunks[2]),
                Screen::ExchangeRates => self.render_exchange_rates(frame, chunks[2]),
                Screen::Reports => self.render_reports(frame, chunks[2]),
                Screen::Export => self.render_export(frame, chunks[2]),
                Screen::UserSelect => {}
            },
            Mode::AddTransaction => self.render_add_transaction_form(frame, chunks[2]),
            Mode::AddExchangeRate => self.render_add_exchange_rate_form(frame, chunks[2]),
            Mode::AddRecurringTransaction => self.render_add_recurring_form(frame, chunks[2]),
            Mode::AddAccount => self.render_add_account_form(frame, chunks[2]),
            Mode::AddCategory => self.render_add_category_form(frame, chunks[2]),
            Mode::AddUser => {} // Handled separately in ui()
            Mode::ConvertCurrency => self.render_currency_conversion(frame, chunks[2]),
            Mode::DeleteConfirm => self.render_delete_confirm(frame, chunks[2]),
            Mode::ViewDetails => self.render_details(frame, chunks[2]),
            Mode::ExportData => self.render_export_dialog(frame, chunks[2]),
            Mode::SelectCurrencyFilter => self.render_currency_filter_dialog(frame, chunks[2]),
            Mode::SelectViewCurrency => self.render_view_currency_dialog(frame, chunks[2]),
        }

        // Footer
        self.render_footer(frame, chunks[3]);
    }

    fn render_user_select(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Title
                Constraint::Min(0),    // User list
                Constraint::Length(3), // Instructions
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "Personal Finance Tracker",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Select a User to Continue",
                Style::default().fg(Color::Yellow),
            )]),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // User list
        let user_items: Vec<ListItem> = self
            .users
            .iter()
            .enumerate()
            .map(|(i, u)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else if i % 2 == 0 {
                    Style::default()
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 30))
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("ID: {} ", u.id), Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{:<20}", u.username),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(format!(" ({})", u.email), Style::default().fg(Color::Cyan)),
                ]))
                .style(style)
            })
            .collect();

        let list = List::new(user_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Available Users ({}) - a: Add New User", self.users.len())),
        );
        frame.render_widget(list, chunks[1]);

        // Instructions
        let instructions =
            Paragraph::new("↑↓: Select | Enter: Login | a: Add User | d: Delete User | q: Quit")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
        frame.render_widget(instructions, chunks[2]);
    }

    fn render_header(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let mode_indicator = match self.mode {
            Mode::Normal => "",
            Mode::AddTransaction => " [ADD TRANSACTION]",
            Mode::AddExchangeRate => " [ADD EXCHANGE RATE]",
            Mode::AddRecurringTransaction => " [ADD RECURRING]",
            Mode::AddAccount => " [ADD ACCOUNT]",
            Mode::AddCategory => " [ADD CATEGORY]",
            Mode::AddUser => " [ADD USER]",
            Mode::ConvertCurrency => " [CONVERT CURRENCY]",
            Mode::DeleteConfirm => " [DELETE CONFIRM]",
            Mode::ViewDetails => " [DETAILS]",
            Mode::ExportData => " [EXPORT DATA]",
            Mode::SelectCurrencyFilter => " [FILTER CURRENCY]",
            Mode::SelectViewCurrency => " [VIEW IN CURRENCY]",
        };

        let current_user = if let Some(user_id) = self.current_user_id {
            if let Some(user) = self.users.iter().find(|u| u.id == user_id) {
                format!(" - User: {}", user.username)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let title = Paragraph::new(format!(
            "Personal Finance Tracker{}{}",
            current_user, mode_indicator
        ))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, area);
    }

    fn render_tabs(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let titles = vec![
            "Dashboard",
            "Accounts",
            "Transactions",
            "Categories",
            "Recurring",
            "FX Rates",
            "Reports",
            "Export",
        ];
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Menu (1-8)"))
            .select(self.selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(tabs, area);
    }

    fn render_dashboard(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(area);

        let total_accounts = self.accounts.len();
        let total_balance: f64 = self.accounts.iter().map(|a| a.current_balance).sum();

        let this_month_income: f64 = self
            .transactions
            .iter()
            .filter(|t| t.transaction_type == "income")
            .map(|t| t.amount)
            .sum();

        let this_month_expenses: f64 = self
            .transactions
            .iter()
            .filter(|t| t.transaction_type == "expense")
            .map(|t| t.amount.abs())
            .sum();

        let net_change = this_month_income - this_month_expenses;

        let stats_text = vec![
            Line::from(vec![
                Span::styled("Total Accounts: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", total_accounts),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Balance: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("${:.2}", total_balance),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("This Month Income: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("${:.2}", this_month_income),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("This Month Expenses: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("${:.2}", this_month_expenses),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(vec![
                Span::styled("Net Change: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "{}{:.2}",
                        if net_change >= 0.0 { "+" } else { "" },
                        net_change
                    ),
                    if net_change >= 0.0 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
            ]),
        ];

        let stats = Paragraph::new(stats_text)
            .block(Block::default().borders(Borders::ALL).title("Quick Stats"))
            .alignment(Alignment::Left);
        frame.render_widget(stats, chunks[0]);

        let transactions: Vec<ListItem> = self
            .transactions
            .iter()
            .take(10)
            .enumerate()
            .map(|(i, t)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let icon = if t.transaction_type == "income" {
                    "+"
                } else {
                    "-"
                };
                let desc = t.description.as_deref().unwrap_or("No description");
                ListItem::new(format!("{} ${:.2} - {}", icon, t.amount.abs(), desc)).style(style)
            })
            .collect();

        let list = List::new(transactions).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Transactions (Up/Down to select, Enter for details)"),
        );
        frame.render_widget(list, chunks[1]);
    }

    fn render_accounts(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let accounts: Vec<ListItem> = self
            .accounts
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else if i % 2 == 0 {
                    Style::default()
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 30))
                };

                let balance_color = if a.current_balance >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };

                let bank = a.bank_name.as_deref().unwrap_or("No Bank");

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<30}", a.name), Style::default().fg(Color::White)),
                    Span::styled(format!("{:<15}", bank), Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{:>12.2} {}", a.current_balance, a.currency),
                        Style::default().fg(balance_color),
                    ),
                ]))
                .style(style)
            })
            .collect();

        let total = self.accounts.len();
        let pos_indicator = if total > 0 {
            format!(" [{}/{}]", self.selected_index + 1, total)
        } else {
            String::new()
        };

        let list = List::new(accounts)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Accounts ({}){} - a: Add | d: Delete | ↑↓: Scroll | Enter: Details", total, pos_indicator)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_transactions(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Filter transactions by currency if filter is set
        let filtered_transactions: Vec<_> = if let Some(ref currency) = self.currency_filter {
            self.transactions
                .iter()
                .filter(|t| {
                    // Find the account for this transaction and check its currency
                    self.accounts
                        .iter()
                        .find(|a| a.id == t.account_id)
                        .map(|a| &a.currency == currency)
                        .unwrap_or(false)
                })
                .collect()
        } else {
            self.transactions.iter().collect()
        };

        let transactions: Vec<ListItem> = filtered_transactions
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else if i % 2 == 0 {
                    Style::default()
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 30))
                };

                let date_str = t.transaction_date.format("%Y-%m-%d").to_string();
                let type_str = match t.transaction_type.as_str() {
                    "income" => "Income  ",
                    "expense" => "Expense ",
                    "transfer" => "Transfer",
                    _ => "Unknown ",
                };
                let desc = t.description.as_deref().unwrap_or("No description");

                // Get currency from account
                let original_currency = self.accounts
                    .iter()
                    .find(|a| a.id == t.account_id)
                    .map(|a| a.currency.as_str())
                    .unwrap_or("???");

                // Determine display amount and currency
                let (display_amount, display_currency) = if let Some(ref target_currency) = self.view_in_currency {
                    let rate = self.get_exchange_rate(original_currency, target_currency);
                    (t.amount.abs() * rate, target_currency.as_str())
                } else {
                    (t.amount.abs(), original_currency)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", date_str), Style::default().fg(Color::Gray)),
                    Span::styled(format!("{} ", type_str), 
                        if t.transaction_type == "income" {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::Red)
                        }
                    ),
                    Span::styled(format!("{:>10.2} ", display_amount), Style::default().fg(Color::White)),
                    Span::styled(format!("{:<4}", display_currency), Style::default().fg(Color::Cyan)),
                    if self.view_in_currency.is_some() && original_currency != display_currency {
                        Span::styled(format!("({})", original_currency), Style::default().fg(Color::DarkGray))
                    } else {
                        Span::raw("")
                    },
                    Span::styled(format!(" | {}", desc), Style::default().fg(Color::White)),
                ]))
                .style(style)
            })
            .collect();

        // Build title with filter and view info
        let filter_str = match &self.currency_filter {
            Some(c) => format!(" [Filter: {}]", c),
            None => String::new(),
        };
        
        let view_str = match &self.view_in_currency {
            Some(c) => format!(" [View: {}]", c),
            None => String::new(),
        };

        let total = filtered_transactions.len();
        let pos_indicator = if total > 0 {
            format!(" [{}/{}]", self.selected_index + 1, total)
        } else {
            String::new()
        };

        let list = List::new(transactions)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Transactions ({}){}{}{} - f: Filter | v: View in $ | ↑↓: Scroll",
                total, filter_str, view_str, pos_indicator
            )))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        // Use stateful widget for proper scrolling
        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_categories(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let total = self.categories.len();
        let pos_indicator = if total > 0 && self.selected_index < total {
            format!(" [{}/{}]", self.selected_index + 1, total)
        } else { String::new() };

        let cat_items: Vec<ListItem> = self.categories.iter()
            .enumerate()
            .map(|(i, c)| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                } else if i % 2 == 0 {
                    Style::default()
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 30))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:>3}: ", c.id), Style::default().fg(Color::Cyan)),
                    Span::styled(&c.name, Style::default().fg(Color::White)),
                ])).style(style)
            })
            .collect();

        let list = List::new(cat_items)
            .block(Block::default().borders(Borders::ALL)
                .title(format!("Categories ({}){} - a: Add | d: Delete | ↑↓: Scroll", total, pos_indicator)))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("► ");

        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_exchange_rates(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let rate_items: Vec<ListItem> = self
            .exchange_rates
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else if i % 2 == 0 {
                    Style::default()
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 30))
                };

                let date_str = r.rate_date.format("%Y-%m-%d").to_string();

                // Truncate long currency names for display
                let to_curr_display = if r.to_currency.len() > 25 {
                    format!("{}...", &r.to_currency[..22])
                } else {
                    r.to_currency.clone()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:<5}", r.from_currency),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(" → ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{:<26}", to_curr_display),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("{:>12.6}", r.rate),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(format!("  {}", date_str), Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!(" [{}]", r.source),
                        Style::default().fg(Color::Yellow),
                    ),
                ]))
                .style(style)
            })
            .collect();

        // Calculate visible position indicator
        let total = self.exchange_rates.len();
        let pos_indicator = if total > 0 {
            format!(" [{}/{}]", self.selected_index + 1, total)
        } else {
            String::new()
        };

        let list = List::new(rate_items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Exchange Rates ({}) - a: Add | c: Convert | d: Delete | ↑↓: Scroll{}",
                total, pos_indicator
            )))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        // Use stateful widget for proper scrolling
        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_reports(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Summary
                Constraint::Length(12), // Top categories
                Constraint::Min(0),     // Account balances
            ])
            .split(area);

        let total_income: f64 = self
            .transactions
            .iter()
            .filter(|t| t.transaction_type == "income")
            .map(|t| t.amount)
            .sum();

        let total_expenses: f64 = self
            .transactions
            .iter()
            .filter(|t| t.transaction_type == "expense")
            .map(|t| t.amount.abs())
            .sum();

        let net_change = total_income - total_expenses;
        let transaction_count = self.transactions.len();

        let summary_text = vec![
            Line::from(vec![
                Span::styled("Report Period: ", Style::default().fg(Color::Gray)),
                Span::styled("All Time", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Total Income:       ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("${:.2}", total_income),
                    Style::default().fg(Color::Green),
                ),
                Span::styled("     Transaction Count:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", transaction_count),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Expenses:     ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("${:.2}", total_expenses),
                    Style::default().fg(Color::Red),
                ),
                Span::styled("     Categories:         ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", self.categories.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Net Change:         ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "{}{:.2}",
                        if net_change >= 0.0 { "+" } else { "" },
                        net_change
                    ),
                    if net_change >= 0.0 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
                Span::styled("     Recurring:          ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", self.recurring_transactions.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let summary = Paragraph::new(summary_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Financial Summary"),
            )
            .alignment(Alignment::Left);
        frame.render_widget(summary, chunks[0]);

        // Top spending categories (Insights)
        let top_categories: Vec<ListItem> = self
            .category_spending
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, cs)| {
                let percentage = if total_expenses > 0.0 {
                    (cs.total_amount / total_expenses) * 100.0
                } else {
                    0.0
                };

                // Create a simple bar
                let bar_width = (percentage / 100.0 * 20.0) as usize;
                let bar = "█".repeat(bar_width);

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{}. ", i + 1),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        format!("{:<20}", cs.category_name),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("${:>10.2}", cs.total_amount),
                        Style::default().fg(Color::Red),
                    ),
                    Span::styled(
                        format!(" ({:>5.1}%) ", percentage),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(bar, Style::default().fg(Color::Magenta)),
                ]))
            })
            .collect();

        let category_list = List::new(top_categories).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Top Spending Categories (Insights)"),
        );
        frame.render_widget(category_list, chunks[1]);

        // Account balances
        let account_items: Vec<ListItem> = self
            .accounts
            .iter()
            .map(|a| {
                let balance_str = format!("{:.2} {}", a.current_balance, a.currency);
                let color = if a.current_balance >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<30}", a.name), Style::default().fg(Color::White)),
                    Span::styled(format!("{:>15}", balance_str), Style::default().fg(color)),
                ]))
            })
            .collect();

        let list = List::new(account_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Account Balances"),
        );
        frame.render_widget(list, chunks[2]);
    }

    fn render_recurring_transactions(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let recurring_items: Vec<ListItem> = self
            .recurring_transactions
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else if i % 2 == 0 {
                    Style::default()
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 30))
                };

                let status = if r.is_active { "Active" } else { "Paused" };
                let status_color = if r.is_active { Color::Green } else { Color::Red };
                let next_date = r.next_occurrence.format("%Y-%m-%d").to_string();
                let desc = r.description.as_deref().unwrap_or("No description");

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", status),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(
                        format!("{:<10}", r.frequency),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!(" ${:>10.2}", r.amount.abs()),
                        if r.transaction_type == "income" {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::Red)
                        },
                    ),
                    Span::styled(format!(" | Next: {} ", next_date), Style::default().fg(Color::Gray)),
                    Span::styled(desc, Style::default().fg(Color::White)),
                ]))
                .style(style)
            })
            .collect();

        let total = self.recurring_transactions.len();
        let pos_indicator = if total > 0 {
            format!(" [{}/{}]", self.selected_index + 1, total)
        } else {
            String::new()
        };

        let list = List::new(recurring_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "Recurring ({}) {} - a: Add | p: Process | t: Toggle | d: Delete | ↑↓: Scroll",
                        total, pos_indicator
                    )),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_export(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Instructions
                Constraint::Min(0),     // Export options
            ])
            .split(area);

        let instructions = vec![
            Line::from(vec![Span::styled(
                "Data Export Options",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("e", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to export data for the current user.", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from("Supported export formats:"),
            Line::from(vec![
                Span::styled("  • CSV  ", Style::default().fg(Color::Green)),
                Span::styled("- Comma-separated values for spreadsheets", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("  • JSON ", Style::default().fg(Color::Green)),
                Span::styled("- Structured data for other applications", Style::default().fg(Color::Gray)),
            ]),
        ];

        let instr_widget = Paragraph::new(instructions)
            .block(Block::default().borders(Borders::ALL).title("Export Data"))
            .alignment(Alignment::Left);
        frame.render_widget(instr_widget, chunks[0]);

        // Show export status or message
        let export_content = if !self.export_message.is_empty() {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    &self.export_message,
                    Style::default().fg(Color::Green),
                )]),
            ]
        } else {
            let mut lines = vec![
                Line::from(vec![Span::styled(
                    "Data Available for Export:",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(""),
            ];

            lines.push(Line::from(format!("  • {} Accounts", self.accounts.len())));
            lines.push(Line::from(format!("  • {} Transactions", self.transactions.len())));
            lines.push(Line::from(format!("  • {} Categories", self.categories.len())));
            lines.push(Line::from(format!("  • {} Recurring Transactions", self.recurring_transactions.len())));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Note: ", Style::default().fg(Color::Gray)),
                Span::styled("Export files will be saved to the current directory.", Style::default().fg(Color::Gray)),
            ]));
            lines
        };

        let export_widget = Paragraph::new(export_content)
            .block(Block::default().borders(Borders::ALL).title("Export Status"))
            .alignment(Alignment::Left);
        frame.render_widget(export_widget, chunks[1]);
    }

    fn render_add_recurring_form(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        let (selected_account_name, selected_currency) = if let Ok(account_id) = self.form_account_id.parse::<i64>() {
            self.accounts.iter().find(|a| a.id == account_id)
                .map(|a| (a.name.clone(), a.currency.clone())).unwrap_or_default()
        } else { (String::new(), String::new()) };

        let currency_display = if !selected_currency.is_empty() {
            format!(" → {} [{}]", selected_account_name, selected_currency)
        } else { String::new() };

        let form_text = vec![
            Line::from(vec![Span::styled("Add Recurring Transaction",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Account: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_account_id,
                    if self.form_field_index == 0 {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else { Style::default().fg(Color::White) }),
                Span::styled(&currency_display, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Amount: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_amount,
                    if self.form_field_index == 1 {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else { Style::default().fg(Color::White) }),
                Span::styled(if !selected_currency.is_empty() { format!(" {}", selected_currency) } else { String::new() },
                    Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Type (i/e): ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_type,
                    if self.form_field_index == 2 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_description,
                    if self.form_field_index == 3 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Category ID: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_category_id,
                    if self.form_field_index == 4 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Frequency: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_recurring_frequency,
                    if self.form_field_index == 5 {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else { Style::default().fg(Color::White) }),
                Span::styled(" (daily/weekly/monthly/yearly)", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("Tab: Next | Enter: Submit | Esc: Cancel", Style::default().fg(Color::Cyan))]),
        ];

        let form = Paragraph::new(form_text)
            .block(Block::default().borders(Borders::ALL).title("Add Recurring"))
            .alignment(Alignment::Left);
        frame.render_widget(form, chunks[0]);

        // Account list grouped by currency
        let mut lines: Vec<Line> = vec![Line::from("Select by Currency:"), Line::from("")];
        let mut currencies: Vec<String> = self.accounts.iter().map(|a| a.currency.clone()).collect();
        currencies.sort(); currencies.dedup();
        for curr in &currencies {
            lines.push(Line::from(Span::styled(format!("── {} ──", curr), Style::default().fg(Color::Green))));
            for a in self.accounts.iter().filter(|a| &a.currency == curr) {
                let sel = self.form_account_id.parse::<i64>().ok() == Some(a.id);
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}: {}", a.id, a.name),
                        if sel { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) }),
                    if sel { Span::styled(" ◄", Style::default().fg(Color::Yellow)) } else { Span::raw("") },
                ]));
            }
        }
        frame.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Accounts")), chunks[1]);
    }

    fn render_export_dialog(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let dialog_text = vec![
            Line::from(vec![Span::styled(
                "Select Export Format",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("1", Style::default().fg(Color::Cyan)),
                Span::styled(" - Export Transactions as CSV", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("2", Style::default().fg(Color::Cyan)),
                Span::styled(" - Export Transactions as JSON", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("3", Style::default().fg(Color::Cyan)),
                Span::styled(" - Export Accounts as CSV", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("4", Style::default().fg(Color::Cyan)),
                Span::styled(" - Export Full Summary as JSON", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press number to export, Esc to cancel",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Export Options"),
            )
            .alignment(Alignment::Center);
        frame.render_widget(dialog, area);
    }

    fn render_currency_filter_dialog(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let current_filter = match &self.currency_filter {
            Some(c) => format!("Current: {}", c),
            None => "Current: All".to_string(),
        };

        let mut dialog_lines = vec![
            Line::from(vec![Span::styled(
                "Filter Transactions by Currency",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(current_filter, Style::default().fg(Color::Gray))]),
            Line::from(""),
            Line::from(vec![
                Span::styled("0", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" - ", Style::default().fg(Color::Gray)),
                Span::styled("Show ALL Currencies", Style::default().fg(Color::White)),
                if self.currency_filter.is_none() {
                    Span::styled(" ◄ active", Style::default().fg(Color::Green))
                } else { Span::raw("") },
            ]),
            Line::from(""),
        ];

        // Add each currency from transactions dynamically (filter_currencies, not all available)
        if self.filter_currencies.is_empty() {
            dialog_lines.push(Line::from(vec![Span::styled(
                "No currencies found in transactions", Style::default().fg(Color::Red)
            )]));
        } else {
            for (i, currency) in self.filter_currencies.iter().enumerate() {
                let is_selected = self.currency_filter.as_ref() == Some(currency);
                let key = if i < 9 { format!("{}", i + 1) } else { format!("{}", (b'a' + (i - 9) as u8) as char) };
                let active_marker = if is_selected { " ◄ active" } else { "" };
                dialog_lines.push(Line::from(vec![
                    Span::styled(key, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(" - ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{}{}", currency, active_marker), 
                        if is_selected { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) }
                        else { Style::default().fg(Color::White) }),
                ]));
            }
        }

        dialog_lines.push(Line::from(""));
        dialog_lines.push(Line::from(vec![Span::styled(
            format!("Found {} currencies in transactions | Press key to filter | Esc: cancel", self.filter_currencies.len()),
            Style::default().fg(Color::Gray),
        )]));

        let dialog = Paragraph::new(dialog_lines)
            .block(Block::default().borders(Borders::ALL).title("Currency Filter"))
            .alignment(Alignment::Center);
        frame.render_widget(dialog, area);
    }

    fn render_add_transaction_form(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Split into form and account list
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        // Get selected account info
        let (selected_account_name, selected_currency) = if let Ok(account_id) = self.form_account_id.parse::<i64>() {
            self.accounts
                .iter()
                .find(|a| a.id == account_id)
                .map(|a| (a.name.clone(), a.currency.clone()))
                .unwrap_or((String::new(), String::new()))
        } else {
            (String::new(), String::new())
        };

        let currency_display = if !selected_currency.is_empty() {
            format!(" → {} [{}]", selected_account_name, selected_currency)
        } else {
            " (enter ID or use shortcuts on right)".to_string()
        };

        let form_text = vec![
            Line::from(vec![Span::styled(
                "Add New Transaction",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Account: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_account_id,
                    if self.form_field_index == 0 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::styled(&currency_display, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Amount: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_amount,
                    if self.form_field_index == 1 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::styled(
                    if !selected_currency.is_empty() { format!(" {}", selected_currency) } else { String::new() },
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Type (i=income/e=expense): ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_type,
                    if self.form_field_index == 2 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_description,
                    if self.form_field_index == 3 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Category ID: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_category_id,
                    if self.form_field_index == 4 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Tab: Next Field | Enter: Submit | Esc: Cancel",
                Style::default().fg(Color::Cyan),
            )]),
        ];

        let form = Paragraph::new(form_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Add Transaction Form"),
            )
            .alignment(Alignment::Left);
        frame.render_widget(form, chunks[0]);

        // Account list with currency - grouped by currency
        let mut account_lines: Vec<Line> = vec![
            Line::from(vec![Span::styled(
                "Quick Select (type number in Account field):",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(""),
        ];

        // Group accounts by currency
        let mut currencies: Vec<String> = self.accounts.iter().map(|a| a.currency.clone()).collect();
        currencies.sort();
        currencies.dedup();

        for currency in &currencies {
            account_lines.push(Line::from(vec![Span::styled(
                format!("━━━ {} ━━━", currency),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )]));
            
            for a in self.accounts.iter().filter(|a| &a.currency == currency) {
                let is_selected = self.form_account_id.parse::<i64>().ok() == Some(a.id);
                account_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {}: ", a.id),
                        if is_selected {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Cyan)
                        },
                    ),
                    Span::styled(
                        format!("{}", a.name),
                        if is_selected {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                    if is_selected {
                        Span::styled(" ◄", Style::default().fg(Color::Yellow))
                    } else {
                        Span::raw("")
                    },
                ]));
            }
            account_lines.push(Line::from(""));
        }

        let account_paragraph = Paragraph::new(account_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Select Account (grouped by Currency)"),
            )
            .alignment(Alignment::Left);
        frame.render_widget(account_paragraph, chunks[1]);
    }

    fn render_add_exchange_rate_form(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let form_text = vec![
            Line::from(vec![Span::styled(
                "Add New Exchange Rate",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "From Currency (e.g., USD): ",
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    &self.form_from_currency,
                    if self.form_field_index == 0 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "To Currency (e.g., Euro (EUR)): ",
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    &self.form_to_currency,
                    if self.form_field_index == 1 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Rate: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_rate,
                    if self.form_field_index == 2 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Source (manual/api/scraper): ",
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    &self.form_source,
                    if self.form_field_index == 3 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Tab: Next field | Enter: Submit | Esc: Cancel",
                Style::default().fg(Color::Cyan),
            )]),
        ];

        let form = Paragraph::new(form_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Add Exchange Rate Form"),
            )
            .alignment(Alignment::Left);
        frame.render_widget(form, area);
    }

    fn render_currency_conversion(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let form_text = vec![
            Line::from(vec![Span::styled(
                "Currency Conversion",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("From Currency: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_convert_from,
                    if self.form_field_index == 0 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("To Currency: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_convert_to,
                    if self.form_field_index == 1 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Amount: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_convert_amount,
                    if self.form_field_index == 2 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Result: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.form_converted_result,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Tab: Next field | Enter: Convert | Esc: Cancel",
                Style::default().fg(Color::Cyan),
            )]),
        ];

        let form = Paragraph::new(form_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Currency Conversion"),
            )
            .alignment(Alignment::Left);
        frame.render_widget(form, area);
    }

    fn render_delete_confirm(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let confirm_text = if self.current_screen == Screen::Transactions
            && self.selected_index < self.transactions.len()
        {
            let t = &self.transactions[self.selected_index];
            vec![
                Line::from(vec![Span::styled(
                    "Delete Transaction?",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("Amount: ${:.2}", t.amount)),
                Line::from(format!("Type: {}", t.transaction_type)),
                Line::from(format!(
                    "Description: {}",
                    t.description.as_deref().unwrap_or("N/A")
                )),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press 'y' to confirm, 'n' to cancel",
                    Style::default().fg(Color::Yellow),
                )]),
            ]
        } else if self.current_screen == Screen::Accounts
            && self.selected_index < self.accounts.len()
        {
            let a = &self.accounts[self.selected_index];
            vec![
                Line::from(vec![Span::styled(
                    "⚠ DELETE ACCOUNT?",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("Name: {}", a.name)),
                Line::from(format!("Bank: {}", a.bank_name.as_deref().unwrap_or("N/A"))),
                Line::from(format!("Balance: {:.2} {}", a.current_balance, a.currency)),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "⚠ WARNING: All transactions for this account",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![Span::styled(
                    "  will also be PERMANENTLY deleted!",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press 'y' to confirm, 'n' to cancel",
                    Style::default().fg(Color::Cyan),
                )]),
            ]
        } else if self.current_screen == Screen::Categories
            && self.selected_index < self.categories.len()
        {
            let c = &self.categories[self.selected_index];
            vec![
                Line::from(vec![Span::styled(
                    "Delete Category?",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("ID: {}", c.id)),
                Line::from(format!("Name: {}", c.name)),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "⚠ Categories linked to transactions cannot be deleted",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press 'y' to confirm, 'n' to cancel",
                    Style::default().fg(Color::Yellow),
                )]),
            ]
        } else if self.current_screen == Screen::ExchangeRates
            && self.selected_index < self.exchange_rates.len()
        {
            let r = &self.exchange_rates[self.selected_index];
            vec![
                Line::from(vec![Span::styled(
                    "Delete Exchange Rate?",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("From: {}", r.from_currency)),
                Line::from(format!("To: {}", r.to_currency)),
                Line::from(format!("Rate: {:.6}", r.rate)),
                Line::from(format!("Date: {}", r.rate_date.format("%Y-%m-%d"))),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press 'y' to confirm, 'n' to cancel",
                    Style::default().fg(Color::Yellow),
                )]),
            ]
        } else if self.current_screen == Screen::RecurringTransactions
            && self.selected_index < self.recurring_transactions.len()
        {
            let r = &self.recurring_transactions[self.selected_index];
            vec![
                Line::from(vec![Span::styled(
                    "Delete Recurring Transaction?",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("Amount: ${:.2}", r.amount)),
                Line::from(format!("Type: {}", r.transaction_type)),
                Line::from(format!("Frequency: {}", r.frequency)),
                Line::from(format!(
                    "Description: {}",
                    r.description.as_deref().unwrap_or("N/A")
                )),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press 'y' to confirm, 'n' to cancel",
                    Style::default().fg(Color::Yellow),
                )]),
            ]
        } else {
            vec![Line::from("Invalid selection")]
        };

        let confirm = Paragraph::new(confirm_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Confirm Delete"),
            )
            .alignment(Alignment::Center);
        frame.render_widget(confirm, area);
    }

    fn render_details(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let details_text = if self.current_screen == Screen::Transactions
            && self.selected_index < self.transactions.len()
        {
            let t = &self.transactions[self.selected_index];
            vec![
                Line::from(vec![Span::styled(
                    "Transaction Details",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("ID: {}", t.id)),
                Line::from(format!("Account ID: {}", t.account_id)),
                Line::from(format!("Amount: ${:.2}", t.amount)),
                Line::from(format!("Type: {}", t.transaction_type)),
                Line::from(format!(
                    "Description: {}",
                    t.description.as_deref().unwrap_or("No description")
                )),
                Line::from(format!(
                    "Date: {}",
                    t.transaction_date.format("%Y-%m-%d %H:%M:%S")
                )),
                Line::from(format!(
                    "Created: {}",
                    t.created_at.format("%Y-%m-%d %H:%M:%S")
                )),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press Esc to go back",
                    Style::default().fg(Color::Gray),
                )]),
            ]
        } else if self.current_screen == Screen::Accounts
            && self.selected_index < self.accounts.len()
        {
            let a = &self.accounts[self.selected_index];
            
            // Get transactions for this account
            let account_txns: Vec<_> = self.transactions.iter()
                .filter(|t| t.account_id == a.id)
                .collect();
            
            // Determine display currency (use account-specific view currency)
            let display_currency = self.account_view_currency.as_ref()
                .map(|s| s.as_str())
                .unwrap_or(&a.currency);
            let rate = self.get_exchange_rate(&a.currency, display_currency);
            
            let view_indicator = if self.account_view_currency.is_some() {
                format!(" → {}", display_currency)
            } else {
                String::new()
            };
            
            let mut lines = vec![
                Line::from(vec![Span::styled(
                    format!("Account: {} [{}]{}", a.name, a.currency, view_indicator),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::Gray)),
                    Span::raw(format!("{} | ", a.account_type)),
                    Span::styled("Bank: ", Style::default().fg(Color::Gray)),
                    Span::raw(a.bank_name.as_deref().unwrap_or("N/A")),
                ]),
                Line::from(vec![
                    Span::styled("Balance: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.2} {}", a.current_balance * rate, display_currency),
                        if a.current_balance >= 0.0 { Style::default().fg(Color::Green) }
                        else { Style::default().fg(Color::Red) }),
                    if self.account_view_currency.is_some() {
                        Span::styled(format!(" ({:.2} {})", a.current_balance, a.currency), 
                            Style::default().fg(Color::DarkGray))
                    } else {
                        Span::raw("")
                    },
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    format!("─── Transactions ({}) ───", account_txns.len()),
                    Style::default().fg(Color::Yellow),
                )]),
            ];
            
            // Show transactions (limit to 15)
            for t in account_txns.iter().take(15) {
                let sign = if t.transaction_type == "income" { "+" } else { "-" };
                let color = if t.transaction_type == "income" { Color::Green } else { Color::Red };
                let desc = t.description.as_deref().unwrap_or("No description");
                let display_amount = t.amount * rate;
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{:.2} ", sign, display_amount), Style::default().fg(color)),
                    Span::styled(format!("{} ", display_currency), Style::default().fg(Color::Gray)),
                    if self.account_view_currency.is_some() {
                        Span::styled(format!("({}{:.2} {}) ", sign, t.amount, a.currency), 
                            Style::default().fg(Color::DarkGray))
                    } else {
                        Span::raw("")
                    },
                    Span::raw(if desc.len() > 25 { format!("{}...", &desc[..22]) } else { desc.to_string() }),
                ]));
            }
            if account_txns.len() > 15 {
                lines.push(Line::from(vec![Span::styled(
                    format!("  ... and {} more", account_txns.len() - 15),
                    Style::default().fg(Color::DarkGray),
                )]));
            }
            
            lines.push(Line::from(""));
            let view_hint = if self.account_view_currency.is_some() {
                format!("v: Change view (current: {})", display_currency)
            } else {
                "v: View in different currency".to_string()
            };
            lines.push(Line::from(vec![Span::styled(
                format!("Esc: Back | {}", view_hint),
                Style::default().fg(Color::Gray),
            )]));
            lines
        } else if self.current_screen == Screen::ExchangeRates
            && self.selected_index < self.exchange_rates.len()
        {
            let r = &self.exchange_rates[self.selected_index];
            vec![
                Line::from(vec![Span::styled(
                    "Exchange Rate Details",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("ID: {}", r.id)),
                Line::from(format!("From Currency: {}", r.from_currency)),
                Line::from(format!("To Currency: {}", r.to_currency)),
                Line::from(format!("Rate: {:.6}", r.rate)),
                Line::from(format!("Source: {}", r.source)),
                Line::from(format!(
                    "Rate Date: {}",
                    r.rate_date.format("%Y-%m-%d %H:%M:%S")
                )),
                Line::from(format!(
                    "Created: {}",
                    r.created_at.format("%Y-%m-%d %H:%M:%S")
                )),
                Line::from(format!(
                    "Updated: {}",
                    r.updated_at.format("%Y-%m-%d %H:%M:%S")
                )),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press Esc to go back",
                    Style::default().fg(Color::Gray),
                )]),
            ]
        } else if self.current_screen == Screen::RecurringTransactions
            && self.selected_index < self.recurring_transactions.len()
        {
            let r = &self.recurring_transactions[self.selected_index];
            vec![
                Line::from(vec![Span::styled(
                    "Recurring Transaction Details",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("ID: {}", r.id)),
                Line::from(format!("Account ID: {}", r.account_id)),
                Line::from(format!("Category ID: {}", r.category_id.map(|c| c.to_string()).unwrap_or("None".to_string()))),
                Line::from(format!("Amount: ${:.2}", r.amount)),
                Line::from(format!("Type: {}", r.transaction_type)),
                Line::from(format!("Frequency: {}", r.frequency)),
                Line::from(format!(
                    "Description: {}",
                    r.description.as_deref().unwrap_or("No description")
                )),
                Line::from(format!("Status: {}", if r.is_active { "Active" } else { "Paused" })),
                Line::from(format!(
                    "Start Date: {}",
                    r.start_date.format("%Y-%m-%d")
                )),
                Line::from(format!(
                    "End Date: {}",
                    r.end_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or("None".to_string())
                )),
                Line::from(format!(
                    "Next Occurrence: {}",
                    r.next_occurrence.format("%Y-%m-%d")
                )),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press Esc to go back",
                    Style::default().fg(Color::Gray),
                )]),
            ]
        } else {
            vec![Line::from("No details available")]
        };

        let details = Paragraph::new(details_text)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .alignment(Alignment::Left);
        frame.render_widget(details, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let help_text = if !self.status_message.is_empty() {
            Paragraph::new(self.status_message.as_str())
                .style(Style::default().fg(Color::Green))
                .alignment(Alignment::Center)
        } else {
            match self.mode {
                Mode::Normal => {
                    if self.current_screen == Screen::UserSelect {
                        Paragraph::new("↑↓: Select | Enter: Login | a: Add | d: Delete | q: Quit")
                    } else if self.current_screen == Screen::Transactions {
                        Paragraph::new("↑↓/[]: Scroll | g/G: Top/Bottom | a: Add | f: Filter | v: View in Currency | d: Delete | Enter: Details | q: Quit")
                    } else if self.current_screen == Screen::ExchangeRates {
                        Paragraph::new("↑↓/[]: Scroll | g/G: Top/Bottom | a: Add | c: Convert | d: Delete | Enter: Details | r: Refresh | q: Quit")
                    } else if self.current_screen == Screen::RecurringTransactions {
                        Paragraph::new("↑↓/[]: Scroll | g/G: Top/Bottom | a: Add | p: Process | t: Toggle | d: Delete | r: Refresh | q: Quit")
                    } else if self.current_screen == Screen::Export {
                        Paragraph::new("←/→ or 1-8: Tabs | e: Export data | r: Refresh | u: Switch user | q: Quit")
                    } else if self.current_screen == Screen::Dashboard {
                        Paragraph::new("←/→ or 1-8: Tabs | ↑/↓: Scroll | r: Refresh | u: Switch user | q: Quit")
                    } else if self.current_screen == Screen::Accounts {
                        Paragraph::new("↑↓/[]: Scroll | g/G: Top/Bottom | a: Add | d: Delete | Enter: Details | r: Refresh | q: Quit")
                    } else if self.current_screen == Screen::Categories {
                        Paragraph::new("↑↓/[]: Scroll | g/G: Top/Bottom | a: Add | d: Delete | r: Refresh | q: Quit")
                    } else if self.current_screen == Screen::Reports {
                        Paragraph::new("←/→ or 1-8: Tabs | r: Refresh | u: Switch user | q: Quit")
                    } else {
                        Paragraph::new("←/→ or 1-8: Tabs | ↑/↓: Select | r: Refresh | u: User | q: Quit")
                    }
                }
                Mode::AddTransaction => Paragraph::new(
                    "Tab: Next field | Enter: Submit | Esc: Cancel | (Tab cycles through fields)"
                ),
                Mode::AddExchangeRate => Paragraph::new(
                    "Tab: Next field | Enter: Submit | Esc: Cancel | (Tab cycles through fields)"
                ),
                Mode::AddRecurringTransaction => Paragraph::new(
                    "Tab: Next field | Enter: Submit | Esc: Cancel | (Tab cycles through fields)"
                ),
                Mode::AddAccount => Paragraph::new(
                    "Tab: Next field | Enter: Submit | Esc: Cancel | (Tab cycles through fields)"
                ),
                Mode::AddCategory => Paragraph::new(
                    "Tab: Next field | Type to input | Enter: Submit | Esc: Cancel"
                ),
                Mode::AddUser => Paragraph::new(
                    "Tab: Next field | Type to input | Enter: Create User | Esc: Cancel"
                ),
                Mode::ConvertCurrency => Paragraph::new(
                    "Tab: Next field | Enter: Convert | Esc: Cancel | (Tab cycles through fields)"
                ),
                Mode::DeleteConfirm => Paragraph::new(
                    "y: Confirm delete | n: Cancel"
                ),
                Mode::ViewDetails => Paragraph::new(
                    "Esc: Go back | v: View in different currency"
                ),
                Mode::ExportData => Paragraph::new(
                    "1-4: Select export format | Esc: Cancel"
                ),
                Mode::SelectCurrencyFilter => Paragraph::new(
                    "0: All Currencies | 1-9: Select currency | Esc: Cancel"
                ),
                Mode::SelectViewCurrency => Paragraph::new(
                    "↑↓: Scroll | []: Jump 10 | Enter: Select | Esc: Cancel"
                ),
            }
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
        };

        let footer = help_text.block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, area);
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.current_screen == Screen::UserSelect {
                        self.handle_user_select_mode(key.code).await;
                    } else {
                        match self.mode {
                            Mode::Normal => self.handle_normal_mode(key.code).await,
                            Mode::AddTransaction => {
                                self.handle_add_transaction_mode(key.code).await
                            }
                            Mode::AddExchangeRate => {
                                self.handle_add_exchange_rate_mode(key.code).await
                            }
                            Mode::AddRecurringTransaction => {
                                self.handle_add_recurring_mode(key.code).await
                            }
                            Mode::AddAccount => {
                                self.handle_add_account_mode(key.code).await
                            }
                            Mode::AddCategory => {
                                self.handle_add_category_mode(key.code).await
                            }
                            Mode::AddUser => {
                                self.handle_add_user_mode(key.code).await
                            }
                            Mode::ConvertCurrency => {
                                self.handle_convert_currency_mode(key.code).await
                            }
                            Mode::DeleteConfirm => self.handle_delete_mode(key.code).await,
                            Mode::ViewDetails => self.handle_details_mode(key.code),
                            Mode::ExportData => self.handle_export_mode(key.code).await,
                            Mode::SelectCurrencyFilter => self.handle_currency_filter_mode(key.code),
                            Mode::SelectViewCurrency => self.handle_view_currency_mode(key.code),
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_user_select_mode(&mut self, code: KeyCode) {
        if self.mode == Mode::AddUser {
            self.handle_add_user_mode(code).await;
            return;
        }
        
        if self.mode == Mode::DeleteConfirm {
            self.handle_delete_user_mode(code).await;
            return;
        }
        
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('a') => {
                self.mode = Mode::AddUser;
                self.clear_user_form();
            }
            KeyCode::Char('d') => {
                if !self.users.is_empty() {
                    self.mode = Mode::DeleteConfirm;
                }
            }
            KeyCode::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            KeyCode::Down => {
                self.selected_index =
                    (self.selected_index + 1).min(self.users.len().saturating_sub(1));
            }
            KeyCode::Enter => {
                if self.selected_index < self.users.len() {
                    self.current_user_id = Some(self.users[self.selected_index].id);
                    self.current_screen = Screen::Dashboard;
                    self.selected_tab = 0;
                    self.selected_index = 0;
                    self.load_data().await;
                    self.status_message =
                        format!("Logged in as {}", self.users[self.selected_index].username);
                }
            }
            _ => {}
        }
    }

    async fn handle_normal_mode(&mut self, code: KeyCode) {
        self.status_message.clear();
        self.export_message.clear();

        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('u') => {
                // Switch user
                self.current_screen = Screen::UserSelect;
                self.selected_index = 0;
                self.current_user_id = None;
                self.accounts.clear();
                self.transactions.clear();
                self.categories.clear();
                self.exchange_rates.clear();
                self.recurring_transactions.clear();
                self.category_spending.clear();
            }
            KeyCode::Char('r') => {
                self.load_data().await;
                self.status_message = "Data refreshed!".to_string();
            }
            KeyCode::Char('a') => {
                if self.current_screen == Screen::Transactions {
                    self.mode = Mode::AddTransaction;
                    self.clear_transaction_form();
                } else if self.current_screen == Screen::Accounts {
                    self.mode = Mode::AddAccount;
                    self.clear_account_form();
                } else if self.current_screen == Screen::Categories {
                    self.mode = Mode::AddCategory;
                    self.clear_category_form();
                } else if self.current_screen == Screen::ExchangeRates {
                    self.mode = Mode::AddExchangeRate;
                    self.clear_exchange_rate_form();
                } else if self.current_screen == Screen::RecurringTransactions {
                    self.mode = Mode::AddRecurringTransaction;
                    self.clear_recurring_form();
                }
            }
            KeyCode::Char('c') => {
                if self.current_screen == Screen::ExchangeRates {
                    self.mode = Mode::ConvertCurrency;
                    self.clear_conversion_form();
                }
            }
            KeyCode::Char('d') => {
                if self.current_screen == Screen::Transactions && !self.transactions.is_empty() {
                    self.mode = Mode::DeleteConfirm;
                } else if self.current_screen == Screen::Accounts && !self.accounts.is_empty() {
                    self.mode = Mode::DeleteConfirm;
                } else if self.current_screen == Screen::Categories && !self.categories.is_empty() {
                    self.mode = Mode::DeleteConfirm;
                } else if self.current_screen == Screen::ExchangeRates && !self.exchange_rates.is_empty() {
                    self.mode = Mode::DeleteConfirm;
                } else if self.current_screen == Screen::RecurringTransactions && !self.recurring_transactions.is_empty() {
                    self.mode = Mode::DeleteConfirm;
                }
            }
            KeyCode::Char('e') => {
                if self.current_screen == Screen::Export {
                    self.mode = Mode::ExportData;
                }
            }
            KeyCode::Char('f') => {
                // Filter by currency on Transactions screen
                if self.current_screen == Screen::Transactions {
                    self.mode = Mode::SelectCurrencyFilter;
                }
            }
            KeyCode::Char('v') => {
                // View in currency on Transactions screen
                if self.current_screen == Screen::Transactions {
                    self.mode = Mode::SelectViewCurrency;
                }
            }
            KeyCode::Char('p') => {
                // Process recurring transactions
                if self.current_screen == Screen::RecurringTransactions {
                    self.process_recurring_transactions().await;
                }
            }
            KeyCode::Char('t') => {
                // Toggle active status for recurring transactions
                if self.current_screen == Screen::RecurringTransactions 
                    && self.selected_index < self.recurring_transactions.len() 
                {
                    self.toggle_recurring_active().await;
                }
            }
            KeyCode::Enter => {
                self.mode = Mode::ViewDetails;
            }
            KeyCode::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            KeyCode::Down => {
                let max = self.get_current_list_len().saturating_sub(1);
                self.selected_index = (self.selected_index + 1).min(max);
            }
            KeyCode::PageUp | KeyCode::Char('[') => {
                // Jump up 10 items (PageUp or '[' for Mac compatibility)
                self.selected_index = self.selected_index.saturating_sub(10);
            }
            KeyCode::PageDown | KeyCode::Char(']') => {
                // Jump down 10 items (PageDown or ']' for Mac compatibility)
                let max = self.get_current_list_len().saturating_sub(1);
                self.selected_index = (self.selected_index + 10).min(max);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                // Jump to first item (Home or 'g' for Mac compatibility)
                self.selected_index = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                // Jump to last item (End or 'G' for Mac compatibility)
                let max = self.get_current_list_len().saturating_sub(1);
                self.selected_index = max;
            }
            KeyCode::Left => {
                self.selected_tab = self.selected_tab.saturating_sub(1);
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Right => {
                self.selected_tab = (self.selected_tab + 1).min(7);
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Char('1') => {
                self.selected_tab = 0;
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Char('2') => {
                self.selected_tab = 1;
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Char('3') => {
                self.selected_tab = 2;
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Char('4') => {
                self.selected_tab = 3;
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Char('5') => {
                self.selected_tab = 4;
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Char('6') => {
                self.selected_tab = 5;
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Char('7') => {
                self.selected_tab = 6;
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Char('8') => {
                self.selected_tab = 7;
                self.update_screen();
                self.selected_index = 0;
            }
            _ => {}
        }
    }

    async fn handle_add_transaction_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Tab => {
                self.form_field_index = (self.form_field_index + 1) % 5;
            }
            KeyCode::BackTab => {
                self.form_field_index = if self.form_field_index == 0 {
                    4
                } else {
                    self.form_field_index - 1
                };
            }
            KeyCode::Char(c) => match self.form_field_index {
                0 => self.form_account_id.push(c),
                1 => self.form_amount.push(c),
                2 => self.form_type.push(c),
                3 => self.form_description.push(c),
                4 => self.form_category_id.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.form_field_index {
                0 => {
                    self.form_account_id.pop();
                }
                1 => {
                    self.form_amount.pop();
                }
                2 => {
                    self.form_type.pop();
                }
                3 => {
                    self.form_description.pop();
                }
                4 => {
                    self.form_category_id.pop();
                }
                _ => {}
            },
            KeyCode::Enter => {
                self.submit_transaction().await;
            }
            _ => {}
        }
    }
    async fn handle_add_exchange_rate_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Tab => {
                self.form_field_index = (self.form_field_index + 1) % 4;
            }
            KeyCode::BackTab => {
                self.form_field_index = if self.form_field_index == 0 {
                    3
                } else {
                    self.form_field_index - 1
                };
            }
            KeyCode::Char(c) => match self.form_field_index {
                0 => self.form_from_currency.push(c),
                1 => self.form_to_currency.push(c),
                2 => self.form_rate.push(c),
                3 => self.form_source.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.form_field_index {
                0 => {
                    self.form_from_currency.pop();
                }
                1 => {
                    self.form_to_currency.pop();
                }
                2 => {
                    self.form_rate.pop();
                }
                3 => {
                    self.form_source.pop();
                }
                _ => {}
            },
            KeyCode::Enter => {
                self.submit_exchange_rate().await;
            }
            _ => {}
        }
    }

    async fn handle_convert_currency_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Tab => {
                self.form_field_index = (self.form_field_index + 1) % 3;
            }
            KeyCode::BackTab => {
                self.form_field_index = if self.form_field_index == 0 {
                    2
                } else {
                    self.form_field_index - 1
                };
            }
            KeyCode::Char(c) => match self.form_field_index {
                0 => self.form_convert_from.push(c),
                1 => self.form_convert_to.push(c),
                2 => self.form_convert_amount.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.form_field_index {
                0 => {
                    self.form_convert_from.pop();
                }
                1 => {
                    self.form_convert_to.pop();
                }
                2 => {
                    self.form_convert_amount.pop();
                }
                _ => {}
            },
            KeyCode::Enter => {
                self.perform_currency_conversion().await;
            }
            _ => {}
        }
    }

    async fn submit_transaction(&mut self) {
        let account_id = self.form_account_id.parse::<i64>();
        let amount = self.form_amount.parse::<f64>();
        let category_id = self.form_category_id.parse::<i64>();

        if account_id.is_err() || amount.is_err() || category_id.is_err() {
            self.status_message =
                "Error: Invalid input! Check account ID, amount, and category ID.".to_string();
            self.mode = Mode::Normal;
            return;
        }

        let account_id = account_id.unwrap();
        let amount = amount.unwrap();
        let category_id = category_id.unwrap();
        
        // Normalize transaction type: accept i/e shortcuts
        let txn_type = match self.form_type.to_lowercase().as_str() {
            "i" | "income" => "income",
            "e" | "expense" | "" => "expense",  // default to expense
            _ => {
                self.status_message = "Error: Type must be 'income' (i) or 'expense' (e)".to_string();
                self.mode = Mode::Normal;
                return;
            }
        };
        
        let description = if self.form_description.is_empty() {
            None
        } else {
            Some(self.form_description.clone())
        };

        let result = sqlx::query(
            "INSERT INTO transactions (account_id, amount, transaction_type, description, transaction_date) VALUES (?, ?, ?, ?, datetime('now'))"
        )
        .bind(account_id)
        .bind(amount)
        .bind(txn_type)
        .bind(&description)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => {
                let transaction_id = res.last_insert_rowid();

                let _ = sqlx::query(
                    "INSERT INTO transaction_categories (transaction_id, category_id, amount) VALUES (?, ?, ?)"
                )
                .bind(transaction_id)
                .bind(category_id)
                .bind(amount)
                .execute(&self.pool)
                .await;

                let balance_change = if txn_type == "income" {
                    amount
                } else {
                    -amount.abs()
                };

                let _ = sqlx::query(
                    "UPDATE accounts SET current_balance = current_balance + ? WHERE id = ?",
                )
                .bind(balance_change)
                .bind(account_id)
                .execute(&self.pool)
                .await;

                self.status_message =
                    format!("Transaction added successfully! ID: {}", transaction_id);
                self.load_data().await;
            }
            Err(e) => {
                self.status_message = format!("Error adding transaction: {}", e);
            }
        }

        self.mode = Mode::Normal;
    }

    async fn submit_exchange_rate(&mut self) {
        let rate = self.form_rate.parse::<f64>();

        if self.form_from_currency.is_empty() || self.form_to_currency.is_empty() || rate.is_err() {
            self.status_message = "Error: Invalid input! Check currencies and rate.".to_string();
            self.mode = Mode::Normal;
            return;
        }

        let rate = rate.unwrap();
        let from_currency = &self.form_from_currency;
        let to_currency = &self.form_to_currency;
        let source = &self.form_source;

        let result = sqlx::query(
            "INSERT INTO exchange_rates (from_currency, to_currency, rate, rate_date, source) VALUES (?, ?, ?, datetime('now'), ?)"
        )
        .bind(from_currency)
        .bind(to_currency)
        .bind(rate)
        .bind(source)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => {
                let rate_id = res.last_insert_rowid();
                self.status_message = format!("Exchange rate added successfully! ID: {}", rate_id);
                self.load_data().await;
            }
            Err(e) => {
                self.status_message = format!("Error adding exchange rate: {}", e);
            }
        }

        self.mode = Mode::Normal;
    }

    async fn perform_currency_conversion(&mut self) {
        let amount = self.form_convert_amount.parse::<f64>();

        if self.form_convert_from.is_empty() || self.form_convert_to.is_empty() || amount.is_err() {
            self.form_converted_result = "Error: Invalid input!".to_string();
            return;
        }

        let amount = amount.unwrap();
        let from = &self.form_convert_from;
        let to = &self.form_convert_to;

        // Query the latest exchange rate
        let rate: Result<Option<f64>, _> = sqlx::query_scalar(
            "SELECT rate FROM exchange_rates 
             WHERE from_currency = ? AND to_currency LIKE ?
             ORDER BY rate_date DESC 
             LIMIT 1",
        )
        .bind(from)
        .bind(format!("%({})%", to))
        .fetch_optional(&self.pool)
        .await;

        match rate {
            Ok(Some(rate)) => {
                let converted = amount * rate;
                self.form_converted_result = format!(
                    "{:.2} {} = {:.2} {} (rate: {:.6})",
                    amount, from, converted, to, rate
                );
                self.status_message = "Conversion successful!".to_string();
            }
            Ok(None) => {
                self.form_converted_result = format!("No rate found from {} to {}", from, to);
            }
            Err(e) => {
                self.form_converted_result = format!("Database error: {}", e);
            }
        }
    }

    async fn handle_delete_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if self.current_screen == Screen::Transactions
                    && self.selected_index < self.transactions.len()
                {
                    let transaction = &self.transactions[self.selected_index];
                    let transaction_id = transaction.id;
                    let account_id = transaction.account_id;
                    let amount = transaction.amount;
                    let transaction_type = transaction.transaction_type.clone();

                    // First update account balance (reverse the transaction effect)
                    let balance_adjustment = if transaction_type == "income" {
                        -amount  // Subtract income that was added
                    } else {
                        amount.abs()  // Add back expense that was subtracted
                    };

                    let balance_result = sqlx::query(
                        "UPDATE accounts SET current_balance = current_balance + ? WHERE id = ?"
                    )
                        .bind(balance_adjustment)
                        .bind(account_id)
                        .execute(&self.pool)
                        .await;

                    if let Err(e) = balance_result {
                        self.status_message = format!("Error updating account balance: {}", e);
                        self.mode = Mode::Normal;
                        return;
                    }

                    // Then delete the transaction
                    let result = sqlx::query("DELETE FROM transactions WHERE id = ?")
                        .bind(transaction_id)
                        .execute(&self.pool)
                        .await;

                    match result {
                        Ok(_) => {
                            self.status_message =
                                format!("Transaction {} deleted, balance updated!", transaction_id);
                            self.load_data().await;
                            self.selected_index = 0;
                        }
                        Err(e) => {
                            self.status_message = format!("Error deleting transaction: {}", e);
                        }
                    }
                } else if self.current_screen == Screen::Accounts
                    && self.selected_index < self.accounts.len()
                {
                    let account = &self.accounts[self.selected_index];
                    let account_id = account.id;
                    let account_name = account.name.clone();

                    // First delete all transaction_categories for this account's transactions
                    let _ = sqlx::query(
                        "DELETE FROM transaction_categories WHERE transaction_id IN (SELECT id FROM transactions WHERE account_id = ?)"
                    )
                        .bind(account_id)
                        .execute(&self.pool)
                        .await;

                    // Then delete all transactions for this account
                    let txn_result = sqlx::query("DELETE FROM transactions WHERE account_id = ?")
                        .bind(account_id)
                        .execute(&self.pool)
                        .await;

                    let txn_deleted = txn_result.map(|r| r.rows_affected()).unwrap_or(0);

                    // Finally delete the account
                    let result = sqlx::query("DELETE FROM accounts WHERE id = ?")
                        .bind(account_id)
                        .execute(&self.pool)
                        .await;

                    match result {
                        Ok(_) => {
                            if txn_deleted > 0 {
                                self.status_message = format!("Account '{}' and {} transactions deleted!", account_name, txn_deleted);
                            } else {
                                self.status_message = format!("Account '{}' deleted!", account_name);
                            }
                            self.load_data().await;
                            self.selected_index = 0;
                        }
                        Err(e) => {
                            self.status_message = format!("Error deleting account: {}", e);
                        }
                    }
                    self.mode = Mode::Normal;
                } else if self.current_screen == Screen::Categories
                    && self.selected_index < self.categories.len()
                {
                    let category = &self.categories[self.selected_index];
                    let category_id = category.id;
                    let category_name = category.name.clone();

                    // Check if category is linked to transactions
                    let link_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction_categories WHERE category_id = ?")
                        .bind(category_id)
                        .fetch_one(&self.pool)
                        .await
                        .unwrap_or((0,));

                    if link_count.0 > 0 {
                        self.status_message = format!("Cannot delete '{}': used by {} transactions.", category_name, link_count.0);
                        self.mode = Mode::Normal;
                        return;
                    }

                    let result = sqlx::query("DELETE FROM categories WHERE id = ?")
                        .bind(category_id)
                        .execute(&self.pool)
                        .await;

                    match result {
                        Ok(_) => {
                            self.status_message = format!("Category '{}' deleted!", category_name);
                            self.load_data().await;
                            self.selected_index = 0;
                        }
                        Err(e) => {
                            self.status_message = format!("Error deleting category: {}", e);
                        }
                    }
                    self.mode = Mode::Normal;
                } else if self.current_screen == Screen::ExchangeRates
                    && self.selected_index < self.exchange_rates.len()
                {
                    let rate_id = self.exchange_rates[self.selected_index].id;

                    let result = sqlx::query("DELETE FROM exchange_rates WHERE id = ?")
                        .bind(rate_id)
                        .execute(&self.pool)
                        .await;

                    match result {
                        Ok(_) => {
                            self.status_message =
                                format!("Exchange rate {} deleted successfully!", rate_id);
                            self.load_data().await;
                            self.selected_index = 0;
                        }
                        Err(e) => {
                            self.status_message = format!("Error deleting exchange rate: {}", e);
                        }
                    }
                } else if self.current_screen == Screen::RecurringTransactions
                    && self.selected_index < self.recurring_transactions.len()
                {
                    let recurring_id = self.recurring_transactions[self.selected_index].id;

                    let result = sqlx::query("DELETE FROM recurring_transactions WHERE id = ?")
                        .bind(recurring_id)
                        .execute(&self.pool)
                        .await;

                    match result {
                        Ok(_) => {
                            self.status_message =
                                format!("Recurring transaction {} deleted successfully!", recurring_id);
                            self.load_data().await;
                            self.selected_index = 0;
                        }
                        Err(e) => {
                            self.status_message = format!("Error deleting recurring transaction: {}", e);
                        }
                    }
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    async fn handle_add_recurring_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Tab => {
                self.form_field_index = (self.form_field_index + 1) % 6;
            }
            KeyCode::BackTab => {
                self.form_field_index = if self.form_field_index == 0 {
                    5
                } else {
                    self.form_field_index - 1
                };
            }
            KeyCode::Char(c) => match self.form_field_index {
                0 => self.form_account_id.push(c),
                1 => self.form_amount.push(c),
                2 => self.form_type.push(c),
                3 => self.form_description.push(c),
                4 => self.form_category_id.push(c),
                5 => self.form_recurring_frequency.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.form_field_index {
                0 => { self.form_account_id.pop(); }
                1 => { self.form_amount.pop(); }
                2 => { self.form_type.pop(); }
                3 => { self.form_description.pop(); }
                4 => { self.form_category_id.pop(); }
                5 => { self.form_recurring_frequency.pop(); }
                _ => {}
            },
            KeyCode::Enter => {
                self.submit_recurring_transaction().await;
            }
            _ => {}
        }
    }

    async fn submit_recurring_transaction(&mut self) {
        let account_id = self.form_account_id.parse::<i64>();
        let amount = self.form_amount.parse::<f64>();
        let category_id = self.form_category_id.parse::<i64>().ok();

        if account_id.is_err() || amount.is_err() {
            self.status_message = "Error: Invalid input! Check account ID and amount.".to_string();
            self.mode = Mode::Normal;
            return;
        }

        let account_id = account_id.unwrap();
        let amount = amount.unwrap();
        
        // Normalize transaction type: accept i/e shortcuts
        let txn_type = match self.form_type.to_lowercase().as_str() {
            "i" | "income" => "income",
            "e" | "expense" | "" => "expense",
            _ => {
                self.status_message = "Error: Type must be 'income' (i) or 'expense' (e)".to_string();
                self.mode = Mode::Normal;
                return;
            }
        };
        
        let description = if self.form_description.is_empty() {
            None
        } else {
            Some(self.form_description.clone())
        };
        
        // Normalize frequency: accept shortcuts d/w/m/y
        let frequency = match self.form_recurring_frequency.to_lowercase().as_str() {
            "d" | "daily" => "daily",
            "w" | "weekly" => "weekly",
            "m" | "monthly" | "" => "monthly",
            "y" | "yearly" => "yearly",
            _ => {
                self.status_message = "Error: Frequency must be daily(d)/weekly(w)/monthly(m)/yearly(y)".to_string();
                self.mode = Mode::Normal;
                return;
            }
        };

        let now = chrono::Utc::now();
        let result = sqlx::query(
            "INSERT INTO recurring_transactions 
             (account_id, category_id, amount, transaction_type, description, frequency, start_date, next_occurrence, is_active) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)"
        )
        .bind(account_id)
        .bind(category_id)
        .bind(amount)
        .bind(txn_type)
        .bind(&description)
        .bind(frequency)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => {
                let recurring_id = res.last_insert_rowid();
                self.status_message = format!("Recurring transaction added successfully! ID: {}", recurring_id);
                self.load_data().await;
            }
            Err(e) => {
                self.status_message = format!("Error adding recurring transaction: {}", e);
            }
        }

        self.mode = Mode::Normal;
    }

    async fn handle_export_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('1') => {
                self.export_transactions_csv().await;
                self.mode = Mode::Normal;
            }
            KeyCode::Char('2') => {
                self.export_transactions_json().await;
                self.mode = Mode::Normal;
            }
            KeyCode::Char('3') => {
                self.export_accounts_csv().await;
                self.mode = Mode::Normal;
            }
            KeyCode::Char('4') => {
                self.export_full_summary().await;
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    async fn export_transactions_csv(&mut self) {
        let mut csv = String::from("id,account_id,amount,type,description,date\n");

        for t in &self.transactions {
            csv.push_str(&format!(
                "{},{},{:.2},{},\"{}\",{}\n",
                t.id,
                t.account_id,
                t.amount,
                t.transaction_type,
                t.description.as_deref().unwrap_or("").replace("\"", "\"\""),
                t.transaction_date.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        match std::fs::write("transactions_export.csv", &csv) {
            Ok(_) => {
                self.export_message = format!("Exported {} transactions to transactions_export.csv", self.transactions.len());
                self.status_message = self.export_message.clone();
            }
            Err(e) => {
                self.export_message = format!("Error exporting: {}", e);
                self.status_message = self.export_message.clone();
            }
        }
    }

    async fn export_transactions_json(&mut self) {
        match serde_json::to_string_pretty(&self.transactions) {
            Ok(json) => {
                match std::fs::write("transactions_export.json", &json) {
                    Ok(_) => {
                        self.export_message = format!("Exported {} transactions to transactions_export.json", self.transactions.len());
                        self.status_message = self.export_message.clone();
                    }
                    Err(e) => {
                        self.export_message = format!("Error exporting: {}", e);
                        self.status_message = self.export_message.clone();
                    }
                }
            }
            Err(e) => {
                self.export_message = format!("Error serializing: {}", e);
                self.status_message = self.export_message.clone();
            }
        }
    }

    async fn export_accounts_csv(&mut self) {
        let mut csv = String::from("id,user_id,name,type,bank_name,currency,initial_balance,current_balance\n");

        for a in &self.accounts {
            csv.push_str(&format!(
                "{},{},\"{}\",{},\"{}\",{},{:.2},{:.2}\n",
                a.id,
                a.user_id,
                a.name.replace("\"", "\"\""),
                a.account_type,
                a.bank_name.as_deref().unwrap_or("").replace("\"", "\"\""),
                a.currency,
                a.initial_balance,
                a.current_balance
            ));
        }

        match std::fs::write("accounts_export.csv", &csv) {
            Ok(_) => {
                self.export_message = format!("Exported {} accounts to accounts_export.csv", self.accounts.len());
                self.status_message = self.export_message.clone();
            }
            Err(e) => {
                self.export_message = format!("Error exporting: {}", e);
                self.status_message = self.export_message.clone();
            }
        }
    }

    async fn export_full_summary(&mut self) {
        use serde_json::json;

        let summary = json!({
            "export_date": chrono::Utc::now().to_rfc3339(),
            "user_id": self.current_user_id,
            "accounts": self.accounts,
            "transactions": self.transactions,
            "categories": self.categories,
            "recurring_transactions": self.recurring_transactions,
        });

        match serde_json::to_string_pretty(&summary) {
            Ok(json) => {
                match std::fs::write("financial_summary.json", &json) {
                    Ok(_) => {
                        self.export_message = "Exported full financial summary to financial_summary.json".to_string();
                        self.status_message = self.export_message.clone();
                    }
                    Err(e) => {
                        self.export_message = format!("Error exporting: {}", e);
                        self.status_message = self.export_message.clone();
                    }
                }
            }
            Err(e) => {
                self.export_message = format!("Error serializing: {}", e);
                self.status_message = self.export_message.clone();
            }
        }
    }

    async fn process_recurring_transactions(&mut self) {
        let now = chrono::Utc::now();
        let mut created_count = 0;

        // Get due recurring transactions
        let due: Vec<_> = self.recurring_transactions
            .iter()
            .filter(|r| r.is_active && r.next_occurrence <= now)
            .cloned()
            .collect();

        for recurring in due {
            // Create the transaction
            let result = sqlx::query(
                "INSERT INTO transactions (account_id, amount, transaction_type, description, transaction_date) 
                 VALUES (?, ?, ?, ?, ?)"
            )
            .bind(recurring.account_id)
            .bind(recurring.amount)
            .bind(&recurring.transaction_type)
            .bind(&recurring.description)
            .bind(recurring.next_occurrence)
            .execute(&self.pool)
            .await;

            if result.is_ok() {
                // Update account balance
                let balance_change = if recurring.transaction_type == "income" {
                    recurring.amount
                } else {
                    -recurring.amount.abs()
                };

                let _ = sqlx::query(
                    "UPDATE accounts SET current_balance = current_balance + ? WHERE id = ?"
                )
                .bind(balance_change)
                .bind(recurring.account_id)
                .execute(&self.pool)
                .await;

                // Calculate next occurrence
                let next = match recurring.frequency.as_str() {
                    "daily" => recurring.next_occurrence + chrono::Duration::days(1),
                    "weekly" => recurring.next_occurrence + chrono::Duration::weeks(1),
                    "monthly" => recurring.next_occurrence + chrono::Duration::days(30),
                    "yearly" => recurring.next_occurrence + chrono::Duration::days(365),
                    _ => recurring.next_occurrence + chrono::Duration::days(30),
                };

                let _ = sqlx::query(
                    "UPDATE recurring_transactions SET next_occurrence = ? WHERE id = ?"
                )
                .bind(next)
                .bind(recurring.id)
                .execute(&self.pool)
                .await;

                created_count += 1;
            }
        }

        self.load_data().await;
        self.status_message = format!("Processed {} recurring transactions - balances updated!", created_count);
    }

    async fn toggle_recurring_active(&mut self) {
        let recurring = &self.recurring_transactions[self.selected_index];
        let new_status = !recurring.is_active;

        let result = sqlx::query(
            "UPDATE recurring_transactions SET is_active = ? WHERE id = ?"
        )
        .bind(new_status)
        .bind(recurring.id)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                self.status_message = format!(
                    "Recurring transaction {} {}",
                    recurring.id,
                    if new_status { "activated" } else { "paused" }
                );
                self.load_data().await;
            }
            Err(e) => {
                self.status_message = format!("Error updating status: {}", e);
            }
        }
    }

    fn handle_details_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                // Clear account-specific view currency when exiting details
                self.account_view_currency = None;
                self.mode = Mode::Normal;
            }
            KeyCode::Char('v') => {
                // Open currency conversion selector when viewing account details
                if self.current_screen == Screen::Accounts {
                    self.mode = Mode::SelectViewCurrency;
                }
            }
            _ => {}
        }
    }

    fn handle_currency_filter_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('0') => {
                // Show all currencies
                self.currency_filter = None;
                self.selected_index = 0;
                self.status_message = "Showing all currencies".to_string();
                self.mode = Mode::Normal;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let num = c.to_digit(10).unwrap() as usize;
                if num > 0 && num <= self.filter_currencies.len() {
                    let currency = self.filter_currencies[num - 1].clone();
                    self.status_message = format!("Filtering by {}", currency);
                    self.currency_filter = Some(currency);
                    self.selected_index = 0;
                    self.mode = Mode::Normal;
                }
            }
            // Support letters a-z for currencies 10+
            KeyCode::Char(c) if c.is_ascii_lowercase() => {
                let idx = (c as u8 - b'a') as usize + 9; // 'a' = index 9, 'b' = index 10, etc.
                if idx < self.filter_currencies.len() {
                    let currency = self.filter_currencies[idx].clone();
                    self.status_message = format!("Filtering by {}", currency);
                    self.currency_filter = Some(currency);
                    self.selected_index = 0;
                    self.mode = Mode::Normal;
                }
            }
            _ => {}
        }
    }

    fn render_view_currency_dialog(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Make popup larger to show more currencies
        let popup_area = ratatui::layout::Rect {
            x: area.x + area.width / 6,
            y: area.y + 2,
            width: area.width * 2 / 3,
            height: area.height.saturating_sub(4),
        };

        frame.render_widget(ratatui::widgets::Clear, popup_area);

        // Determine if this is for account details or transactions
        let is_account_details = self.current_screen == Screen::Accounts;
        let current_selection = if is_account_details {
            &self.account_view_currency
        } else {
            &self.view_in_currency
        };

        // Calculate visible area (subtract borders and header/footer)
        let visible_height = popup_area.height.saturating_sub(6) as usize;
        let total_currencies = self.available_currencies.len();
        
        // Build currency list items
        let mut items: Vec<ListItem> = Vec::new();
        
        // First item: Original (no conversion)
        let is_original_active = current_selection.is_none();
        let is_original_highlighted = self.currency_scroll_offset == 0;
        let original_style = if is_original_highlighted {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled("► ", if is_original_highlighted { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
            Span::raw("Original (no conversion)"),
            if is_original_active { 
                Span::styled(" ✓ ACTIVE", Style::default().fg(Color::Green)) 
            } else { 
                Span::raw("") 
            },
        ])).style(original_style));

        // Show currencies
        for (i, curr) in self.available_currencies.iter().enumerate() {
            let is_active = current_selection.as_ref() == Some(curr);
            let is_highlighted = self.currency_scroll_offset == i + 1;
            let item_style = if is_highlighted {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            items.push(ListItem::new(Line::from(vec![
                Span::styled("► ", if is_highlighted { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
                Span::raw(curr.clone()),
                if is_active { 
                    Span::styled(" ✓ ACTIVE", Style::default().fg(Color::Green)) 
                } else { 
                    Span::raw("") 
                },
            ])).style(item_style));
        }

        // Show scroll indicator in title
        let scroll_info = if total_currencies > visible_height {
            format!(" [{}-{}/{}] ↑↓ to scroll", 
                self.currency_scroll_offset + 1,
                (self.currency_scroll_offset + visible_height).min(total_currencies + 1),
                total_currencies + 1)
        } else {
            format!(" [{} currencies]", total_currencies)
        };

        let title = if is_account_details {
            format!(" View Account In Currency{} ", scroll_info)
        } else {
            format!(" View Transactions In Currency{} ", scroll_info)
        };

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(title)
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            .highlight_style(Style::default().bg(Color::DarkGray));

        // Apply scroll offset
        let mut state = ListState::default();
        state.select(Some(self.currency_scroll_offset));
        frame.render_stateful_widget(list, popup_area, &mut state);
    }

    fn handle_view_currency_mode(&mut self, code: KeyCode) {
        // Determine if we're setting account-specific or global view currency
        let is_account_details = self.current_screen == Screen::Accounts;
        let max_scroll = self.available_currencies.len(); // index 0 = "Original", 1+ = currencies
        
        match code {
            KeyCode::Esc => {
                self.currency_scroll_offset = 0;
                self.mode = if is_account_details { Mode::ViewDetails } else { Mode::Normal };
            }
            // Scrolling support
            KeyCode::Up => {
                self.currency_scroll_offset = self.currency_scroll_offset.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.currency_scroll_offset < max_scroll {
                    self.currency_scroll_offset += 1;
                }
            }
            KeyCode::PageUp | KeyCode::Char('[') => {
                self.currency_scroll_offset = self.currency_scroll_offset.saturating_sub(10);
            }
            KeyCode::PageDown | KeyCode::Char(']') => {
                self.currency_scroll_offset = (self.currency_scroll_offset + 10).min(max_scroll);
            }
            KeyCode::Home => {
                self.currency_scroll_offset = 0;
            }
            KeyCode::End => {
                self.currency_scroll_offset = max_scroll;
            }
            // Enter to select the highlighted currency
            KeyCode::Enter => {
                let selected_idx = self.currency_scroll_offset;
                self.currency_scroll_offset = 0;
                
                if selected_idx == 0 {
                    // "Original (no conversion)" selected
                    if is_account_details {
                        self.account_view_currency = None;
                        self.status_message = "Account: showing original currency".to_string();
                        self.mode = Mode::ViewDetails;
                    } else {
                        self.view_in_currency = None;
                        self.status_message = "Showing original currencies".to_string();
                        self.mode = Mode::Normal;
                    }
                } else if selected_idx <= self.available_currencies.len() {
                    // Currency selected (index 1 = first currency)
                    let currency = self.available_currencies[selected_idx - 1].clone();
                    if is_account_details {
                        self.status_message = format!("Account: viewing in {}", currency);
                        self.account_view_currency = Some(currency);
                        self.mode = Mode::ViewDetails;
                    } else {
                        self.status_message = format!("Viewing all amounts in {}", currency);
                        self.view_in_currency = Some(currency);
                        self.mode = Mode::Normal;
                    }
                }
            }
            _ => {}
        }
    }

    /// Extract 3-letter currency code from strings like "Argentine Peso (ARS)" or "USD"
    fn extract_currency_code(currency: &str) -> String {
        // Check if it contains parentheses with a code like "(ARS)"
        if let Some(start) = currency.rfind('(') {
            if let Some(end) = currency.rfind(')') {
                if end > start {
                    let code = &currency[start + 1..end];
                    // Verify it looks like a currency code (2-4 uppercase letters)
                    if code.len() >= 2 && code.len() <= 4 && code.chars().all(|c| c.is_ascii_uppercase()) {
                        return code.to_string();
                    }
                }
            }
        }
        // Otherwise return the original string (it's probably already a code)
        currency.to_string()
    }

    /// Check if two currency strings match (handles both codes and full names)
    fn currencies_match(a: &str, b: &str) -> bool {
        let code_a = Self::extract_currency_code(a);
        let code_b = Self::extract_currency_code(b);
        code_a == code_b
    }

    fn get_exchange_rate(&self, from: &str, to: &str) -> f64 {
        let from_code = Self::extract_currency_code(from);
        let to_code = Self::extract_currency_code(to);
        
        if from_code == to_code {
            return 1.0;
        }
        
        // Try to find direct rate (with flexible matching)
        if let Some(rate) = self.exchange_rates.iter().find(|r| 
            Self::currencies_match(&r.from_currency, &from_code) && 
            Self::currencies_match(&r.to_currency, &to_code)
        ) {
            return rate.rate;
        }
        
        // Try reverse rate
        if let Some(rate) = self.exchange_rates.iter().find(|r| 
            Self::currencies_match(&r.from_currency, &to_code) && 
            Self::currencies_match(&r.to_currency, &from_code)
        ) {
            return 1.0 / rate.rate;
        }
        
        // Try triangulation via common intermediate currencies (USD, EUR, CAD, GBP)
        let intermediates = ["USD", "EUR", "CAD", "GBP"];
        
        for intermediate in intermediates {
            // Skip if intermediate is one of our currencies
            if from_code == intermediate || to_code == intermediate {
                continue;
            }
            
            // Find rate from source to intermediate
            let from_to_inter = self.exchange_rates.iter()
                .find(|r| Self::currencies_match(&r.from_currency, &from_code) && 
                          Self::currencies_match(&r.to_currency, intermediate))
                .map(|r| r.rate)
                .or_else(|| self.exchange_rates.iter()
                    .find(|r| Self::currencies_match(&r.from_currency, intermediate) && 
                              Self::currencies_match(&r.to_currency, &from_code))
                    .map(|r| 1.0 / r.rate));
            
            // Find rate from intermediate to target
            let inter_to_target = self.exchange_rates.iter()
                .find(|r| Self::currencies_match(&r.from_currency, intermediate) && 
                          Self::currencies_match(&r.to_currency, &to_code))
                .map(|r| r.rate)
                .or_else(|| self.exchange_rates.iter()
                    .find(|r| Self::currencies_match(&r.from_currency, &to_code) && 
                              Self::currencies_match(&r.to_currency, intermediate))
                    .map(|r| 1.0 / r.rate));
            
            // If both rates found, return the combined rate
            if let (Some(f), Some(t)) = (from_to_inter, inter_to_target) {
                return f * t;
            }
        }
        
        1.0 // Default to 1.0 if no rate found
    }

    fn update_screen(&mut self) {
        self.current_screen = match self.selected_tab {
            0 => Screen::Dashboard,
            1 => Screen::Accounts,
            2 => Screen::Transactions,
            3 => Screen::Categories,
            4 => Screen::RecurringTransactions,
            5 => Screen::ExchangeRates,
            6 => Screen::Reports,
            7 => Screen::Export,
            _ => Screen::Dashboard,
        };
    }

    fn get_current_list_len(&self) -> usize {
        match self.current_screen {
            Screen::Dashboard => self.transactions.len().min(10),
            Screen::Accounts => self.accounts.len(),
            Screen::Transactions => {
                // Account for currency filter
                if let Some(ref currency) = self.currency_filter {
                    self.transactions
                        .iter()
                        .filter(|t| {
                            self.accounts
                                .iter()
                                .find(|a| a.id == t.account_id)
                                .map(|a| &a.currency == currency)
                                .unwrap_or(false)
                        })
                        .count()
                } else {
                    self.transactions.len()
                }
            }
            Screen::Categories => self.categories.len(),
            Screen::RecurringTransactions => self.recurring_transactions.len(),
            Screen::ExchangeRates => self.exchange_rates.len(),
            _ => 0,
        }
    }

    fn clear_transaction_form(&mut self) {
        self.form_account_id.clear();
        self.form_amount.clear();
        self.form_type = String::from("expense");
        self.form_description.clear();
        self.form_category_id.clear();
        self.form_field_index = 0;
    }

    fn clear_exchange_rate_form(&mut self) {
        self.form_from_currency.clear();
        self.form_to_currency.clear();
        self.form_rate.clear();
        self.form_source = String::from("manual");
        self.form_field_index = 0;
    }

    fn clear_conversion_form(&mut self) {
        self.form_convert_from.clear();
        self.form_convert_to.clear();
        self.form_convert_amount.clear();
        self.form_converted_result.clear();
        self.form_field_index = 0;
    }

    fn clear_recurring_form(&mut self) {
        self.form_account_id.clear();
        self.form_amount.clear();
        self.form_type = String::from("expense");
        self.form_description.clear();
        self.form_category_id.clear();
        self.form_recurring_frequency = String::from("monthly");
        self.form_field_index = 0;
    }

    fn clear_account_form(&mut self) {
        self.form_account_name.clear();
        self.form_account_bank.clear();
        self.form_account_type = String::from("checking");
        self.form_account_currency = String::from("USD");
        self.form_account_balance = String::from("0");
        self.form_field_index = 0;
    }

    fn render_add_account_form(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        let currencies = vec!["USD", "EUR", "GBP", "CAD", "JPY", "AUD", "CHF", "CNY", "INR", "MXN", "BRL", "KRW"];

        // Get display name for current type
        let type_display = match self.form_account_type.to_lowercase().as_str() {
            "c" | "checking" => "checking",
            "s" | "savings" => "savings", 
            "r" | "credit" => "credit",
            "i" | "investment" => "investment",
            "h" | "cash" => "cash",
            _ => &self.form_account_type,
        };

        let form_text = vec![
            Line::from(Span::styled("Add New Account", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_account_name,
                    if self.form_field_index == 0 { Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED) }
                    else { Style::default().fg(Color::White) }),
            ]),
            Line::from(vec![
                Span::styled("Bank (optional): ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_account_bank,
                    if self.form_field_index == 1 { Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED) }
                    else { Style::default().fg(Color::White) }),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_account_type,
                    if self.form_field_index == 2 { Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED) }
                    else { Style::default().fg(Color::White) }),
                Span::styled(format!(" → {}", type_display), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("c", Style::default().fg(Color::Green)),
                Span::styled("=checking ", Style::default().fg(Color::DarkGray)),
                Span::styled("s", Style::default().fg(Color::Green)),
                Span::styled("=savings ", Style::default().fg(Color::DarkGray)),
                Span::styled("r", Style::default().fg(Color::Green)),
                Span::styled("=credit ", Style::default().fg(Color::DarkGray)),
                Span::styled("i", Style::default().fg(Color::Green)),
                Span::styled("=investment ", Style::default().fg(Color::DarkGray)),
                Span::styled("h", Style::default().fg(Color::Green)),
                Span::styled("=cash", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled("Currency: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_account_currency,
                    if self.form_field_index == 3 { Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED) }
                    else { Style::default().fg(Color::Green) }),
            ]),
            Line::from(vec![
                Span::styled("Initial Balance: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_account_balance,
                    if self.form_field_index == 4 { Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED) }
                    else { Style::default().fg(Color::White) }),
                Span::styled(format!(" {}", self.form_account_currency), Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Tab: Next | Enter: Submit | Esc: Cancel", Style::default().fg(Color::Cyan))),
        ];

        let form = Paragraph::new(form_text)
            .block(Block::default().borders(Borders::ALL).title("Add Account"))
            .alignment(Alignment::Left);
        frame.render_widget(form, chunks[0]);

        // Right panel: Currency and Type options
        let mut help_lines = vec![
            Line::from(Span::styled("Available Currencies:", Style::default().fg(Color::Yellow))),
            Line::from(""),
        ];
        for (i, curr) in currencies.iter().enumerate() {
            let is_sel = self.form_account_currency.to_uppercase() == *curr;
            help_lines.push(Line::from(Span::styled(
                format!("  {}{}", curr, if is_sel { " ◄" } else { "" }),
                if is_sel { Style::default().fg(Color::Green) } else { Style::default().fg(Color::White) }
            )));
            if (i + 1) % 4 == 0 { help_lines.push(Line::from("")); }
        }
        help_lines.push(Line::from(""));
        help_lines.push(Line::from(Span::styled("Account Types:", Style::default().fg(Color::Yellow))));
        help_lines.push(Line::from(""));
        let account_types = [("c", "checking"), ("s", "savings"), ("r", "credit"), ("i", "investment"), ("h", "cash")];
        for (shortcut, name) in &account_types {
            let is_sel = self.form_account_type.to_lowercase() == *name 
                || self.form_account_type.to_lowercase() == *shortcut;
            help_lines.push(Line::from(Span::styled(
                format!("  {} = {}{}", shortcut, name, if is_sel { " ◄" } else { "" }),
                if is_sel { Style::default().fg(Color::Green) } else { Style::default().fg(Color::White) }
            )));
        }

        let help = Paragraph::new(help_lines)
            .block(Block::default().borders(Borders::ALL).title("Options"))
            .alignment(Alignment::Left);
        frame.render_widget(help, chunks[1]);
    }

    async fn handle_add_account_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => { self.mode = Mode::Normal; }
            KeyCode::Tab => { self.form_field_index = (self.form_field_index + 1) % 5; }
            KeyCode::BackTab => {
                self.form_field_index = if self.form_field_index == 0 { 4 } else { self.form_field_index - 1 };
            }
            KeyCode::Enter => { self.submit_account().await; }
            KeyCode::Char(c) => match self.form_field_index {
                0 => self.form_account_name.push(c),
                1 => self.form_account_bank.push(c),
                2 => self.form_account_type.push(c),
                3 => self.form_account_currency.push(c.to_ascii_uppercase()),
                4 => if c.is_ascii_digit() || c == '.' || c == '-' { self.form_account_balance.push(c); }
                _ => {}
            },
            KeyCode::Backspace => match self.form_field_index {
                0 => { self.form_account_name.pop(); }
                1 => { self.form_account_bank.pop(); }
                2 => { self.form_account_type.pop(); }
                3 => { self.form_account_currency.pop(); }
                4 => { self.form_account_balance.pop(); }
                _ => {}
            },
            _ => {}
        }
    }

    async fn submit_account(&mut self) {
        if self.form_account_name.trim().is_empty() {
            self.status_message = "Error: Account name is required!".to_string();
            self.mode = Mode::Normal;
            return;
        }

        let balance = self.form_account_balance.parse::<f64>().unwrap_or(0.0);
        let bank_name = if self.form_account_bank.trim().is_empty() { None } else { Some(self.form_account_bank.clone()) };

        // Normalize account type shortcuts
        let account_type = match self.form_account_type.to_lowercase().as_str() {
            "c" | "checking" => "checking",
            "s" | "savings" => "savings",
            "r" | "credit" => "credit",
            "i" | "investment" => "investment",
            "h" | "cash" => "cash",
            _ => "checking",
        };

        let currency = self.form_account_currency.to_uppercase();
        if currency.is_empty() {
            self.status_message = "Error: Currency is required!".to_string();
            self.mode = Mode::Normal;
            return;
        }

        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                self.status_message = "Error: No user logged in!".to_string();
                self.mode = Mode::Normal;
                return;
            }
        };

        let result = sqlx::query(
            "INSERT INTO accounts (user_id, name, account_type, currency, current_balance, bank_name) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(&self.form_account_name)
        .bind(account_type)
        .bind(&currency)
        .bind(balance)
        .bind(&bank_name)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => {
                let account_id = res.last_insert_rowid();
                self.status_message = format!("Account '{}' created! ID: {} [{}]", self.form_account_name, account_id, currency);
                self.load_data().await;
                self.mode = Mode::Normal;
            }
            Err(e) => {
                self.status_message = format!("Error creating account: {}", e);
                self.mode = Mode::Normal;
            }
        }
    }

    fn clear_category_form(&mut self) {
        self.form_category_name.clear();
        self.form_category_type = String::from("expense");
        self.form_field_index = 0;
    }

    fn render_add_category_form(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let form_text = vec![
            Line::from(Span::styled("Add New Category", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_category_name,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Enter: Submit | Esc: Cancel", Style::default().fg(Color::Cyan))),
            Line::from(""),
            Line::from(Span::styled("Examples: Salary, Groceries, Rent, Entertainment, Subscriptions...", Style::default().fg(Color::DarkGray))),
        ];

        let form = Paragraph::new(form_text)
            .block(Block::default().borders(Borders::ALL).title("Add Category"))
            .alignment(Alignment::Left);
        frame.render_widget(form, area);
    }

    async fn handle_add_category_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => { self.mode = Mode::Normal; }
            KeyCode::Enter => { self.submit_category().await; }
            KeyCode::Char(c) => { self.form_category_name.push(c); }
            KeyCode::Backspace => { self.form_category_name.pop(); }
            _ => {}
        }
    }

    async fn submit_category(&mut self) {
        if self.form_category_name.trim().is_empty() {
            self.status_message = "Error: Category name is required!".to_string();
            self.mode = Mode::Normal;
            return;
        }

        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                self.status_message = "Error: No user logged in!".to_string();
                self.mode = Mode::Normal;
                return;
            }
        };

        let result = sqlx::query(
            "INSERT INTO categories (user_id, name) VALUES (?, ?)"
        )
        .bind(user_id)
        .bind(&self.form_category_name)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => {
                let category_id = res.last_insert_rowid();
                self.status_message = format!("Category '{}' created! ID: {}", self.form_category_name, category_id);
                self.load_data().await;
                self.mode = Mode::Normal;
            }
            Err(e) => {
                self.status_message = format!("Error creating category: {}", e);
                self.mode = Mode::Normal;
            }
        }
    }

    fn clear_user_form(&mut self) {
        self.form_user_username.clear();
        self.form_user_email.clear();
        self.form_field_index = 0;
    }

    fn render_add_user_form(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Title
                Constraint::Min(0),     // Form
                Constraint::Length(3),  // Instructions
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new(vec![
            Line::from(Span::styled("Personal Finance Tracker", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Create New User", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Form
        let form_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Username: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_user_username,
                    if self.form_field_index == 0 { Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED) }
                    else { Style::default().fg(Color::White) }),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Email:    ", Style::default().fg(Color::Gray)),
                Span::styled(&self.form_user_email,
                    if self.form_field_index == 1 { Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED) }
                    else { Style::default().fg(Color::White) }),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("  (Password will be set to 'password123' by default)", Style::default().fg(Color::DarkGray))),
        ];

        let form = Paragraph::new(form_text)
            .block(Block::default().borders(Borders::ALL).title("New User Details"));
        frame.render_widget(form, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Tab: Next field | Enter: Create | Esc: Cancel")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(instructions, chunks[2]);
    }

    async fn handle_add_user_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => { self.mode = Mode::Normal; }
            KeyCode::Tab => { self.form_field_index = (self.form_field_index + 1) % 2; }
            KeyCode::BackTab => {
                self.form_field_index = if self.form_field_index == 0 { 1 } else { 0 };
            }
            KeyCode::Enter => { self.submit_user().await; }
            KeyCode::Char(c) => match self.form_field_index {
                0 => self.form_user_username.push(c),
                1 => self.form_user_email.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.form_field_index {
                0 => { self.form_user_username.pop(); }
                1 => { self.form_user_email.pop(); }
                _ => {}
            },
            _ => {}
        }
    }

    async fn submit_user(&mut self) {
        if self.form_user_username.trim().is_empty() {
            self.status_message = "Error: Username is required!".to_string();
            return;
        }

        if self.form_user_email.trim().is_empty() {
            self.status_message = "Error: Email is required!".to_string();
            return;
        }

        // Use a simple default password hash (in production, this should be properly hashed)
        let default_password_hash = "$argon2id$v=19$m=19456,t=2,p=1$defaulthash";

        let result = sqlx::query(
            "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)"
        )
        .bind(&self.form_user_username)
        .bind(&self.form_user_email)
        .bind(default_password_hash)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => {
                let user_id = res.last_insert_rowid();
                self.status_message = format!("User '{}' created! ID: {}", self.form_user_username, user_id);
                // Reload users list
                self.users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username")
                    .fetch_all(&self.pool)
                    .await
                    .unwrap_or_default();
                self.mode = Mode::Normal;
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE") {
                    self.status_message = "Error: Username or email already exists!".to_string();
                } else {
                    self.status_message = format!("Error creating user: {}", e);
                }
            }
        }
    }

    fn render_delete_user_confirm(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Title
                Constraint::Min(0),     // Confirm dialog
                Constraint::Length(3),  // Instructions
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new(vec![
            Line::from(Span::styled("Personal Finance Tracker", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Delete User", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Confirm dialog
        let user = if self.selected_index < self.users.len() {
            &self.users[self.selected_index]
        } else {
            return;
        };

        let dialog_text = vec![
            Line::from(""),
            Line::from(Span::styled("⚠ DELETE USER?", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(format!("  Username: {}", user.username)),
            Line::from(format!("  Email: {}", user.email)),
            Line::from(format!("  ID: {}", user.id)),
            Line::from(""),
            Line::from(Span::styled("  ⚠ WARNING: This will PERMANENTLY delete:", Style::default().fg(Color::Yellow))),
            Line::from(Span::styled("    - All accounts for this user", Style::default().fg(Color::Yellow))),
            Line::from(Span::styled("    - All transactions for this user", Style::default().fg(Color::Yellow))),
            Line::from(Span::styled("    - All categories for this user", Style::default().fg(Color::Yellow))),
            Line::from(Span::styled("    - All recurring transactions", Style::default().fg(Color::Yellow))),
            Line::from(""),
        ];

        let dialog = Paragraph::new(dialog_text)
            .block(Block::default().borders(Borders::ALL).title("Confirm Delete"));
        frame.render_widget(dialog, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("y: Confirm Delete | n/Esc: Cancel")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(instructions, chunks[2]);
    }

    async fn handle_delete_user_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if self.selected_index < self.users.len() {
                    let user = &self.users[self.selected_index];
                    let user_id = user.id;
                    let username = user.username.clone();

                    // Delete in order: transaction_categories, transactions, recurring_transactions, categories, accounts, user
                    // 1. Delete transaction_categories for all user's transactions
                    let _ = sqlx::query(
                        "DELETE FROM transaction_categories WHERE transaction_id IN 
                         (SELECT id FROM transactions WHERE account_id IN 
                          (SELECT id FROM accounts WHERE user_id = ?))"
                    ).bind(user_id).execute(&self.pool).await;

                    // 2. Delete all transactions for user's accounts
                    let _ = sqlx::query(
                        "DELETE FROM transactions WHERE account_id IN (SELECT id FROM accounts WHERE user_id = ?)"
                    ).bind(user_id).execute(&self.pool).await;

                    // 3. Delete recurring transactions
                    let _ = sqlx::query("DELETE FROM recurring_transactions WHERE account_id IN (SELECT id FROM accounts WHERE user_id = ?)")
                        .bind(user_id).execute(&self.pool).await;

                    // 4. Delete categories
                    let _ = sqlx::query("DELETE FROM categories WHERE user_id = ?")
                        .bind(user_id).execute(&self.pool).await;

                    // 5. Delete accounts
                    let _ = sqlx::query("DELETE FROM accounts WHERE user_id = ?")
                        .bind(user_id).execute(&self.pool).await;

                    // 6. Delete user
                    let result = sqlx::query("DELETE FROM users WHERE id = ?")
                        .bind(user_id)
                        .execute(&self.pool)
                        .await;

                    match result {
                        Ok(_) => {
                            self.status_message = format!("User '{}' and all data deleted!", username);
                            // Reload users list
                            self.users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username")
                                .fetch_all(&self.pool)
                                .await
                                .unwrap_or_default();
                            self.selected_index = 0;
                        }
                        Err(e) => {
                            self.status_message = format!("Error deleting user: {}", e);
                        }
                    }
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }
}
