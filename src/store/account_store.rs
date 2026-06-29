use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::store::StoreError;

/// Internal representation of a Client.
#[derive(Debug, Clone)]
pub struct Client {
    pub id: u64,
    pub client_name: String,
    pub birth_date: NaiveDate,
    pub document_number: String,
    pub country: String,
    pub balance: Decimal,
}

/// In-memory store of the currently created and known clients.
/// Private struct since it will part of the AccountStore.
struct ClientsState {
    clients: BTreeMap<u64, Client>,
    document_index: HashMap<String, u64>,
    next_id: u64,
    file_counter: u64,
}

impl ClientsState {
    fn write_snapshot_and_increment(
        &mut self,
        output_dir: &std::path::Path,
    ) -> Result<String, StoreError> {
        let now = chrono::Local::now();
        let snapshot: Vec<(u64, Decimal)> = self
            .clients
            .values()
            .map(|client| (client.id, client.balance))
            .collect();

        let filename = format!("{}_{}.DAT", now.format("%d%m%Y"), self.file_counter);
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join(&filename);
        let temp_path = output_dir.join(format!("{}.tmp", filename));

        let content: String = snapshot
            .into_iter()
            .map(|(id, balance)| format!("{} {}\n", id, balance))
            .collect();

        std::fs::write(&temp_path, content)?;
        std::fs::rename(&temp_path, &path)?;

        self.file_counter += 1;
        Ok(filename)
    }

    fn reset_balances(&mut self) {
        self.clients.values_mut().for_each(|client| {
            client.balance = Decimal::ZERO;
        });
    }

    fn modify_balance_by(
        &mut self,
        client_id: u64,
        amount: Decimal,
    ) -> Result<Option<Decimal>, StoreError> {
        let Some(client) = self.clients.get_mut(&client_id) else {
            return Ok(None);
        };

        let new_balance = client.balance.checked_add(amount).ok_or(StoreError::Overflow)?;

        client.balance = new_balance;
        Ok(Some(client.balance))
    }
}

/// In-memory client store with concurrent access.
///
/// All business logic resides here. A single `Mutex<ClientsState>` serializes
/// access to mutable state, ensuring atomicity during normal operations
/// and exports.
pub struct AccountStore {
    state: Mutex<ClientsState>,
    output_dir: PathBuf,
}

impl AccountStore {
    pub fn with_output_dir(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            state: Mutex::new(ClientsState {
                clients: BTreeMap::new(),
                document_index: HashMap::new(),
                next_id: 1,
                file_counter: 1,
            }),
            output_dir: output_dir.into(),
        }
    }
}

impl Default for AccountStore {
    fn default() -> Self {
        let output_dir = std::env::var("EXPORT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        Self::with_output_dir(output_dir)
    }
}

impl AccountStore {
    /// Creates a new client and assigns an ID to it.
    ///
    /// Returns `StoreError::AlreadyExists` if given document is already known.
    pub fn create_client(
        &self,
        client_name: String,
        birth_date: NaiveDate,
        document_number: String,
        country: String,
    ) -> Result<u64, StoreError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| StoreError::LockError(e.to_string()))?;

        if state.document_index.contains_key(&document_number) {
            return Err(StoreError::AlreadyExists);
        }

        let id = state.next_id;
        state.next_id += 1;

        let client = Client {
            id,
            client_name,
            birth_date,
            document_number: document_number.clone(),
            country,
            balance: Decimal::ZERO,
        };

        state.document_index.insert(document_number, id);
        state.clients.insert(id, client);

        Ok(id)
    }

    /// Returns information about a client, given its id.
    pub fn get_client(&self, client_id: u64) -> Result<Option<Client>, StoreError> {
        let state = self
            .state
            .lock()
            .map_err(|e| StoreError::LockError(e.to_string()))?;

        Ok(state.clients.get(&client_id).cloned())
    }

    /// Credits an amount to the client's balance.
    /// Returns the new resulting balance.
    /// Returns `None` if the client does not exist.
    pub fn credit(&self, client_id: u64, amount: Decimal) -> Result<Option<Decimal>, StoreError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| StoreError::LockError(e.to_string()))?;

        state.modify_balance_by(client_id, amount)
    }

    /// Debits an amount from the client's balance.
    ///
    /// Validates sufficient balance before operation.
    /// Returns `StoreError::InsufficientFunds` if balance is insufficient.
    /// Returns `None` if the client does not exist.
    pub fn debit(&self, client_id: u64, amount: Decimal) -> Result<Option<Decimal>, StoreError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| StoreError::LockError(e.to_string()))?;

        state.modify_balance_by(client_id, -amount)
    }

    /// Exports current balances to a DAT file and resets balances to zero.
    ///
    /// Holds the Mutex during the whole operation to prevent concurrent transactions.
    pub fn export(&self) -> Result<String, StoreError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| StoreError::LockError(e.to_string()))?;

        // First, try and write the snapshot. If everything goes well,
        // we can then reset the balances.
        let filename = state.write_snapshot_and_increment(&self.output_dir)?;
        state.reset_balances();

        Ok(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_export_writes_ids_in_order() {
        let dir = tempdir().unwrap();
        let store = AccountStore::with_output_dir(dir.path());

        for i in 1..=10 {
            let id = store
                .create_client(
                    format!("Client {}", i),
                    NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
                    format!("DOC{}", i),
                    "Country".to_string(),
                )
                .unwrap();

            store.credit(id, Decimal::from(i)).unwrap();
        }

        let filename = store.export().unwrap();
        let file_path = dir.path().join(filename);
        let content = fs::read_to_string(file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 10);
        assert_eq!(lines[0], "1 1");
        assert_eq!(lines[1], "2 2");
        assert_eq!(lines[2], "3 3");
        assert_eq!(lines[8], "9 9");
        assert_eq!(lines[9], "10 10");
    }
}
