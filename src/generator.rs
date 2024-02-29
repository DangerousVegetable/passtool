use rand::Rng;

const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"1234567890";
const SPECIAL: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

pub fn generate_password(len: u16, letters: bool, digits: bool, special: bool) -> String {
    let mut a = Vec::default();
    if letters {a.append(&mut Vec::from(LETTERS));}
    if digits {a.append(&mut Vec::from(DIGITS));}
    if special {a.append(&mut Vec::from(SPECIAL));}
    
    let mut rng = rand::thread_rng();
    let mut res = String::new();
    for _ in 0..len {
        res.push(a[rng.gen_range(0..a.len())] as char);
    }
    res
}

#[test]
fn generate_letters_test() {
    let pass = generate_password(10, true, false, false);
    assert_eq!(pass.len(), 10);
    assert!(!pass.contains(|x : char| {DIGITS.contains(&(x as u8)) || SPECIAL.contains(&(x as u8))}));
}

#[test]
fn generate_digits_test() {
    let pass = generate_password(10, false, true, false);
    assert_eq!(pass.len(), 10);
    assert!(!pass.contains(|x : char| {LETTERS.contains(&(x as u8)) || SPECIAL.contains(&(x as u8))}));
}

#[test]
fn generate_special_test() {
    let pass = generate_password(10, false, false, true);
    assert_eq!(pass.len(), 10);
    assert!(!pass.contains(|x : char| {LETTERS.contains(&(x as u8)) || DIGITS.contains(&(x as u8))}));
}