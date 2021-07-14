use ring::digest::{Context, SHA256};
use rand_seeder::{Seeder};
use rand_pcg::Pcg64;
use std::io::{BufReader, Read};

//produce a random number generator from SHA256 hashes
fn generator_from_sha256<R: Read>(mut reader: R) -> Result<Pcg64,std::io::Error> {
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
    use rand::Rng;
    use std::fs::File;
    #[test]
    fn generator_from_sha256_test()-> Result<(),std::io::Error> {
        
        
        let path = "target/debug/bin.d";
    
        let input = File::open(path)?;
        let reader = BufReader::new(input);
        let mut rng = generator_from_sha256(reader)?;
    
        println!("random number from sha256 is {}",rng.gen_range(-5..=5));
    
        Ok(())
    }
}
