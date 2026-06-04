// Re-export everything inside them so they are accessible at this level

/* Accounts Management */
pub use crate::signup::signup::*;
pub use crate::login::login::*;
pub use crate::forgotten_password::forgotten_pwd::*;
pub use crate::change_password::change_pwd::*;

/* Verification Routes */
pub use crate::verification::verify_account::*;

/* Web Socket Routes */
pub use crate::ws::ws_handler::*;

/* Index File */
pub use crate::index::*;
