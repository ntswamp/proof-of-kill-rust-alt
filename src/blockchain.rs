use super::*;
use crate::block::*;
use crate::transaction::*;
use bincode::{deserialize, serialize};
use failure::format_err;
use sled;
use std::collections::HashMap;


const GENESIS_COINBASE_DATA: &str = "18:29, August 3rd, 2021, Tokyo. The sunset is beautiful.";

/// Blockchain implements interactions with a DB
#[derive(Debug)]
pub struct Blockchain {
    pub tip: String,
    pub db: sled::Db,
    pub kills: u128,
}

/// BlockchainIterator is used to iterate over blockchain blocks
pub struct BlockchainIterator<'a> {
    current_hash: String,
    blockchain: &'a Blockchain,
}

impl Blockchain {

    /// init() creates a new blockchain with DB initialized.
    pub fn init(address: String, node_id:&str) -> Result<Blockchain> {
        info!("Initialize New Blockchain");
        //e.g., data_3000/chain
        let db_path = "data_".to_owned() + node_id + "/chain";
        if db_exist(&db_path) {
            return Err(format_err!("ERROR: Blockchain Already Exists."));
        }

        //std::fs::remove_dir_all(&db_path).ok();
        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);

        let db = sled::open(db_path)?;
        debug!("Configuring A New Blockchain Database...");

        db.insert(genesis.get_hash(), serialize(&genesis)?)?;
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
            kills:0,
        };
        bc.db.flush()?;
        Ok(bc)
    }

    /// load() loads an existed Blockchain on the disk and .
    pub fn load(node_id:&str) -> Result<Blockchain> {
        //e.g., data/chain_3000
        let db_path = "data_".to_owned() + node_id + "/chain";
        if !db_exist(&db_path) {
            return Err(format_err!("blockchain database is not initialized.\nuse command `initdb` to initialize one."));
        }

        info!("Blockchain Database is Found. Loading...");
        let db = sled::open(db_path)?;
        let hash = match db.get("LAST")? {
            Some(last) => last.to_vec(),
            None => Vec::new(),
        };
        
        let lasthash = if hash.is_empty() {
            String::new()
        } else {
            String::from_utf8(hash.to_vec())?
        };
        info!("Loaded.");
        //**********TODO: kills is not 0***************
        Ok(Blockchain { tip: lasthash, db, kills:0 })
    }

     /// Saves the block into the blockchain
     pub fn add_block(&mut self, block: Block) -> Result<()> {
        let data = serialize(&block)?;
        //if this block is already exists, discard it.
        if let Some(_) = self.db.get(block.get_hash())? {
            return Ok(());
        }
        self.db.insert(block.get_hash(), data)?;

        let lastheight = self.get_best_height()?;
        if block.get_height() > lastheight {
            self.db.insert("LAST", block.get_hash().as_bytes())?;
            self.tip = block.get_hash();
            self.kills = self.kills + block.get_kills() as u128;
            self.db.flush()?;
        }
        Ok(())
    }


    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
        
        info!("mine a new block");

        for tx in &transactions {
            if !self.verify_transacton(tx)? {
                return Err(format_err!("ERROR: Invalid transaction"));
            }
        }

        
        let last_hash = self.db.get("LAST")?.unwrap();

        //this will start dogfight() to each of transaction with own agent.
        let newblock = Block::new_block(
            transactions,
            String::from_utf8(last_hash.to_vec())?,
            self.get_best_height()? + 1,
        )?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        Ok(newblock)
    }

    /// Iterator returns a BlockchainIterat
    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip.clone(),
            blockchain: &self,
        }
    }

    /// FindUTXO finds and returns all unspent transaction outputs
    pub fn find_utxo(&self) -> HashMap<String, TXOutputs> {
        let mut utxos: HashMap<String, TXOutputs> = HashMap::new();
        let mut spend_txos: HashMap<String, Vec<i32>> = HashMap::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for index in 0..tx.vout.len() {
                    if let Some(ids) = spend_txos.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    match utxos.get_mut(&tx.id) {
                        Some(v) => {
                            v.outputs.push(tx.vout[index].clone());
                        }
                        None => {
                            utxos.insert(
                                tx.id.clone(),
                                TXOutputs {
                                    outputs: vec![tx.vout[index].clone()],
                                },
                            );
                        }
                    }
                }

                if !tx.is_coinbase() {
                    for i in &tx.vin {
                        match spend_txos.get_mut(&i.txid) {
                            Some(v) => {
                                v.push(i.vout);
                            }
                            None => {
                                spend_txos.insert(i.txid.clone(), vec![i.vout]);
                            }
                        }
                    }
                }
            }
        }

        utxos
    }

    /// FindTransaction finds a transaction by its ID
    pub fn find_transacton(&self, id: &str) -> Result<Transaction> {
        for b in self.iter() {
            for tx in b.get_transaction() {
                if tx.id == id {
                    return Ok(tx.clone());
                }
            }
        }
        Err(format_err!("Transaction is not found"))
    }

    fn get_prev_txs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
        let mut prev_txs = HashMap::new();
        for vin in &tx.vin {
            let prev_tx = self.find_transacton(&vin.txid)?;
            prev_txs.insert(prev_tx.id.clone(), prev_tx);
        }
        Ok(prev_txs)
    }

    /// SignTransaction signs inputs of a Transaction
    pub fn sign_transacton(&self, tx: &mut Transaction, private_key: &[u8]) -> Result<()> {
        let prev_txs = self.get_prev_txs(tx)?;
        tx.sign(private_key, prev_txs)?;
        Ok(())
    }

    /// VerifyTransaction verifies transaction input signatures
    pub fn verify_transacton(&self, tx: &Transaction) -> Result<bool> {
        if tx.is_coinbase() {
            return Ok(true);
        }
        let prev_txs = self.get_prev_txs(tx)?;
        tx.verify(prev_txs)
    }

    /// GetBlock finds a block by its hash and returns it
    pub fn get_block(&self, block_hash: &str) -> Result<Block> {
        let data = self.db.get(block_hash)?.unwrap();
        let block = deserialize(&data.to_vec())?;
        Ok(block)
    }

    /// returns the height of the latest block,
    /// return u128::MAX if no blockchain is found
    pub fn get_best_height(&self) -> Result<u128> {
        let lasthash = if let Some(h) = self.db.get("LAST")? {
            h
        } else {
            return Ok(u128::MAX);
        };
        let last_data = self.db.get(lasthash)?.unwrap();
        let last_block: Block = deserialize(&last_data.to_vec())?;
        Ok(last_block.get_height())
    }

    pub fn get_kills(&self) -> u128 {
        self.kills
    }

    /// GetBlockHashes returns a list of hashes of all the blocks in the chain
    pub fn get_block_hashs(&self) -> Vec<String> {
        let mut list = Vec::new();
        for b in self.iter() {
            list.push(b.get_hash());
        }
        list
    }
}

impl<'a> Iterator for BlockchainIterator<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.blockchain.db.get(&self.current_hash) {
            return match encoded_block {
                Some(b) => {
                    if let Ok(block) = deserialize::<Block>(&b) {
                        self.current_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}

///Returns true if db_path points at an existing entity.
pub fn db_exist(db_path: &str) -> bool {
    std::path::Path::new(db_path).exists()
}