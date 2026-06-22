# End-to-End Tests

This directory contains the End-to-End (E2E) integration tests for the `challenge-prex` application.

## Test Workflow

The E2E tests (`e2e.rs`) simulate a complete, sequential user journey to ensure all endpoints work together correctly and state changes are properly tracked in memory and persisted when requested.

The single test scenario performs the following operations in sequence:

1. **Account Creation (`/new_client`)**
   - **Action**: Sends a `POST` request with valid user details (name, past birthdate, document number, country code).
   - **Validation**: Ensures a valid `client_id` is generated and returned.

2. **Initial Balance Check (`/client_balance`)**
   - **Action**: Sends a `GET` request using the newly created `client_id`.
   - **Validation**: Asserts the initial account balance is exactly `0`.

3. **First Balance Store (`/store_balances`)**
   - **Action**: Sends a `POST` request to trigger a file dump of the in-memory state.
   - **Validation**: 
     - Verifies the atomic file counter initialized at `1` by checking that the filename ends with `_1.DAT`.
     - Reads the generated file to ensure it accurately contains `[CLIENT_ID] 0`.
     - Cleans up (deletes) the test file to avoid polluting the workspace.

4. **Credit Transaction (`/new_credit_transaction`)**
   - **Action**: Sends a `POST` request to credit `150.50` to the account.
   - **Validation**: 
     - Checks the endpoint response returns the expected new balance.
     - Fetches the balance again via `/client_balance` to ensure the core data structure was genuinely updated to `150.50`.

5. **Second Balance Store (`/store_balances`)**
   - **Action**: Sends a `POST` request to trigger a second file dump.
   - **Validation**: 
     - Verifies the atomic file counter safely incremented to `2` (`..._2.DAT`).
     - Reads the file to ensure the content reflects the new credited balance (`[CLIENT_ID] 150.50`).
     - Cleans up the test file.

6. **Debit Transaction (`/new_debit_transaction`)**
   - **Action**: Sends a `POST` request to debit `50.25` from the account.
   - **Validation**: 
     - Checks the endpoint response reflects the expected `100.25` balance.
     - Fetches the balance via `/client_balance` again to ensure the memory state correctly reflects the subtraction.

7. **Third Balance Store (`/store_balances`)**
   - **Action**: Sends a `POST` request to trigger a third and final file dump.
   - **Validation**: 
     - Verifies the atomic file counter incremented to `3` (`..._3.DAT`).
     - Reads the file to ensure the content reflects the final debited balance (`[CLIENT_ID] 100.25`).
     - Cleans up the test file.

## Running the Tests

To run the E2E tests alongside the standard unit tests, simply use the standard Cargo test command:

```bash
cargo test
```

## Architecture Note: Concurrency
The `/store_balances` endpoint is fully synchronized using a `std::sync::Mutex<u64>` to wrap the file counter. This architecture choice ensures that the process of formatting the sequential filenames, writing the states to disk, and incrementing the counter operates as a single, atomic operation across requests—preventing file overwrites and skipped numbers under load.
