mod key_value;

pub use key_value::KeyValueStorageAdapter;

use crate::account::{Account, AccountIdentifier};
use crate::address::Address;
use crate::transaction::Transaction;
use bee_crypto::ternary::Hash;
use once_cell::sync::OnceCell;

static INSTANCE: OnceCell<Box<dyn StorageAdapter + Sync + Send>> = OnceCell::new();

/// sets the storage adapter
pub fn set_adapter(storage: impl StorageAdapter + Sync + Send + 'static) -> crate::Result<()> {
  INSTANCE
    .set(Box::new(storage))
    .map_err(|_| anyhow::anyhow!("failed to globall set the storage instance"))?;
  Ok(())
}

/// gets the storage adapter
#[allow(clippy::borrowed_box)]
pub(crate) fn get_adapter() -> crate::Result<&'static Box<dyn StorageAdapter + Sync + Send>> {
  INSTANCE.get_or_try_init(|| {
    let instance = Box::new(KeyValueStorageAdapter::new("./example-database")?)
      as Box<dyn StorageAdapter + Sync + Send>;
    Ok(instance)
  })
}

/// The storage adapter.
pub trait StorageAdapter {
  /// Gets the account with the given id/alias from the storage.
  fn get(&self, key: AccountIdentifier) -> crate::Result<String>;
  /// Gets all the accounts from the storage.
  fn get_all(&self) -> crate::Result<Vec<String>>;
  /// Saves or updates an account on the storage.
  fn set(&self, key: AccountIdentifier, account: String) -> crate::Result<()>;
  /// Removes an account from the storage.
  fn remove(&self, key: AccountIdentifier) -> crate::Result<()>;
}

/// Transaction type.
pub enum TransactionType {
  /// Transaction received.
  Received,
  /// Transaction sent.
  Sent,
  /// Transaction not broadcasted.
  Failed,
  /// Transaction not confirmed.
  Unconfirmed,
}

pub(crate) fn parse_accounts<'a>(accounts: &'a Vec<String>) -> crate::Result<Vec<Account<'a>>> {
  let mut err = None;
  let accounts: Vec<Option<Account<'a>>> = accounts
    .iter()
    .map(|account| {
      let res: Option<Account<'a>> = serde_json::from_str(&account)
        .map(|v| Some(v))
        .unwrap_or_else(|e| {
          err = Some(e);
          None
        });
      res
    })
    .collect();

  if let Some(err) = err {
    Err(err.into())
  } else {
    let accounts = accounts
      .iter()
      .map(|account| account.clone().unwrap())
      .collect();
    Ok(accounts)
  }
}

/// Gets the account's total balance.
/// It's read directly from the storage. To read the latest account balance, you should `sync` first.
pub(crate) fn total_balance(account_id: &str) -> crate::Result<u64> {
  unimplemented!()
}

/// Gets the account's available balance.
/// It's read directly from the storage. To read the latest account balance, you should `sync` first.
///
/// The available balance is the balance users are allowed to spend.
/// For example, if a user with 50i total account balance has made a transaction spending 20i,
/// the available balance should be (50i-30i) = 20i.
pub(crate) fn available_balance(account_id: &str) -> crate::Result<u64> {
  unimplemented!()
}

/// Updates the account alias.
pub(crate) fn set_alias(account_id: &str, alias: &str) -> crate::Result<()> {
  unimplemented!()
}

/// Gets a list of transactions on the given account.
/// It's fetched from the storage. To ensure the database is updated with the latest transactions,
/// `sync` should be called first.
///
/// * `account_id` - The account identifier
/// * `count` - Number of (most recent) transactions to fetch.
/// * `from` - Starting point of the subset to fetch.
/// * `transaction_type` - Optional transaction type filter.
pub(crate) fn list_transactions(
  account_id: &str,
  count: u64,
  from: u64,
  transaction_type: Option<TransactionType>,
) -> crate::Result<Vec<Transaction>> {
  Ok(vec![])
}

/// Gets the addresses linked to the given account.
///
/// * `account_id` - The account identifier.
/// * `unspent` - Whether it should get only unspent addresses or not.
pub(crate) fn list_addresses(account_id: &str, unspent: bool) -> crate::Result<Vec<Address>> {
  unimplemented!()
}

/// Gets the transaction associated with the given hash.
pub(crate) fn get_transaction(transaction_hash: Hash) -> crate::Result<Transaction> {
  unimplemented!()
}

/// Gets a new unused address and links it to the given account.
pub(crate) fn generate_address(account_id: &str) -> crate::Result<Address> {
  unimplemented!()
}
