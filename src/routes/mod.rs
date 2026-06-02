pub mod index;
pub mod signup;
pub mod login;
pub mod verify_signup;
pub mod forgotten_pwd;
pub mod verify_fpwd;
pub mod change_pwd;

// Re-export everything inside them so they are accessible at this level
pub use index::*;
pub use signup::*;
pub use login::*;
pub use verify_signup::*;
pub use forgotten_pwd::*;
pub use verify_fpwd::*;
pub use change_pwd::*;
