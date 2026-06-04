pub mod index;
pub use index::*;

// Re-export everything inside them so they are accessible at this level

/* Accounts Management */
pub use signup::signup::*;
pub use login::login::*;
pub use forgotten_password::forgotten_pwd::*;
pub use change_password::change_pwd::*;

/* Verification Routes */
pub use verification::verify_account::*;

/* Web Socket Routes */
pub use ws::ws_handler::*;

/* */