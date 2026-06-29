use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Client already exists")]
    AlreadyExists,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Mathematical overflow")]
    Overflow,
    #[error("Lock error: {0}")]
    LockError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
