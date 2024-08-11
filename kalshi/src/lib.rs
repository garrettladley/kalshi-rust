//! An HTTPS and Websocket wrapper that allows users to write trading bots for the [Kalshi events trading platform](https://kalshi.com).
//!
//! kalshi-rust is asynchronous, performant, and succint. Dash past verbose and annoying HTTPS requests
//! and use this wrapper to quickly write blazingly fast trading bots in Rust!
//!
//! As of version 0.9.0, HTTPS features are fully complete but websocket support and advanced API access features are not complete.
//! If you'd like to keep up on kalshi-rust's development, report bugs, or view a sample trading script,
//! feel free to visit the [github](https://github.com/dpeachpeach/kalshi-rust)!
//! A star would also be greatly appreciated, I'm a student developer writing this for free and any recognition is incredibly helpful!
//!
//! ## The Kalshi Struct
//!
//! The [Kalshi](Kalshi) struct is the central component of this crate.
//! All authentication, order routing, market requests, and position snapshots are handled through the struct and its methods.
//!
//! For more details, see [Kalshi](Kalshi).
//!
//! For a quick tutorial / beginners guide, jump [here](#quick-start-guide).
//!
//! ### Initializing the Kalshi struct in demo mode.
//! ```
//! use kalshi::Kalshi;
//! use kalshi::TradingEnvironment;
//!
//! let kalshi_instance = Kalshi::new(TradingEnvironment::DemoMode);
//! ```
//!
//! ## Quick Start Guide
//!
//! First, list the Kalshi struct as a dependency in your crate.
//!
//! ```toml
//! kalshi = { version = "0.9"}
//! ```
//!
//! Initialize the Kalshi Struct and login using your authentication details:
//! - **IMPORTANT**:  A user's authentication token expires every thirty minutes, this means
//! that you'll need to call the login function every thirty minutes in order to
//! ensure that you remain authenticated with a valid token.
//! - Storing user / password information in plaintext is not recommended,
//! an implementation of extracting user details from local environmental variables
//! is available [here](https://github.com/dpeachpeach/kalshi-rust/blob/main/sample_bot/src/main.rs#L12)
//! ```
//! use kalshi::Kalshi;
//! use kalshi::TradingEnvironment;
//!
//! let username = "johndoe@example.com";
//! let password = "example_password";
//!
//! let mut kalshi_instance = Kalshi::new(TradingEnvironment::DemoMode);
//!
//! let kalshi_instance = kalshi_instance.login(username, password).await?;
//! ```
//!
//! After logging in, you can call any method present in the crate without issue.
//! Here is a script that buys a 'yes' contract on November 13th's New York temperature
//! market.
//!
//! ```
//! let new_york_ticker = "HIGHNY-23NOV13-T51".to_string();
//!
//! let bought_order = kalshi_instance
//!     .create_order(
//!     kalshi::Action::Buy,
//!     None,
//!     1,
//!     kalshi::Side::Yes,
//!     new_york_ticker,
//!     kalshi::OrderType::Limit,
//!     None,
//!     None,
//!     None,
//!     None,
//!     Some(5)).await.unwrap();
//! ```
//!
//! Refer to the rest of the documentation for details on all other methods!
//! All methods found in the [kalshi API documentation](https://trading-api.readme.io/reference/getting-started) are wrapped around in this crate.
//!
//! ## Returned Values
//!
//! Whenever a user makes a method call using the kalshi struct, data is typically returned
//! in structs that encapsulate the json fields returned by the server. All data
//! in the structs is owned so a user can access the attributes without issue.
//!
//! ### Examples:
//!
//! #### Obtaining the Exchange's current status
//! Returns a struct that represents whether trading or the exchange are currently active.
//! ```
//! use kalshi::Kalshi;
//! use kalshi::TradingEnvironment;
//! let kalshi_instance = Kalshi::new(TradingEnvironment::DemoMode);
//!
//! kalshi_instance.get_exchange_status().await.unwrap();
//! ```
//!
//! #### Obtaining 5 miscellaneous market events
//! Returns a vector of 'event' structs and a cursor.
//! ```
//! use kalshi::Kalshi;
//! use kalshi::TradingEnvironment;
//! let kalshi_instance = Kalshi::new(TradingEnvironment::DemoMode);
//!
//! kalshi_instance.get_multiple_events(Some(5), None, None, None, None).await.unwrap();
//! ```
//! #### Checking the User's balance
//! Returns an i64 representing the user's balance in cents.
//! ```
//! use kalshi::Kalshi;
//! use kalshi::TradingEnvironment;
//! let kalshi_instance = Kalshi::new(TradingEnvironment::DemoMode);
//!
//! kalshi_instance.get_balance();
//! ```
//!

#[macro_use]
mod utils;
mod auth;
mod exchange;
mod kalshi_error;
mod market;
mod portfolio;

use std::marker::PhantomData;

pub use exchange::*;
pub use kalshi_error::*;
pub use market::*;
pub use portfolio::*;

#[derive(Debug, Clone)]
pub struct LoggedOut;

#[derive(Debug, Clone)]
pub struct LoggedIn;

/// The Kalshi struct is the core of the kalshi-crate. It acts as the interface
/// between the user and the market, abstracting away the meat of requests
/// by encapsulating authentication information and the client itself.
///
/// ## Creating a new `Kalshi` instance for demo mode:
///
/// ```
/// use kalshi::Kalshi;
/// use kalshi::TradingEnvironment;
///
/// let kalshi_instance = Kalshi::new(TradingEnvironment::DemoMode);
/// ```
///
///
#[derive(Debug, Clone)]
pub struct Kalshi<State = LoggedOut> {
    /// - `base_url`: The base URL for the API, determined by the trading environment.
    base_url: String,
    /// - `curr_token`: A field for storing the current authentication token.
    curr_token: Option<String>,
    /// - `member_id`: A field for storing the member ID.
    member_id: Option<String>,
    /// - `client`: The HTTP client used for making requests to the marketplace.
    client: reqwest::Client,
    state: PhantomData<State>,
}

impl Kalshi {
    /// Creates a new instance of Kalshi with the specified trading environment.
    /// This environment determines the base URL used for API requests.
    ///
    /// # Arguments
    ///
    /// * `trading_env` - The trading environment to be used (LiveMarketMode: Trading with real money. DemoMode: Paper Trading).
    ///
    /// # Example
    ///
    /// ## Creating a Demo instance.
    /// ```
    /// use kalshi::{Kalshi, TradingEnvironment};
    /// let kalshi = Kalshi::new(TradingEnvironment::DemoMode);
    /// ```
    ///
    /// ## Creating a Live Trading instance (Warning, you're using real money!)
    /// ```
    /// use kalshi::{Kalshi, TradingEnvironment};
    /// let kalshi = Kalshi::new(TradingEnvironment::LiveMarketMode);
    /// ```
    ///
    pub fn new(trading_env: TradingEnvironment) -> Kalshi<LoggedOut> {
        Kalshi {
            base_url: utils::build_base_url(trading_env).to_string(),
            curr_token: None,
            member_id: None,
            client: reqwest::Client::new(),
            state: PhantomData,
        }
    }

    /// Retrieves the current user authentication token, if available.
    ///
    /// # Returns
    ///
    /// Returns an `Option<String>` containing the authentication token. If no token
    /// is currently stored, it returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kalshi::{Kalshi, TradingEnvironment};
    /// let kalshi = Kalshi::new(TradingEnvironment::DemoMode);
    /// let token = kalshi.get_user_token();
    /// if let Some(t) = token {
    ///     println!("Current token: {}", t);
    /// } else {
    ///     println!("No token found");
    /// }
    /// ```
    ///
    pub fn get_user_token(&self) -> Option<String> {
        self.curr_token.clone()
    }
}

// GENERAL ENUMS
// -----------------------------------------------

/// Defines the trading environment for the Kalshi exchange.
///
/// This enum is used to specify whether the interaction with the Kalshi API should be in a demo (simulated) environment
/// or in the live market with real financial transactions.
///
pub enum TradingEnvironment {
    /// The demo mode represents a simulated environment where trades do not involve real money.
    /// This mode is typically used for testing and practice purposes.
    DemoMode,

    /// The live market mode is the real trading environment where all transactions involve actual financial stakes.
    /// Use this mode for actual trading activities with real money.
    LiveMarketMode,
}
