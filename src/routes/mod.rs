pub mod index;
pub use index::index;

pub mod signup;
pub use signup::signup;

pub mod login;
pub use login::login;

pub mod verify_signup;
pub use verify_signup::verify_signup;

pub mod forgotten_pwd;
pub use forgotten_pwd::forgotten_pwd;

pub mod verify_fpwd;
pub use verify_fpwd::change_pwd as verify_change_pwd; 

pub mod change_pwd;
pub use change_pwd::change_pwd;