use core::fmt;

use std::collections::HashMap;

use hex_literal::hex;

use sha2::{Sha256, Sha512, Digest}; 
use sha2::digest::typenum::Unsigned;

use aes_gcm_siv::{
    aead::{Aead, KeyInit, Key, OsRng},
    Aes256GcmSiv, Nonce
};

use serde::{Serialize, Deserialize};

pub mod passtable{
    use super::*;

    pub use Error::*;
    #[derive(Debug, PartialEq)]
    pub enum Error {
        PassExists,
        PassNotFound, 
        IncorrectPass,
        AES
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::PassExists => f.write_str("Password already exists"),
                Self::PassNotFound => f.write_str("Password not found"),
                Self::IncorrectPass => f.write_str("Incorrect password"),
                Self::AES => f.write_str("AES Error")
            }
        }
    }

    pub type PassHasher = Sha256;
    pub type PassCypher = Aes256GcmSiv;

    pub fn nonce_from_password<D: Digest>(password: &str) -> Nonce {
        let mut hasher = D::new();
        hasher.update(password.as_bytes());
        hasher.update(b"nonce");
        *Nonce::from_slice(&hasher.finalize()[..12])
    }

    pub fn key_from_password<D: Digest, K : KeyInit>(password: &str) -> Key<K> {
        let mut hasher = D::new();
        hasher.update(password.as_bytes());
        hasher.update(b"password");
        let len : usize = K::KeySize::to_usize();
        Key::<K>::from_slice(&hasher.finalize()[..len]).clone()
    }

    pub fn encrypt(message : &[u8], password : &str) -> Result<Vec<u8>, aes_gcm_siv::Error>{
        let key = key_from_password::<PassHasher, PassCypher>(password);
        let cipher = PassCypher::new(&key);
        let nonce = nonce_from_password::<PassHasher>(password);
        
        cipher.encrypt(&nonce, message)
    }

    pub fn decrypt(message : &[u8], password : &str) -> Result<Vec<u8>, aes_gcm_siv::Error>{
        let key = key_from_password::<PassHasher, PassCypher>(password);
        let cipher = PassCypher::new(&key);
        let nonce = nonce_from_password::<PassHasher>(password);
        
        cipher.decrypt(&nonce, message)
    }
    
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct PassTable {
        passwords: HashMap<String, Vec<u8>>
    }
    
    impl PassTable {
        pub fn new() -> Self {
            PassTable { passwords: HashMap::new() }
        }
    
        fn get_cypher(&self, name: &str) -> Option<&Vec<u8>> {
            self.passwords.get(name)
        }

        fn add_cypher(&mut self, name: String, cypher: Vec<u8>) {
            self.passwords.insert(name, cypher);
        }

        pub fn get_password(&self, name: &str, key: &str) -> Result<String, passtable::Error> {
            let cypher = self.get_cypher(name).ok_or(passtable::PassNotFound)?;
            let password = passtable::decrypt(cypher, key).or(Err(passtable::IncorrectPass))?;
            String::from_utf8(password).or(Err(passtable::AES))
        }

        pub fn add_password(&mut self, name: &str, password: &str, key: &str) -> Result<(), passtable::Error>{
            if self.passwords.contains_key(name) {return Err(passtable::PassExists)}
            let cypher = passtable::encrypt(password.as_bytes(), key).or(Err(passtable::AES))?;
            self.add_cypher(String::from(name), cypher);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use super::passtable::*;

    #[test]
    fn passtable_test() -> Result<(), passtable::Error>{
        let message = "super secret message";
        let password = "super secret password";
        let mut pt = PassTable::new();
        let name = String::from("test");
        pt.add_password(&name, message, password)?;
        let pass = pt.get_password(&name, password)?;
        assert_eq!(pass, message);
        Ok(())
    }

    #[test]
    fn passtable_test2() -> Result<(), passtable::Error>{
        use random_string::generate;
        let charset = "1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let data: Vec<(String, String, String)> = (0..10).map(|x| (x.to_string(), generate(100, charset), generate(50, charset))).collect();
        let mut pt = PassTable::new();
        for (n, m, p) in &data{
            pt.add_password(n, m, p)?;
        }

        for (n, m, p) in &data{
            let pass = pt.get_password(n, p)?;
            assert_eq!(&pass, m);
        }
        Ok(())
    }

    #[test]
    fn incorrect_password_passtable_test() -> Result<(), passtable::Error>{
        let message = "super secret message";
        let password = "super secret password";
        let mut pt = PassTable::new();
        let name = String::from("test");
        pt.add_password(&name, message, password)?;
        let pass = pt.get_password(&name, "bebra");
        assert!(pass.is_err_and(|x| x == passtable::IncorrectPass));
        Ok(())
    }
    #[test]
    fn not_found_passtable_test() -> Result<(), passtable::Error>{
        let message = "super secret message";
        let password = "super secret password";
        let mut pt = PassTable::new();
        let name = String::from("test");
        pt.add_password(&name, message, password)?;
        let pass = pt.get_password(&"test2".to_string(), "bebra");
        assert!(pass.is_err_and(|x| if let passtable::PassNotFound = x {true} else {false}));
        Ok(())
    }

    #[test]
    fn alredy_exists_passtable_test() -> Result<(), passtable::Error>{
        let message = "super secret message";
        let password = "super secret password";
        let mut pt = PassTable::new();
        let name = String::from("test");
        pt.add_password(&name, message, password)?;
        let res = pt.add_password(&name, message, password);
        assert!(res.is_err_and(|x| if let passtable::PassExists = x {true} else {false}));
        Ok(())
    }

    #[test]
    fn password_encrypt_test() -> Result<(), aes_gcm_siv::Error>{
        let password = "super secret password";
        let message = Vec::from(b"Hello world!");
        let cypher = passtable::encrypt(&message, password)?;
        let message2 = passtable::decrypt(&cypher, password)?;
        assert_eq!(&message, &message2);
        Ok(())
    }

    #[test]
    fn incorrect_password_encrypt_test2() -> Result<(), aes_gcm_siv::Error>{
        let password = "super secret password";
        let password2 = "super not secret password";
        let message = Vec::from(b"Hello world!");
        let cypher = passtable::encrypt(&message, password)?;
        let message2 = passtable::decrypt(&cypher, password2);
        assert!(message2.is_err());
        Ok(())
    }

    #[test]
    fn serialize_test() -> Result<(), passtable::Error>{
        let mut pt = PassTable::new();
        pt.add_password("pass1", "test1", "password1")?;
        pt.add_password("pass2", "test2", "password2")?;
        pt.add_password("pass3", "test3", "password3")?;

        let encoded: Vec<u8> = bincode::serialize(&pt).unwrap();
        println!("{:?}", encoded);
        let decoded: PassTable = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(pt, decoded);
        let pass = decoded.get_password("pass2", "password2")?;
        assert_eq!(pass, "test2");
        Ok(())
    }

    #[test]
    #[ignore]
    fn simple_serialize_test(){
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Entity {
            x: f32,
            y: f32,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct World(Vec<Entity>);

        let world = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);
        let encoded: Vec<u8> = bincode::serialize(&world).unwrap();
        //println!("{:?}", encoded);
        // 8 bytes for the length of the vector, 4 bytes per float.
        assert_eq!(encoded.len(), 8 + 4 * 4);
        let decoded: World = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(world, decoded);
    }

    #[test]
    #[ignore]
    fn create_nonce_test() -> Result<(), aes_gcm_siv::Error>{
        let key = Aes256GcmSiv::generate_key(&mut OsRng);
        //println!("{:?}", key);
        let cipher = Aes256GcmSiv::new(&key);
        let nonce = &passtable::nonce_from_password::<Sha256>("xd");
        
        let ciphertext = cipher.encrypt(nonce, "plaintext message".as_ref())?;
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;
        assert_eq!(&plaintext, b"plaintext message");
        //println!("{:?}", nonce);
        Ok(())
    }

    #[test]
    #[ignore]
    fn create_pass_test() -> Result<(), aes_gcm_siv::Error>{
        let password = "xd";
        let key = passtable::key_from_password::<Sha256, Aes256GcmSiv>(password);
        //println!("{:?}", key);
        let cipher = Aes256GcmSiv::new(&key);
        let nonce = &passtable::nonce_from_password::<Sha256>(password);
        
        let ciphertext = cipher.encrypt(nonce, b"plaintext message".as_ref())?;
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;
        assert_eq!(&plaintext, b"plaintext message");
        Ok(())
    }

    #[test]
    #[ignore]
    fn simple_cipher_test() -> Result<(), aes_gcm_siv::Error>{
        let key = Aes256GcmSiv::generate_key(&mut OsRng);
        //println!("{:?}", key);
        let cipher = Aes256GcmSiv::new(&key);
        let nonce = Nonce::from_slice(b"unique nonce"); // 96-bits; unique per message
        let ciphertext = cipher.encrypt(nonce, b"plaintext message".as_ref())?;
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;
        assert_eq!(&plaintext, b"plaintext message");
        Ok(())
    }

    #[test]
    #[ignore]
    fn simple_hash_test() {
        // create a Sha256 object
        let mut hasher = Sha256::new();

        // write input message
        hasher.update(b"hello world");

        // read hash digest and consume hasher
        let result = hasher.finalize();

        assert_eq!(result[..], hex!("
            b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        ")[..]);

        // same for Sha512
        let mut hasher = Sha512::new();
        hasher.update(b"hello world");
        let result = hasher.finalize();

        assert_eq!(result[..], hex!("
            309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f
            989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f
        ")[..]);
    }
}