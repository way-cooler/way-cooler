//! Tests for the registry

use std::sync::mpsc;
use std::thread;

use rustc_serialize::Decodable;
use rustc_serialize::json::ToJson;

use registry;
use registry::{LUA_READ, LUA_WRITE, LUA_PRIVATE, get_data};

json_convertible! {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Point {
        x: i32,
        y: i32
    }
}

const ERR: &'static str = "Key which was added no longer exists!";

#[test]
fn add_keys() {
    let num = 1i32;
    let double = -392f64;
    let string = "Hello world".to_string();
    let numbers = vec![1, 2, 3, 4, 5];
    let point = Point { x: -11, y: 12 };

    registry::set_data("test_num".to_string(), LUA_READ, num);
    registry::set_data("test_double".to_string(), LUA_READ, double);
    registry::set_data("test_string".to_string(), LUA_READ, string.clone());
    registry::set_data("test_numbers".to_string(), LUA_READ, numbers.clone());
    registry::set_data("test_point".to_string(), LUA_READ, point.clone());

    assert!(registry::contains_key(&"test_num".to_string()));
    assert!(registry::contains_key(&"test_double".to_string()));
    assert!(registry::contains_key(&"test_string".to_string()));
    assert!(registry::contains_key(&"test_numbers".to_string()));
    assert!(registry::contains_key(&"test_point".to_string()));

    assert_eq!(get_data::<_, i32>(&"test_num".to_string()).expect(ERR).1, num);
    assert_eq!(get_data::<_, f64>(&"test_double".to_string()).expect(ERR).1, double);
    assert_eq!(get_data::<_,String>(&"test_string".to_string()).expect(ERR).1, string);
    assert_eq!(get_data::<_, Vec<i32>>(&"test_numbers".to_string()).expect(ERR).1,
               numbers);
    assert_eq!(get_data::<_, Point>(&"test_point".to_string()).expect(ERR).1, point);

}

#[test]
fn lua_perms() {
    registry::set_data("perm_none".to_string(), LUA_PRIVATE, 0);
    registry::set_data("perm_read".to_string(), LUA_READ, 1);
    registry::set_data("perm_write".to_string(), LUA_WRITE, 2);

    assert_eq!(get_data::<_, i32>(&"perm_none".to_string()).expect(ERR).0, LUA_PRIVATE);
    assert_eq!(get_data::<_, i32>(&"perm_read".to_string()).expect(ERR).0, LUA_READ);
    assert_eq!(get_data::<_, i32>(&"perm_write".to_string()).expect(ERR).0, LUA_WRITE);
}

#[test]
fn multithreaded() {
    use std::time::Duration;
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
        if let Ok(acc_val) = get_data::<_, T>(&name) {
            let (acc, val) = acc_val;
            assert!(acc.contains(LUA_READ));
            assert_eq!(val, in_val);
        }
        else {
            sender.send(false).expect("Unable to reply to test thread");
        }
    }
    sender.send(true).expect("Unable to reply to test thread");
}
