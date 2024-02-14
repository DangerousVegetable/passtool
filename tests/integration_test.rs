use passtool::*;

#[test]
fn passtable_test() -> Result<(), Error>{
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
fn passtable_test2() -> Result<(), Error>{
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
fn incorrect_password_passtable_test() -> Result<(), Error>{
    let message = "super secret message";
    let password = "super secret password";
    let mut pt = PassTable::new();
    let name = String::from("test");
    pt.add_password(&name, message, password)?;
    let pass = pt.get_password(&name, "bebra");
    assert!(pass.is_err_and(|x| x == IncorrectPass));
    Ok(())
}
#[test]
fn not_found_passtable_test() -> Result<(), Error>{
    let message = "super secret message";
    let password = "super secret password";
    let mut pt = PassTable::new();
    let name = String::from("test");
    pt.add_password(&name, message, password)?;
    let pass = pt.get_password(&"test2".to_string(), "bebra");
    assert!(pass.is_err_and(|x| if let PassNotFound = x {true} else {false}));
    Ok(())
}

#[test]
fn alredy_exists_passtable_test() -> Result<(), Error>{
    let message = "super secret message";
    let password = "super secret password";
    let mut pt = PassTable::new();
    let name = String::from("test");
    pt.add_password(&name, message, password)?;
    let res = pt.add_password(&name, message, password);
    assert!(res.is_err_and(|x| if let PassExists = x {true} else {false}));
    Ok(())
}

#[test]
fn password_encrypt_test() -> Result<(), aes_gcm_siv::Error>{
    let password = "super secret password";
    let message = Vec::from(b"Hello world!");
    let cypher = encrypt(&message, password)?;
    let message2 = decrypt(&cypher, password)?;
    assert_eq!(&message, &message2);
    Ok(())
}

#[test]
fn incorrect_password_encrypt_test2() -> Result<(), aes_gcm_siv::Error>{
    let password = "super secret password";
    let password2 = "super not secret password";
    let message = Vec::from(b"Hello world!");
    let cypher = encrypt(&message, password)?;
    let message2 = decrypt(&cypher, password2);
    assert!(message2.is_err());
    Ok(())
}