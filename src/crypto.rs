use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use rand_core::SeedableRng;
use rand_seeder::{Seeder, SipHasher};
use rand_pcg::Pcg64;
use rand::Rng;

use std::fs::File;
use std::io::{BufReader, Read, Write};

//produce a random number generator from SHA256 hashes
fn sha256_generator<R: Read>(mut reader: R) -> Result<Pcg64,std::io::Error> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    let rng: Pcg64 = Seeder::from(context.finish().as_ref()).make_rng();
    Ok(rng)
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sha256_generator_test()-> Result<(),std::io::Error> {
        let path = "file.txt";
    
        let mut output = File::create(path)?;
        write!(output, "aaaasdfsdWe will generate a digest of this text")?;
    
        let input = File::open(path)?;
        let reader = BufReader::new(input);
        let mut rng = sha256_generator(reader)?;
    
        println!("random number from sha256 is {}",rng.gen_range(-5..=5));
    
        Ok(())
    }
}
