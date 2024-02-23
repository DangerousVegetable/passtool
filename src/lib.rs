use core::fmt;
use std::fs;
use std::collections::HashMap;

use hex_literal::hex;

use sha2::{Sha256, Sha512, Digest}; 
use sha2::digest::typenum::Unsigned;

use aes_gcm_siv::{
    aead::{Aead, KeyInit, Key},
    Aes256GcmSiv, Nonce
};

use serde::{Serialize, Deserialize};

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
            Self::PassExists => f.write_str("password already exists"),
            Self::PassNotFound => f.write_str("password not found"),
            Self::IncorrectPass => f.write_str("incorrect password"),
            Self::AES => f.write_str("aes error")
        }
    }
}

impl std::error::Error for Error {}

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

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct PasswordMeta {
    pub description: String,
    pub apps: Vec<String>
}

impl PasswordMeta {
    pub fn new(description: String, apps: Vec<String>) -> Self {
        Self{description, apps}
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Password {
    cypher: Vec<u8>,
    meta: PasswordMeta
}

impl Password {
    pub fn from_cypher(cypher: Vec<u8>) -> Self{
        Password{cypher, meta: Default::default()}
    }

    pub fn update_meta(&mut self, meta: PasswordMeta) {
        self.meta = meta;
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PassTable {
    passwords: HashMap<String, Password>
}

impl PassTable {
    pub fn new() -> Self {
        PassTable { passwords: HashMap::new() }
    }

    fn encoded(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_binary(encoded: &[u8]) -> Result<Self, Box<dyn std::error::Error>>  {
        let table: Self = bincode::deserialize(encoded)?;
        Ok(table)
    }

    pub fn from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>>  {
        let encoded = fs::read(filename)?;
        PassTable::from_binary(&encoded)
    }

    pub fn to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>>{
        let encoded = self.encoded();
        fs::write(filename, encoded)?;
        Ok(())
    }

    fn get_cypher(&self, name: &str) -> Option<&Password> {
        self.passwords.get(name)
    }

    fn remove_cypher(&mut self, name: &str) -> Result<(), Error> {
        self.passwords.remove(name).ok_or(Error::PassNotFound)?;
        Ok(())
    }

    fn add_cypher(&mut self, name: String, cypher: Vec<u8>, meta: PasswordMeta) {
        self.passwords.insert(name, Password{cypher, meta});
    }

    pub fn get_password(&self, name: &str, key: &str) -> Result<String, Error> {
        let cypher = self.get_cypher(name).ok_or(PassNotFound)?;
        let password = decrypt(&cypher.cypher, key).or(Err(IncorrectPass))?;
        String::from_utf8(password).or(Err(AES))
    }

    pub fn add_password(&mut self, name: &str, password: &str, meta: PasswordMeta, key: &str) -> Result<(), Error>{
        if self.passwords.contains_key(name) {return Err(PassExists)}
        let cypher = encrypt(password.as_bytes(), key).or(Err(AES))?;
        self.add_cypher(String::from(name), cypher, meta);
        Ok(())
    }

    pub fn get_metadata(&self, name: &str) -> Result<&PasswordMeta, Error> {
        let p = self.get_cypher(name).ok_or(Error::PassNotFound)?;
        Ok(&p.meta)
    }

    pub fn update_metadata(&mut self, name: &str, meta: PasswordMeta) -> Result<(), Error> {
        let p = self.passwords.get_mut(name).ok_or(Error::PassNotFound)?;
        p.update_meta(meta);
        Ok(())
    }

    pub fn remove_password(&mut self, name: &str) -> Result<(), Error> {
        self.remove_cypher(name)
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    #[ignore]
    fn serialize_test() -> Result<(), Error>{
        let mut pt = PassTable::new();
        pt.add_password("pass1", "test1", PasswordMeta::default(), "password1")?;
        pt.add_password("pass2", "test2", PasswordMeta::default(), "password2")?;
        pt.add_password("pass3", "test3", PasswordMeta::default(), "password3")?;

        let encoded = pt.encoded();
        println!("{:?}", encoded);
        let pt2 = PassTable::from_binary(&encoded).unwrap();
        assert_eq!(pt, pt2);
        let pass = pt2.get_password("pass2", "password2")?;
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
        let nonce = &nonce_from_password::<Sha256>("xd");
        
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
        let key = key_from_password::<Sha256, Aes256GcmSiv>(password);
        //println!("{:?}", key);
        let cipher = Aes256GcmSiv::new(&key);
        let nonce = &nonce_from_password::<Sha256>(password);
        
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