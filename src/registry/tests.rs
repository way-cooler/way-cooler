//! Tests for the registry

use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::thread;

use rustc_serialize::Decodable;
use rustc_serialize::json::{Json, ToJson};

use registry;
use registry::{AccessFlags, get_struct};

json_convertible! {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Point {
        x: i32,
        y: i32
    }
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }
}

const ERR: &'static str = "Key which was added no longer exists!";

fn prop_get() -> Json {
    Point::new(0, 0).to_json()
}

#[test]
fn add_keys() {
    let num = 1i32;
    let double = -392f64;
    let string = "Hello world".to_string();
    let numbers = vec![1, 2, 3, 4, 5];
    let point = Point::new(-11, 12);

    registry::set_struct("test_num".to_string(), AccessFlags::READ(), num.to_json()).expect(ERR);
    registry::set_struct("test_double".to_string(), AccessFlags::READ(), double).expect(ERR);
    registry::set_struct("test_string".to_string(), AccessFlags::READ(), string.clone()).expect(ERR);
    registry::set_struct("test_numbers".to_string(), AccessFlags::READ(), numbers.clone()).expect(ERR);
    registry::set_struct("test_point".to_string(), AccessFlags::READ(), point.clone()).expect(ERR);
    registry::set_property_field("test_func".to_string(), Some(Arc::new(prop_get)), None).expect(ERR);
}

#[test]
fn contains_keys() {
    thread::sleep(Duration::from_millis(240));
    assert!(registry::contains_key(&"test_num".to_string()), "num");
    assert!(registry::contains_key(&"test_double".to_string()), "double");
    assert!(registry::contains_key(&"test_string".to_string()), "string");
    assert!(registry::contains_key(&"test_numbers".to_string()), "numbers");
    assert!(registry::contains_key(&"test_point".to_string()), "point");
    assert!(registry::contains_key(&"test_func".to_string()), "func");
}

#[test]
fn keys_equal() {
    let num = 1i32;
    let double = -392f64;
    let string = "Hello world".to_string();
    let numbers = vec![1, 2, 3, 4, 5];
    let point = Point::new(-11, 12);
    thread::sleep(Duration::from_millis(240));
    assert_eq!(get_struct::<_, i32>(&"test_num".to_string()).expect(ERR).1, num);
    assert_eq!(get_struct::<_, f64>(&"test_double".to_string()).expect(ERR).1, double);
    assert_eq!(get_struct::<_,String>(&"test_string".to_string()).expect(ERR).1, string);
    assert_eq!(get_struct::<_, Vec<i32>>(&"test_numbers".to_string()).expect(ERR).1,
               numbers);
    assert_eq!(get_struct::<_, Point>(&"test_point".to_string()).expect(ERR).1, point);
    assert_eq!(get_struct::<_, Point>(&"test_func".to_string()).expect(ERR).1,
               Point::new(0, 0));
}

#[test]
fn key_perms() {
    thread::sleep(Duration::from_millis(240));
    registry::set_struct("perm_none".to_string(), AccessFlags::empty(), 0).expect(ERR);
    registry::set_struct("perm_read".to_string(), AccessFlags::READ(), 1).expect(ERR);
    registry::set_struct("perm_write".to_string(), AccessFlags::WRITE(), 2).expect(ERR);

    assert_eq!(get_struct::<_, i32>(&"perm_none".to_string()).expect(ERR).0, AccessFlags::empty());
    assert_eq!(get_struct::<_, i32>(&"perm_read".to_string()).expect(ERR).0, AccessFlags::READ());
    assert_eq!(get_struct::<_, i32>(&"perm_write".to_string()).expect(ERR).0, AccessFlags::WRITE());
    assert_eq!(registry::get_json(&"test_func".to_string()).expect(ERR).0, AccessFlags::all());
}

#[test]
fn multithreaded() {
    let (tx, rx) = mpsc::channel();
    thread::sleep(Duration::from_millis(240));
    let num = 1i32;
    let double = -392f64;
    let string = "Hello world".to_string();
    let numbers = vec![1, 2, 3, 4, 5];
    let point = Point { x: -11, y: 12 };

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

    for _ in 0..5 {
        result = result && rx.recv().expect("Unable to connect to read thread");
    }
    assert!(result);
}

fn read_thread<T>(name: String, in_val: T, sender: mpsc::Sender<bool>)
where T: ::std::fmt::Debug + Decodable + PartialEq {
    for _ in 1 .. 50 {
        if let Ok(acc_val) = get_struct::<_, T>(&name) {
            let (acc, val) = acc_val;
            assert!(acc.contains(AccessFlags::READ()));
            assert_eq!(val, in_val);
        }
        else {
            sender.send(false).expect("Unable to reply to test thread");
        }
    }
    sender.send(true).expect("Unable to reply to test thread");
}
