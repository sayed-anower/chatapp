mod index;
mod signup;
mod login;
mod verify_signup;
mod forgotten_pwd;
 mod verify_fpwd;
 mod change_pwd;

// Re-export everything inside them so they are accessible at this level
pub use index::*;
pub use signup::*;
pub use login::*;
pub use verify_signup::*;
pub use forgotten_pwd::*;
pub use verify_fpwd::*;
pub use change_pwd::*;
