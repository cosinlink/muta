use std::sync::Arc;

use futures::future::{err, ok, result, Future};
use rocksdb::{ColumnFamily, Error as RocksError, Options, WriteBatch, DB};

use core_runtime::{DataCategory, Database, DatabaseError, FutRuntimeResult};

pub struct RocksDB {
    db: Arc<DB>,
}

impl RocksDB {
    // TODO: Configure RocksDB options for each column.
    pub fn new(path: &str) -> Result<Self, DatabaseError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let categories = [
            map_data_category(DataCategory::Block),
            map_data_category(DataCategory::Transaction),
            map_data_category(DataCategory::Receipt),
            map_data_category(DataCategory::State),
            map_data_category(DataCategory::TransactionPool),
            map_data_category(DataCategory::TransactionPosition),
        ];
        let db = DB::open_cf(&opts, path, categories.iter())
            .map_err(|e| DatabaseError::Internal(e.to_string()))?;

        Ok(RocksDB { db: Arc::new(db) })
    }

    #[cfg(test)]
    pub fn clean(&self) {
        let categories = [
            map_data_category(DataCategory::Block),
            map_data_category(DataCategory::Transaction),
            map_data_category(DataCategory::Receipt),
            map_data_category(DataCategory::State),
            map_data_category(DataCategory::TransactionPool),
            map_data_category(DataCategory::TransactionPosition),
        ];

        for c in categories.iter() {
            self.db.drop_cf(c).unwrap();
        }
    }

    fn get_column(&self, c: DataCategory) -> Result<ColumnFamily, DatabaseError> {
        self.db
            .cf_handle(map_data_category(c))
            .ok_or(DatabaseError::NotFound)
    }
}

impl Database for RocksDB {
    fn get(&self, c: DataCategory, key: &[u8]) -> FutRuntimeResult<Vec<u8>, DatabaseError> {
        let column = match self.get_column(c) {
            Ok(column) => column,
            Err(e) => return Box::new(err(e)),
        };

        let v = match self.db.get_cf(column, key) {
            Ok(opt_v) => match opt_v {
                Some(v) => v.to_vec(),
                None => return Box::new(err(DatabaseError::NotFound)),
            },
            Err(e) => return Box::new(err(map_db_err(e))),
        };

        Box::new(ok(v.to_vec()))
    }

    fn get_batch(
        &self,
        c: DataCategory,
        keys: &[Vec<u8>],
    ) -> FutRuntimeResult<Vec<Option<Vec<u8>>>, DatabaseError> {
        let column = match self.get_column(c) {
            Ok(column) => column,
            Err(e) => return Box::new(err(e)),
        };

        let values: Vec<Option<Vec<u8>>> = keys
            .iter()
            .map(|key| match self.db.get_cf(column, key) {
                Ok(opt_v) => match opt_v {
                    Some(v) => Some(v.to_vec()),
                    None => None,
                },
                Err(e) => {
                    log::error!(target: "database rocksdb", "{}", e);
                    None
                }
            })
            .collect();

        Box::new(ok(values))
    }

    fn insert(
        &self,
        c: DataCategory,
        key: &[u8],
        value: &[u8],
    ) -> FutRuntimeResult<(), DatabaseError> {
        let column = match self.get_column(c) {
            Ok(column) => column,
            Err(e) => return Box::new(err(e)),
        };

        let fut = result(self.db.put_cf(column, key, value)).map_err(map_db_err);
        Box::new(fut)
    }

    fn insert_batch(
        &self,
        c: DataCategory,
        keys: &[Vec<u8>],
        values: &[Vec<u8>],
    ) -> FutRuntimeResult<(), DatabaseError> {
        if keys.len() != values.len() {
            return Box::new(err(DatabaseError::InvalidData));
        }

        let column = match self.get_column(c) {
            Ok(column) => column,
            Err(e) => return Box::new(err(e)),
        };

        let mut batch = WriteBatch::default();
        for i in 0..keys.len() {
            if let Err(e) = batch.put_cf(column, &keys[i], &values[i]) {
                return Box::new(err(map_db_err(e)));
            }
        }

        let fut = result(self.db.write(batch)).map_err(map_db_err);
        Box::new(fut)
    }

    fn contains(&self, c: DataCategory, key: &[u8]) -> FutRuntimeResult<bool, DatabaseError> {
        let column = match self.get_column(c) {
            Ok(column) => column,
            Err(e) => return Box::new(err(e)),
        };

        let v = match self.db.get_cf(column, key) {
            Ok(opt_v) => opt_v.is_some(),
            Err(e) => return Box::new(err(map_db_err(e))),
        };

        Box::new(ok(v))
    }

    fn remove(&self, c: DataCategory, key: &[u8]) -> FutRuntimeResult<(), DatabaseError> {
        let column = match self.get_column(c) {
            Ok(column) => column,
            Err(e) => return Box::new(err(e)),
        };

        let fut = result(self.db.delete_cf(column, key)).map_err(map_db_err);
        Box::new(fut)
    }

    fn remove_batch(
        &self,
        c: DataCategory,
        keys: &[Vec<u8>],
    ) -> FutRuntimeResult<(), DatabaseError> {
        let column = match self.get_column(c) {
            Ok(column) => column,
            Err(e) => return Box::new(err(e)),
        };

        let mut batch = WriteBatch::default();
        for key in keys {
            if let Err(e) = batch.delete_cf(column, &key) {
                return Box::new(err(map_db_err(e)));
            }
        }
        let fut = result(self.db.write(batch)).map_err(map_db_err);
        Box::new(fut)
    }
}

const C_BLOCK: &str = "c1";
const C_TRANSACTION: &str = "c2";
const C_RECEIPT: &str = "c3";
const C_STATE: &str = "c4";
const C_TRANSACTION_POOL: &str = "c5";
const C_TRANSACTION_POSITION: &str = "c6";

fn map_data_category(category: DataCategory) -> &'static str {
    match category {
        DataCategory::Block => C_BLOCK,
        DataCategory::Transaction => C_TRANSACTION,
        DataCategory::Receipt => C_RECEIPT,
        DataCategory::State => C_STATE,
        DataCategory::TransactionPool => C_TRANSACTION_POOL,
        DataCategory::TransactionPosition => C_TRANSACTION_POSITION,
    }
}

fn map_db_err(err: RocksError) -> DatabaseError {
    DatabaseError::Internal(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::RocksDB;

    use core_runtime::{DataCategory, Database, DatabaseError};
    use futures::future::Future;

    #[test]
    fn test_get_should_return_ok() {
        let db = RocksDB::new("rocksdb/test_get_should_return_ok").unwrap();

        check_not_found(db.get(DataCategory::Block, b"test").wait());
        db.insert(DataCategory::Block, b"test", b"test")
            .wait()
            .unwrap();
        let v = db.get(DataCategory::Block, b"test").wait().unwrap();
        assert_eq!(v, b"test".to_vec());
        db.clean();
    }

    #[test]
    fn test_insert_should_return_ok() {
        let db = RocksDB::new("rocksdb/test_insert_should_return_ok").unwrap();

        db.insert(DataCategory::Block, b"test", b"test")
            .wait()
            .unwrap();
        assert_eq!(
            b"test".to_vec(),
            db.get(DataCategory::Block, b"test").wait().unwrap()
        );
        db.clean();
    }

    #[test]
    fn test_insert_batch_should_return_ok() {
        let db = RocksDB::new("rocksdb/test_insert_batch_should_return_ok").unwrap();

        db.insert_batch(
            DataCategory::Block,
            &[b"test1".to_vec(), b"test2".to_vec()],
            &[b"test1".to_vec(), b"test2".to_vec()],
        )
        .wait()
        .unwrap();
        assert_eq!(
            b"test1".to_vec(),
            db.get(DataCategory::Block, b"test1").wait().unwrap()
        );
        assert_eq!(
            b"test2".to_vec(),
            db.get(DataCategory::Block, b"test2").wait().unwrap()
        );
        db.clean();
    }

    #[test]
    fn test_contain_should_return_true() {
        let db = RocksDB::new("rocksdb/test_contain_should_return_true").unwrap();

        db.insert(DataCategory::Block, b"test", b"test")
            .wait()
            .unwrap();
        assert_eq!(
            db.contains(DataCategory::Block, b"test").wait().unwrap(),
            true
        );

        db.clean()
    }

    #[test]
    fn test_contain_should_return_false() {
        let db = RocksDB::new("rocksdb/test_contain_should_return_false").unwrap();

        assert_eq!(
            db.contains(DataCategory::Block, b"test").wait().unwrap(),
            false
        );
        db.clean();
    }

    #[test]
    fn test_remove_should_return_ok() {
        let db = RocksDB::new("rocksdb/test_remove_should_return_ok").unwrap();

        db.insert(DataCategory::Block, b"test", b"test")
            .wait()
            .unwrap();
        db.remove(DataCategory::Block, b"test").wait().unwrap();
        check_not_found(db.get(DataCategory::Block, b"test").wait());
        db.clean();
    }

    #[test]
    fn test_remove_batch_should_return_ok() {
        let db = RocksDB::new("rocksdb/test_remove_batch_should_return_ok").unwrap();

        db.insert_batch(
            DataCategory::Block,
            &[b"test1".to_vec(), b"test2".to_vec()],
            &[b"test1".to_vec(), b"test2".to_vec()],
        )
        .wait()
        .unwrap();
        db.remove_batch(DataCategory::Block, &[b"test1".to_vec(), b"test2".to_vec()])
            .wait()
            .unwrap();
        check_not_found(db.get(DataCategory::Block, b"test1").wait());
        check_not_found(db.get(DataCategory::Block, b"test2").wait());
        db.clean();
    }

    fn check_not_found<T>(res: Result<T, DatabaseError>) {
        match res {
            Ok(_) => panic!("The result must be an error not found"),
            Err(e) => assert_eq!(e, DatabaseError::NotFound),
        }
    }
}