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
    ExchangeRates,
    Reports,
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    AddTransaction,
    AddExchangeRate,
    DeleteConfirm,
    ViewDetails,
    ConvertCurrency,
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

    // Selection state
    selected_index: usize,
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
            for id in account_ids {
                q = q.bind(id);
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

        if let Ok(rates) = sqlx::query_as::<_, ExchangeRate>(
            "SELECT * FROM exchange_rates ORDER BY rate_date DESC, from_currency LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await
        {
            self.exchange_rates = rates;
        }
    }

    fn ui(&self, frame: &mut Frame) {
        if self.current_screen == Screen::UserSelect {
            self.render_user_select(frame);
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
                Screen::ExchangeRates => self.render_exchange_rates(frame, chunks[2]),
                Screen::Reports => self.render_reports(frame, chunks[2]),
                Screen::UserSelect => {}
            },
            Mode::AddTransaction => self.render_add_transaction_form(frame, chunks[2]),
            Mode::AddExchangeRate => self.render_add_exchange_rate_form(frame, chunks[2]),
            Mode::ConvertCurrency => self.render_currency_conversion(frame, chunks[2]),
            Mode::DeleteConfirm => self.render_delete_confirm(frame, chunks[2]),
            Mode::ViewDetails => self.render_details(frame, chunks[2]),
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
                .title(format!("Available Users ({})", self.users.len())),
        );
        frame.render_widget(list, chunks[1]);

        // Instructions
        let instructions =
            Paragraph::new("Up/Down: Select user | Enter: Login as selected user | q: Quit")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
        frame.render_widget(instructions, chunks[2]);
    }

    fn render_header(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let mode_indicator = match self.mode {
            Mode::Normal => "",
            Mode::AddTransaction => " [ADD MODE]",
            Mode::AddExchangeRate => " [ADD EXCHANGE RATE]",
            Mode::ConvertCurrency => " [CONVERT CURRENCY]",
            Mode::DeleteConfirm => " [DELETE CONFIRM]",
            Mode::ViewDetails => " [DETAILS]",
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
            "Exchange Rates",
            "Reports",
        ];
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Menu"))
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

        let list = List::new(accounts).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("All Accounts ({})", self.accounts.len())),
        );
        frame.render_widget(list, area);
    }

    fn render_transactions(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let transactions: Vec<ListItem> = self
            .transactions
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

                ListItem::new(format!(
                    "{} | {} | ${:>10.2} | {}",
                    date_str,
                    type_str,
                    t.amount.abs(),
                    desc
                ))
                .style(style)
            })
            .collect();

        let list =
            List::new(transactions).block(Block::default().borders(Borders::ALL).title(format!(
                "All Transactions ({}) - a: Add | d: Delete | Enter: Details",
                self.transactions.len()
            )));
        frame.render_widget(list, area);
    }

    fn render_categories(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let all_cats: Vec<ListItem> = self
            .categories
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("ID: {} - {}", c.id, c.name)).style(style)
            })
            .collect();

        let left_half: Vec<ListItem> = all_cats.iter().take(all_cats.len() / 2).cloned().collect();
        let right_half: Vec<ListItem> = all_cats.iter().skip(all_cats.len() / 2).cloned().collect();

        let left_list = List::new(left_half).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Categories (Part 1)"),
        );
        frame.render_widget(left_list, chunks[0]);

        let right_list = List::new(right_half).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Categories (Part 2)"),
        );
        frame.render_widget(right_list, chunks[1]);
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

        let list =
            List::new(rate_items).block(Block::default().borders(Borders::ALL).title(format!(
                "Exchange Rates ({}) - a: Add | c: Convert | d: Delete | Enter: Details",
                self.exchange_rates.len()
            )));
        frame.render_widget(list, area);
    }

    fn render_reports(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(0)])
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
            ]),
            Line::from(vec![
                Span::styled("Total Expenses:     ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("${:.2}", total_expenses),
                    Style::default().fg(Color::Red),
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
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Transaction Count:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", transaction_count),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Categories:         ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", self.categories.len()),
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
        frame.render_widget(list, chunks[1]);
    }

    fn render_add_transaction_form(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let form_text = vec![
            Line::from(vec![Span::styled(
                "Add New Transaction",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Account ID: ", Style::default().fg(Color::Gray)),
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
            ]),
            Line::from(vec![
                Span::styled("Type (income/expense): ", Style::default().fg(Color::Gray)),
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
                "Tab: Next field | Shift+Tab: Previous | Enter: Submit | Esc: Cancel",
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
        frame.render_widget(form, area);
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
                "Tab: Next field | Shift+Tab: Previous | Enter: Submit | Esc: Cancel",
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
            vec![
                Line::from(vec![Span::styled(
                    "Account Details",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(format!("ID: {}", a.id)),
                Line::from(format!("Name: {}", a.name)),
                Line::from(format!("Type: {}", a.account_type)),
                Line::from(format!("Bank: {}", a.bank_name.as_deref().unwrap_or("N/A"))),
                Line::from(format!("Currency: {}", a.currency)),
                Line::from(format!("Initial Balance: ${:.2}", a.initial_balance)),
                Line::from(format!("Current Balance: ${:.2}", a.current_balance)),
                Line::from(format!(
                    "Created: {}",
                    a.created_at.format("%Y-%m-%d %H:%M:%S")
                )),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press Esc to go back",
                    Style::default().fg(Color::Gray),
                )]),
            ]
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
                        Paragraph::new("Up/Down: Select user | Enter: Login | q: Quit")
                    } else if self.current_screen == Screen::ExchangeRates {
                        Paragraph::new("Left/Right or 1-6: Switch tabs | Up/Down: Select | a: Add | c: Convert | d: Delete | Enter: Details | r: Refresh | u: Switch user | q: Quit")
                    } else {
                        Paragraph::new("Left/Right or 1-6: Switch tabs | Up/Down: Select | a: Add | d: Delete | Enter: Details | r: Refresh | u: Switch user | q: Quit")
                    }
                }
                Mode::AddTransaction => Paragraph::new(
                    "Tab/Shift+Tab: Navigate fields | Type to input | Enter: Submit | Esc: Cancel"
                ),
                Mode::AddExchangeRate => Paragraph::new(
                    "Tab/Shift+Tab: Navigate fields | Type to input | Enter: Submit | Esc: Cancel"
                ),
                Mode::ConvertCurrency => Paragraph::new(
                    "Tab/Shift+Tab: Navigate fields | Type to input | Enter: Convert | Esc: Cancel"
                ),
                Mode::DeleteConfirm => Paragraph::new(
                    "y: Confirm delete | n: Cancel"
                ),
                Mode::ViewDetails => Paragraph::new(
                    "Esc: Go back"
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
                            Mode::ConvertCurrency => {
                                self.handle_convert_currency_mode(key.code).await
                            }
                            Mode::DeleteConfirm => self.handle_delete_mode(key.code).await,
                            Mode::ViewDetails => self.handle_details_mode(key.code),
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_user_select_mode(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
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
            }
            KeyCode::Char('r') => {
                self.load_data().await;
                self.status_message = "Data refreshed!".to_string();
            }
            KeyCode::Char('a') => {
                if self.current_screen == Screen::Transactions {
                    self.mode = Mode::AddTransaction;
                    self.clear_transaction_form();
                } else if self.current_screen == Screen::ExchangeRates {
                    self.mode = Mode::AddExchangeRate;
                    self.clear_exchange_rate_form();
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
                }
            }
            KeyCode::Enter => {
                self.mode = Mode::ViewDetails;
            }
            KeyCode::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            KeyCode::Down => {
                let max = match self.current_screen {
                    Screen::Dashboard => self.transactions.len().min(10).saturating_sub(1),
                    Screen::Accounts => self.accounts.len().saturating_sub(1),
                    Screen::Transactions => self.transactions.len().saturating_sub(1),
                    Screen::Categories => self.categories.len().saturating_sub(1),
                    Screen::ExchangeRates => self.exchange_rates.len().saturating_sub(1),
                    _ => 0,
                };
                self.selected_index = (self.selected_index + 1).min(max);
            }
            KeyCode::Left => {
                self.selected_tab = self.selected_tab.saturating_sub(1);
                self.update_screen();
                self.selected_index = 0;
            }
            KeyCode::Right => {
                self.selected_tab = (self.selected_tab + 1).min(4);
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
        let txn_type = &self.form_type;
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
                    let transaction_id = self.transactions[self.selected_index].id;

                    let result = sqlx::query("DELETE FROM transactions WHERE id = ?")
                        .bind(transaction_id)
                        .execute(&self.pool)
                        .await;

                    match result {
                        Ok(_) => {
                            self.status_message =
                                format!("Transaction {} deleted successfully!", transaction_id);
                            self.load_data().await;
                            self.selected_index = 0;
                        }
                        Err(e) => {
                            self.status_message = format!("Error deleting transaction: {}", e);
                        }
                    }
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
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_details_mode(&mut self, code: KeyCode) {
        if code == KeyCode::Esc {
            self.mode = Mode::Normal;
        }
    }

    fn update_screen(&mut self) {
        self.current_screen = match self.selected_tab {
            0 => Screen::Dashboard,
            1 => Screen::Accounts,
            2 => Screen::Transactions,
            3 => Screen::Categories,
            4 => Screen::ExchangeRates,
            5 => Screen::Reports,
            _ => Screen::Dashboard,
        };
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
}
