//! Tests for the registry

use std::sync::mpsc;
use std::thread;

use super::*;

use super::super::convert::{ToTable, FromTable, LuaDecoder};

use hlua;
use hlua::any::AnyLuaValue;

lua_convertible! {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Point {
        x: i32,
        y: i32
    }
}

#[test]
fn registry_tests() {
    let num = 1i32;
    let double = -392f64;
    let string = "Hello world".to_string();
    let numbers = vec![1, 2, 3, 4, 5];
    let point = Point { x: -11, y: 12 };

    set(String::from("test_num"), LUA_ACCESS, num);
    set(String::from("test_double"), LUA_ACCESS, double);
    set(String::from("test_string"), LUA_ACCESS, string.clone());
    set(String::from("test_numbers"), LUA_ACCESS, numbers.clone());
    set(String::from("test_point"), LUA_ACCESS, point.clone());

    assert!(contains_key(&String::from("test_num")));
    assert!(contains_key(&String::from("test_double")));
    assert!(contains_key(&String::from("test_string")));
    assert!(contains_key(&String::from("test_numbers")));
    assert!(contains_key(&String::from("test_point")));

    assert_eq!(get::<_, i32>(&String::from("test_num")).unwrap().1, num);
    assert_eq!(get::<_, f64>(&String::from("test_double")).unwrap().1, double);
    assert_eq!(get::<_,String>(&String::from("test_string")).unwrap().1, string);
    assert_eq!(get::<_, Vec<i32>>(&String::from("test_numbers")).unwrap().1,
               numbers);
    assert_eq!(get::<_, Point>(&String::from("test_point")).unwrap().1, point);

    let (tx, rx) = mpsc::channel();

    let tx1 = tx.clone();
    thread::spawn(move || {
        read_thread(String::from("test_num"), num, tx1);
    });
    let tx2 = tx.clone();
    thread::spawn(move || {
        read_thread(String::from("test_double"), double, tx2);
    });
    let tx3 = tx.clone();
    thread::spawn(move || {
        read_thread(String::from("test_string"), string, tx3);
    });
    let tx4 = tx.clone();
    thread::spawn(move || {
        read_thread(String::from("test_numbers"), numbers, tx4);
    });
    let tx5 = tx.clone();
    thread::spawn(move || {
        read_thread(String::from("test_point"), point, tx5);
    });

    let mut result = true;

    for i in 0..5 {
        result = result && rx.recv().unwrap();
    }
    assert!(result);
}

fn read_thread<T>(name: String, in_val: T, sender: mpsc::Sender<bool>)
where T: ::std::fmt::Debug + FromTable + PartialEq {
    for _ in 1 .. 50 {
        if let Ok(acc_val) = get::<_, T>(&name) {
            let (acc, val) = acc_val;
            assert!(acc.contains(LUA_ACCESS));
            assert_eq!(val, in_val);
        }
        else {
            sender.send(false).unwrap();
        }
    }
    sender.send(true).unwrap();
}
