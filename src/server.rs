//! server of Blockchain

use super::*;
use crate::block::*;
use crate::transaction::*;
use crate::utxoset::*;
use bincode::{deserialize, serialize};
use failure::format_err;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::*;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Message {
    Addr(Vec<String>),
    Version(Versionmsg),
    Tx(Txmsg),
    GetData(GetDatamsg),
    GetBlock(GetBlocksmsg),
    Inv(Invmsg),
    Block(Blockmsg),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Blockmsg {
    from_ip: String,
    block: Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetBlocksmsg {
    from_ip: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetDatamsg {
    from_ip: String,
    kind: String,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Invmsg {
    from_ip: String,
    kind: String,
    items: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Txmsg {
    from_ip: String,
    transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Versionmsg {
    from_ip: String,
    version: i32,
    //current height of the blockchain
    best_height: u128,
    //total kills included by current blockchain
    kills: u128,
}

pub struct Server {
    node_ip: String,
    mining_address: String,
    inner: Arc<Mutex<ServerInner>>,
}

struct ServerInner {
    known_nodes: HashSet<String>,
    utxo: UTXOSet,
    blocks_in_transit: Vec<String>,
    mempool: HashMap<String, Transaction>,
}

const CENTRAL_NODE: &str = "localhost:3333";
const CMD_LEN: usize = 12;
const VERSION: i32 = 1;

impl Server {
    pub fn new(port: &str, miner_address: &str, utxo: UTXOSet) -> Result<Server> {
        let mut node_set = HashSet::new();
        node_set.insert(String::from(CENTRAL_NODE));
        Ok(Server {
            node_ip: String::from("localhost:") + port,
            mining_address: miner_address.to_string(),
            inner: Arc::new(Mutex::new(ServerInner {
                known_nodes: node_set,
                utxo,
                blocks_in_transit: Vec::new(),
                mempool: HashMap::new(),
            })),
        })
    }

    pub fn start(&self) -> Result<()> {
        let server1 = Server {
            node_ip: self.node_ip.clone(),
            mining_address: self.mining_address.clone(),
            inner: Arc::clone(&self.inner),
        };
        info!(
            "Start server at {}, collect coins by address: {}",
            &self.node_ip, &self.mining_address
        );

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            //if server1 has not any existing blockchain yet
            if server1.get_best_height()? == u128::MAX {
                //server1.request_blocks()
                //request blocks from localhost:3333
                server1.request_blocks()
            } else {
                server1.send_version(CENTRAL_NODE)
            }
        });

        let listener = TcpListener::bind(&self.node_ip).unwrap();
        info!("Server listen...");

        for stream in listener.incoming() {
            let stream = stream?;
            let server1 = Server {
                node_ip: self.node_ip.clone(),
                mining_address: self.mining_address.clone(),
                inner: Arc::clone(&self.inner),
            };
            thread::spawn(move || server1.handle_connection(stream));
        }

        Ok(())
    }

    pub fn send_transaction(tx: &Transaction, utxoset: UTXOSet) -> Result<()> {
        let server = Server::new("7000", "", utxoset)?;
        server.send_tx(CENTRAL_NODE, tx)?;
        Ok(())
    }

    /* ------------------- helper functions for Server ----------------------------------*/

    fn remove_node(&self, addr: &str) {
        self.inner.lock().unwrap().known_nodes.remove(addr);
    }

    fn add_nodes(&self, addr: &str) {
        self.inner
            .lock()
            .unwrap()
            .known_nodes
            .insert(String::from(addr));
    }

    fn get_known_nodes(&self) -> HashSet<String> {
        self.inner.lock().unwrap().known_nodes.clone()
    }

    fn is_node_known(&self, addr: &str) -> bool {
        self.inner.lock().unwrap().known_nodes.get(addr).is_some()
    }

    fn replace_in_transit(&self, hashs: Vec<String>) {
        let bit = &mut self.inner.lock().unwrap().blocks_in_transit;
        bit.clone_from(&hashs);
    }

    fn get_in_transit(&self) -> Vec<String> {
        self.inner.lock().unwrap().blocks_in_transit.clone()
    }

    fn get_mempool_tx(&self, addr: &str) -> Option<Transaction> {
        match self.inner.lock().unwrap().mempool.get(addr) {
            Some(tx) => Some(tx.clone()),
            None => None,
        }
    }

    fn get_mempool(&self) -> HashMap<String, Transaction> {
        self.inner.lock().unwrap().mempool.clone()
    }

    fn insert_mempool(&self, tx: Transaction) {
        self.inner.lock().unwrap().mempool.insert(tx.id.clone(), tx);
    }

    fn clear_mempool(&self) {
        self.inner.lock().unwrap().mempool.clear()
    }

    fn get_best_height(&self) -> Result<u128> {
        self.inner.lock().unwrap().utxo.blockchain.get_best_height()
    }

    fn get_kills(&self) -> u128 {
        self.inner.lock().unwrap().utxo.blockchain.get_kills()
    }

    fn get_block_hashs(&self) -> Vec<String> {
        self.inner.lock().unwrap().utxo.blockchain.get_block_hashs()
    }

    fn get_block(&self, block_hash: &str) -> Result<Block> {
        self.inner
            .lock()
            .unwrap()
            .utxo
            .blockchain
            .get_block(block_hash)
    }

    fn verify_tx(&self, tx: &Transaction) -> Result<bool> {
        self.inner
            .lock()
            .unwrap()
            .utxo
            .blockchain
            .verify_transacton(tx)
    }

    fn add_block(&self, block: Block) -> Result<()> {
        self.inner.lock().unwrap().utxo.blockchain.add_block(block)
    }

    fn mine_block(&self, txs: Vec<Transaction>) -> Result<Block> {
        self.inner.lock().unwrap().utxo.blockchain.mine_block(txs)
    }

    fn utxo_reindex(&self) -> Result<()> {
        self.inner.lock().unwrap().utxo.reindex()
    }

    /* -----------------------------------------------------*/

    fn send_data(&self, addr: &str, data: &[u8]) -> Result<()> {
        if addr == &self.node_ip {
            return Ok(());
        }
        let mut stream = match TcpStream::connect(addr) {
            Ok(s) => s,
            Err(_) => {
                self.remove_node(addr);
                return Ok(());
            }
        };

        stream.write(data)?;

        info!("data send successfully");
        Ok(())
    }

    fn request_blocks(&self) -> Result<()> {
        for node in self.get_known_nodes() {
            self.request_get_blocks(&node)?
        }
        Ok(())
    }

    fn send_block(&self, addr: &str, b: &Block) -> Result<()> {
        info!("send block data to: {} block hash: {}", addr, b.get_hash());
        let data = Blockmsg {
            from_ip: self.node_ip.clone(),
            block: b.clone(),
        };
        let data = serialize(&(cmd_to_byte("block"), data))?;
        self.send_data(addr, &data)
    }

    fn send_addr(&self, addr: &str) -> Result<()> {
        info!("send address info to: {}", addr);
        let nodes = self.get_known_nodes();
        let data = serialize(&(cmd_to_byte("addr"), nodes))?;
        self.send_data(addr, &data)
    }

    fn send_inv(&self, addr: &str, kind: &str, items: Vec<String>) -> Result<()> {
        info!(
            "send inv message to: {} kind: {} data: {:?}",
            addr, kind, items
        );
        let data = Invmsg {
            from_ip: self.node_ip.clone(),
            kind: kind.to_string(),
            items,
        };
        let data = serialize(&(cmd_to_byte("inv"), data))?;
        self.send_data(addr, &data)
    }

    //addr = localhost:3333 by default
    fn request_get_blocks(&self, addr: &str) -> Result<()> {
        info!("send get blocks message to: {}", addr);
        let data = GetBlocksmsg {
            from_ip: self.node_ip.clone(),
        };
        let data = serialize(&(cmd_to_byte("getblocks"), data))?;
        self.send_data(addr, &data)
    }

    fn request_get_data(&self, addr: &str, kind: &str, id: &str) -> Result<()> {
        info!(
            "send get data message to: {} kind: {} id: {}",
            addr, kind, id
        );
        let data = GetDatamsg {
            from_ip: self.node_ip.clone(),
            kind: kind.to_string(),
            id: id.to_string(),
        };
        let data = serialize(&(cmd_to_byte("getdata"), data))?;
        self.send_data(addr, &data)
    }

    pub fn send_tx(&self, addr: &str, tx: &Transaction) -> Result<()> {
        info!("send tx to: {} txid: {}", addr, &tx.id);
        let data = Txmsg {
            from_ip: self.node_ip.clone(),
            transaction: tx.clone(),
        };
        let data = serialize(&(cmd_to_byte("tx"), data))?;
        self.send_data(addr, &data)
    }

    fn send_version(&self, to_ip: &str) -> Result<()> {
        info!("send version info to: {}", to_ip);
        let data = Versionmsg {
            from_ip: self.node_ip.clone(),
            best_height: self.get_best_height()?,
            version: VERSION,
            kills: self.get_kills(),
        };
        let data = serialize( &(cmd_to_byte("version"), data) )?;
        self.send_data(to_ip, &data)
    }

    /// handle_version defines forking strategy
    fn handle_version(&self, msg: Versionmsg) -> Result<()> {
        info!("receive version msg: {:#?}", msg);
        /* old version backup
        let my_best_height = self.get_best_height()?;
        if my_best_height < msg.best_height {
            self.request_get_blocks(&msg.from_ip)?;
        } else if my_best_height > msg.best_height {
            self.send_version(&msg.from_ip)?;
        }
        */
        //always picking the most winning version
        if self.get_kills() < msg.kills {
            self.request_get_blocks(&msg.from_ip)?;
        } else if self.get_kills() > msg.kills {
            self.send_version(&msg.from_ip)?;
        }

        self.send_addr(&msg.from_ip)?;

        if !self.is_node_known(&msg.from_ip) {
            self.add_nodes(&msg.from_ip);
        }
        Ok(())
    }

    fn handle_addr(&self, msg: Vec<String>) -> Result<()> {
        info!("receive address msg: {:#?}", msg);
        for node in msg {
            self.add_nodes(&node);
        }
        /*
        //self.request_blocks()?;
        for node in self.get_known_nodes() {
            self.request_get_blocks(&node)?
        }
        Ok(())
        */
        Ok(())
    }

    fn handle_block(&self, msg: Blockmsg) -> Result<()> {
        info!(
            "receive block msg: {}, {}",
            msg.from_ip,
            msg.block.get_hash()
        );
        self.add_block(msg.block)?;

        let mut in_transit = self.get_in_transit();
        if in_transit.len() > 0 {
            let block_hash = &in_transit[0];
            self.request_get_data(&msg.from_ip, "block", block_hash)?;
            in_transit.remove(0);
            self.replace_in_transit(in_transit);
        } else {
            self.utxo_reindex()?;
        }

        Ok(())
    }

    fn handle_inv(&self, msg: Invmsg) -> Result<()> {
        info!("receive inv msg: {:#?}", msg);
        if msg.kind == "block" {
            let block_hash = &msg.items[0];
            self.request_get_data(&msg.from_ip, "block", block_hash)?;

            let mut new_in_transit = Vec::new();
            for b in &msg.items {
                if b != block_hash {
                    new_in_transit.push(b.clone());
                }
            }
            self.replace_in_transit(new_in_transit);
        } else if msg.kind == "tx" {
            let txid = &msg.items[0];
            match self.get_mempool_tx(txid) {
                Some(tx) => {
                    if tx.id.is_empty() {
                        self.request_get_data(&msg.from_ip, "tx", txid)?
                    }
                }
                None => self.request_get_data(&msg.from_ip, "tx", txid)?,
            }
        }
        Ok(())
    }

    fn handle_get_blocks(&self, msg: GetBlocksmsg) -> Result<()> {
        info!("receive get blocks msg: {:#?}", msg);
        let block_hashs = self.get_block_hashs();
        self.send_inv(&msg.from_ip, "block", block_hashs)?;
        Ok(())
    }

    fn handle_get_data(&self, msg: GetDatamsg) -> Result<()> {
        info!("receive get data msg: {:#?}", msg);
        if msg.kind == "block" {
            let block = self.get_block(&msg.id)?;
            self.send_block(&msg.from_ip, &block)?;
        } else if msg.kind == "tx" {
            let tx = self.get_mempool_tx(&msg.id).unwrap();
            self.send_tx(&msg.from_ip, &tx)?;
        }
        Ok(())
    }

    fn handle_tx(&self, msg: Txmsg) -> Result<()> {
        info!("receive tx msg: {} {}", msg.from_ip, &msg.transaction.id);
        self.insert_mempool(msg.transaction.clone());

        let known_nodes = self.get_known_nodes();
        if self.node_ip == CENTRAL_NODE {
            for node in known_nodes {
                if node != self.node_ip && node != msg.from_ip {
                    self.send_inv(&node, "tx", vec![msg.transaction.id.clone()])?;
                }
            }
        } else {
            let mut mempool = self.get_mempool();
            debug!("Current mempool: {:#?}", &mempool);
            if mempool.len() >= 1 && !self.mining_address.is_empty() {
                loop {
                    let mut txs = Vec::new();

                    for (_, tx) in &mempool {
                        if self.verify_tx(tx)? {
                            txs.push(tx.clone());
                        }
                    }

                    if txs.is_empty() {
                        return Ok(());
                    }

                    let cbtx =
                        Transaction::new_coinbase(self.mining_address.clone(), String::new())?;
                    txs.push(cbtx);

                    for tx in &txs {
                        mempool.remove(&tx.id);
                    }

                    let new_block = self.mine_block(txs)?;
                    self.utxo_reindex()?;

                    for node in self.get_known_nodes() {
                        if node != self.node_ip {
                            self.send_inv(&node, "block", vec![new_block.get_hash()])?;
                        }
                    }

                    if mempool.len() == 0 {
                        break;
                    }
                }
                self.clear_mempool();
            }
        }

        Ok(())
    }

    fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let mut buffer = Vec::new();
        let count = stream.read_to_end(&mut buffer)?;
        info!("Accept request: length {}", count);

        let cmd = byte_to_cmd(&buffer)?;

        match cmd {
            Message::Addr(data) => self.handle_addr(data)?,
            Message::Block(data) => self.handle_block(data)?,
            Message::Inv(data) => self.handle_inv(data)?,
            Message::GetBlock(data) => self.handle_get_blocks(data)?,
            Message::GetData(data) => self.handle_get_data(data)?,
            Message::Tx(data) => self.handle_tx(data)?,
            Message::Version(data) => self.handle_version(data)?,
        }

        Ok(())
    }
}

fn cmd_to_byte(cmd: &str) -> [u8; CMD_LEN] {
    let mut data = [0; CMD_LEN];
    for (i, d) in cmd.as_bytes().iter().enumerate() {
        data[i] = *d;
    }
    data
}

fn byte_to_cmd(byte: &[u8]) -> Result<Message> {
    let cmd_byte = &byte[..CMD_LEN];
    let data = &byte[CMD_LEN..];

    let mut cmd = Vec::new();
    for b in cmd_byte {
        if 0 as u8 != *b {
            cmd.push(*b);
        }
    }
    info!("cmd: {}", String::from_utf8(cmd.clone())?);

    if cmd == "addr".as_bytes() {
        let data: Vec<String> = deserialize(data)?;
        Ok(Message::Addr(data))
    } else if cmd == "block".as_bytes() {
        let data: Blockmsg = deserialize(data)?;
        Ok(Message::Block(data))
    } else if cmd == "inv".as_bytes() {
        let data: Invmsg = deserialize(data)?;
        Ok(Message::Inv(data))
    } else if cmd == "getblocks".as_bytes() {
        let data: GetBlocksmsg = deserialize(data)?;
        Ok(Message::GetBlock(data))
    } else if cmd == "getdata".as_bytes() {
        let data: GetDatamsg = deserialize(data)?;
        Ok(Message::GetData(data))
    } else if cmd == "tx".as_bytes() {
        let data: Txmsg = deserialize(data)?;
        Ok(Message::Tx(data))
    } else if cmd == "version".as_bytes() {
        let data: Versionmsg = deserialize(data)?;
        Ok(Message::Version(data))
    } else {
        Err(format_err!("Unknown command in the server"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::blockchain::*;
    use crate::agent::*;


    #[test]
    fn test_cmd() {
        let build:Build = Build::new (
            "Tim".to_owned(),
            "Warrior".to_owned(),
            "Axe".to_owned(),
        );
        let mut agent = Agent::new(build,"test").unwrap();
        let wa1 = agent.generate_address();
        let bc = Blockchain::init(wa1,"test").unwrap();
        let utxo_set = UTXOSet { blockchain: bc };
        let server = Server::new("7878", "localhost:3001", utxo_set).unwrap();

        let vmsg = Versionmsg {
            from_ip: server.node_ip.clone(),
            best_height: server.get_best_height().unwrap(),
            version: VERSION,
            kills:server.get_kills(),
        };
        let data = serialize(&(cmd_to_byte("version"), vmsg.clone())).unwrap();
        if let Message::Version(v) = byte_to_cmd(&data).unwrap() {
            assert_eq!(v, vmsg);
        } else {
            panic!("wrong!");
        }
    }
}
