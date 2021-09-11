//! cli process

use super::*;
use crate::blockchain::*;
use crate::server::*;
use crate::transaction::*;
use crate::utxoset::*;
use crate::agent::*;
use bitcoincash_addr::Address;
use clap::{App, Arg};
use std::process::exit;
use std::io;
use std::{thread, time};
use std::env;

pub struct Cli {}

impl Cli {
    pub fn new() -> Cli {
        Cli {}
    }

    pub fn run(&mut self) -> Result<()> {
        info!("run app");
        let node_id = match env::var("NODE_ID") {
            Ok(id) => id,
            Err(err) => {
                println!("environment variable 'NODE_ID' should be set properly.\nuse 'export NODE_ID=xxxx' to set\n{:?}",err);
                panic!() 
            },
        };
        
        let matches = App::new("proof-of-kill-demo-version")
            .version("0.0.1")
            .author("ntswamp <nterheoid@gmail.com>")
            .about("a demonstration of PoK(Proof-of-Kill) consensus model")
            .subcommand(App::new("chain").about("print out current state of blockchain"))
            .subcommand(App::new("newagent").about("(re)create an agent to start collecting coins!"))
            .subcommand(App::new("agent").about("show agent stats"))
            .subcommand(App::new("newaddr").about("create an address for your agent"))
            .subcommand(App::new("addr").about("list all addresses held by your agent"))
            .subcommand(App::new("reindex").about("reindex UTXO"))
            .subcommand(
                App::new("startnode")
                    .about("start the node server")
                    .arg(Arg::from_usage("<port> 'the port server bind to locally'")),
            )
            .subcommand(
                App::new("startminer")
                    .about("start the minner server")
                    .arg(Arg::from_usage("<port> 'the port server bind to locally'"))
                    .arg(Arg::from_usage("<address> 'wallet address'")),
            )
            .subcommand(
                App::new("bal")
                    .about("get balance in the blockchain")
                    .arg(Arg::from_usage(
                        "<address> 'The address to get balance for'",
                    )),
            )
            .subcommand(App::new("initdb").about("initialize blockchain database").arg(
                Arg::from_usage("<address> 'The address to send genesis block reward to'"),
            ))
            .subcommand(
                App::new("send")
                    .about("send in the blockchain")
                    .arg(Arg::from_usage("<from> 'Source Address'"))
                    .arg(Arg::from_usage("<to> 'Destination Address'"))
                    .arg(Arg::from_usage("<amount> 'Amount To Send'"))
                    .arg(Arg::from_usage(
                        "-m --mine 'Let The From Address Mine Immediately'",
                    )),
            )
            .get_matches();

        if let Some(ref matches) = matches.subcommand_matches("bal") {
            if let Some(address) = matches.value_of("address") {
                let balance = cmd_bal(address)?;
                println!("Balance: {}\n", balance);
            }
        } else if let Some(_) = matches.subcommand_matches("newagent") {
            println!("address: {}", cmd_newagent()?);
        } else if let Some(_) = matches.subcommand_matches("agent") {
            cmd_agent()?;
        } else if let Some(_) = matches.subcommand_matches("newaddr") {
            cmd_newaddr()?;
        } else if let Some(_) = matches.subcommand_matches("addr") {
            cmd_addr()?;
        } else if let Some(_) = matches.subcommand_matches("chain") {
            cmd_chain()?;
        } else if let Some(_) = matches.subcommand_matches("reindex") {
            let count = cmd_reindex()?;
            println!("Done! There are {} transactions in the UTXO set.", count);
        } else if let Some(ref matches) = matches.subcommand_matches("initdb") {
            if let Some(address) = matches.value_of("address") {
                cmd_init_db(address)?;
            }
        } else if let Some(ref matches) = matches.subcommand_matches("send") {
            let from = if let Some(address) = matches.value_of("from") {
                address
            } else {
                println!("from not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let to = if let Some(address) = matches.value_of("to") {
                address
            } else {
                println!("to not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let amount: i32 = if let Some(amount) = matches.value_of("amount") {
                amount.parse()?
            } else {
                println!("amount in send not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            if matches.is_present("mine") {
                cmd_send(from, to, amount, true)?;
            } else {
                cmd_send(from, to, amount, false)?;
            }
        } else if let Some(ref matches) = matches.subcommand_matches("startnode") {
            if let Some(port) = matches.value_of("port") {
                println!("Start node...");
                let bc = Blockchain::load(&node_id)?;
                let utxo_set = UTXOSet { blockchain: bc };
                let server = Server::new(port, "", utxo_set)?;
                server.start_server()?;
            }
        } else if let Some(ref matches) = matches.subcommand_matches("startminer") {
            let address = if let Some(address) = matches.value_of("address") {
                address
            } else {
                println!("address not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let port = if let Some(port) = matches.value_of("port") {
                port
            } else {
                println!("port not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            println!("Start miner node...");
            let bc = Blockchain::load(&node_id)?;
            let utxo_set = UTXOSet { blockchain: bc };
            let server = Server::new(port, address, utxo_set)?;
            server.start_server()?;
        }

        Ok(())
    }
}

fn cmd_send(from: &str, to: &str, amount: i32, mine_now: bool) -> Result<()> {
    let node_id = env::var("NODE_ID").unwrap();
    let bc = Blockchain::load(&node_id)?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let agent = Agent::load().unwrap();
    let from_keypair = agent.get_keypair_by_address(from).unwrap();
    let tx = Transaction::send(from_keypair, to, amount, &utxo_set,agent.get_build().clone())?;
    if mine_now {
        let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward!"))?;
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;

        utxo_set.update(&new_block)?;
    } else {
        Server::send_transaction(&tx, utxo_set)?;
    }

    println!("success!");
    Ok(())
}

fn cmd_newagent() -> Result<String> {
    let node_id = env::var("NODE_ID").unwrap();
    let agent_path = "data_".to_owned() + &node_id + "/agent";

    println!("this operation will remove current agent. continue?(y/n)");
    let mut yesno = String::new();
    io::stdin()
    .read_line(&mut yesno)
    .expect("please enter y or n");

    if yesno.trim() == "n" {
        return Ok("Creation Canceled".to_owned());
    }
    
    
    std::fs::remove_dir_all(&agent_path).ok();

    loop {
        let mut name = String::new();
        let mut class = String::new();
        let mut weapon = String::new();

        println!("\nPlease name your agent:");
        io::stdin()
            .read_line(&mut name)
            .expect("failed to read name");
        name = name.trim().to_owned();

        println!();
        println!();
        println!();
        
        println!("Welcome to the world of PoK, {}.",&name);
        println!("Now tell me the *class* of your agent, by enter a number:");
        println!("#1 Warrior");
        println!("#2 Mage");
        println!("#3 Archer");

        io::stdin()
            .read_line(&mut class)
            .expect("failed to read class");

        class = match class.trim().parse() {
            Ok(num) => {
                match num {
                    1 => "Warrior".to_owned(),
                    2 => "Mage".to_owned(),
                    3 => "Archer".to_owned(),
                    _ => panic!()
                }
            },
            Err(_) => panic!(),
        };

        println!();
        println!();
        println!();

        println!("Good. Your agent looks like an experienced {}.",&class);
        println!("Now pick a weapon for your agent:");

        match &*class {
            "Warrior" => {
                println!("#1 Axe");
                println!("#2 Warhammer");
                io::stdin()
                .read_line(&mut weapon)
                .expect("failed to read weapon");
                weapon = match weapon.trim().parse() {
                    Ok(num) => {
                        match num {
                            1 => "Axe".to_owned(),
                            2 => "Warhammer".to_owned(),
                            _ => panic!(),
                        }
                    },
                    Err(_) => panic!(),
                };
            },
            "Mage" => {
                println!("#1 Wand");
                println!("#2 Sword");
                io::stdin()
                .read_line(&mut weapon)
                .expect("failed to read weapon");
                weapon = match weapon.trim().parse() {
                    Ok(num) => {
                        match num {
                            1 => "Wand".to_owned(),
                            2 => "Sword".to_owned(),
                            _ => panic!(),
                        }
                    },
                    Err(_) => panic!(),
                };
            },
            "Archer" => {
                println!("#1 Longbow");
                println!("#2 Crossbow");
                io::stdin()
                .read_line(&mut weapon)
                .expect("failed to read weapon");
                weapon = match weapon.trim().parse() {
                    Ok(num) => {
                        match num {
                            1 => "Longbow".to_owned(),
                            2 => "Crossbow".to_owned(),
                            _ => panic!(),
                        }
                    },
                    Err(_) => panic!(),
                };

            },
            _ => {}
        }


        println!();
        println!();
        println!();

        println!("{}? this is a good pick.",&weapon);
        thread::sleep(time::Duration::from_secs(1));

        println!("At the end of the day, this is what your agent looks like.");
        println!("Name   :  {}",&name);
        println!("Class  :  {}",&class);
        println!("Weapon :  {}",&weapon);
        println!();
        println!("Are you satisfied with this agent?: (y/n)");

        let mut yesno = String::new();
        io::stdin()
        .read_line(&mut yesno)
        .expect("please enter y or n");

        if yesno.trim() == "n" {
            continue;
        }

        println!();
        println!();
        println!();
        
        let build = Build::new(name,class,weapon);

        let mut agent = Agent::new(build,&node_id).unwrap();
        let address = agent.generate_address();
        agent.save()?;

        println!("Congratulation, you made a wise choice.");
        println!("Use Command `agent` to greet your agent.");
        
        return Ok(address)
    }
}

fn cmd_agent()-> Result<()> {
    match Agent::load() {
        Ok(agent) => {
            println!("agent name: {:?}", agent.get_build().name);
            println!("agent class: {:?}", agent.get_build().class);
            println!("agent's weapon: {:?}", agent.get_build().weapon);
            return Ok(());
        },
        Err(err) => return Err(err),
    }
}

fn cmd_newaddr() ->Result<()> {
    let mut agent = Agent::load().unwrap();
    let address = agent.generate_address();
    agent.save()?;
    println!("new address generated:\n{}",&address);
    Ok(())
}

fn cmd_reindex() -> Result<i32> {
    let node_id = env::var("NODE_ID").unwrap();
    let bc = Blockchain::load(&node_id)?;
    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    utxo_set.count_transactions()
}

fn cmd_init_db(address: &str) -> Result<()> {
    let node_id = env::var("NODE_ID").unwrap();
    let address = String::from(address);
    let bc = Blockchain::init(address,&node_id)?;

    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    println!("blockchain initialized");
    Ok(())
}

fn cmd_bal(address: &str) -> Result<i32> {
    let node_id = env::var("NODE_ID").unwrap();
    let pub_key_hash = Address::decode(address).unwrap().body;
    let bc = match Blockchain::load(&node_id) {
        Ok(bc) => bc,
        Err(_) => {
            return Ok(0);
        }
    };
  
    let utxo_set = UTXOSet { blockchain: bc };
    let utxos = utxo_set.find_utxo(&pub_key_hash)?;

    let mut balance = 0;
    for out in utxos.outputs {
        balance += out.value;
    }
    Ok(balance)
}

fn cmd_chain() -> Result<()> {
    let node_id = env::var("NODE_ID").unwrap();
    let bc = Blockchain::load(&node_id)?;
    for b in bc.iter() {
        println!("{:#?}", b);
    }
    Ok(())
}

fn cmd_addr() -> Result<()> {
    let agent = Agent::load().unwrap();
    let addresses = agent.get_all_addresses();
    println!("addresses: ");
    for ad in addresses {
        println!("{}", ad);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_locally() {
        let addr1 = cmd_newagent().unwrap();
        let addr2 = cmd_newagent().unwrap();
        cmd_init_db(&addr1).unwrap();

        let b1 = cmd_bal(&addr1).unwrap();
        let b2 = cmd_bal(&addr2).unwrap();
        assert_eq!(b1, 10);
        assert_eq!(b2, 0);

        cmd_send(&addr1, &addr2, 5, true).unwrap();

        let b1 = cmd_bal(&addr1).unwrap();
        let b2 = cmd_bal(&addr2).unwrap();
        assert_eq!(b1, 15);
        assert_eq!(b2, 5);

        cmd_send(&addr2, &addr1, 15, true).unwrap_err();
        let b1 = cmd_bal(&addr1).unwrap();
        let b2 = cmd_bal(&addr2).unwrap();
        assert_eq!(b1, 15);
        assert_eq!(b2, 5);
    }
}
