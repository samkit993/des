#![feature(alloc)]
// use of Weak, downgrade, strong_count

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

extern crate rand;
use rand::distributions::IndependentSample;
use rand::distributions::exponential::Exp;

#[derive(Debug)]
struct Request {
    id: usize,
    arrival_time: usize,
    total_service: usize,
    remaining_service: usize,
}

#[derive(Debug)]
struct Core {
    status: CoreStatus,
    request: Option<Rc<RefCell<Request>>>,
    quantum_start: usize,
    total_busy_time: usize,
}

#[derive(Debug, Eq, PartialEq)]
enum CoreStatus {
    Idle, Busy
}

#[derive(Debug)]
enum EventType {
    Arrival(Rc<RefCell<Request>>),
    Departure(Rc<RefCell<Core>>),
    QuantumOver(Rc<RefCell<Core>>),
    Timeout(Weak<RefCell<Request>>)
}
// EventType impl {{{
impl PartialEq for EventType {
    fn eq(&self, other: &EventType) -> bool {
        use EventType::*;
        match (self, other) {
            (&Arrival(_), &Arrival(_)) | (&Departure(_), &Departure(_)) |
                (&QuantumOver(_), &QuantumOver(_)) | (&Timeout(_), &Timeout(_))
                => true,
            _ => false
        }
    }
}
impl Eq for EventType { }
impl PartialOrd for EventType {
    fn partial_cmp(&self, other: &EventType) -> Option<Ordering> {
        if self.eq(&other) {
             return Some(Ordering::Equal);
        }
        match (self, other) {
            (&EventType::Timeout(_), _) => Some(Ordering::Greater),
            (_, &EventType::Timeout(_)) => Some(Ordering::Less),
            _ => None,
        }
    }
}
// EventType }}}

#[derive(Debug)]
struct Event {
    _type: EventType,
    timestamp: usize,
}
// Event impl {{{
impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        // Consider incomparable 'EventType's as equals too
        self.timestamp == other.timestamp && !(self._type > other._type || self._type < other._type)
    }
}
impl Eq for Event { }
impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        if self.eq(&other) {
            return Ordering::Equal;
        }
        // Notice that the we flip the ordering here to make it a min-heap
        match other.timestamp.cmp(&self.timestamp) {
            Ordering::Equal => self._type.partial_cmp(&other._type).unwrap(),
            ord => ord,
        }
    }
}
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
// Event }}}

fn main() {
    use EventType::{Arrival, Timeout};

    let mut events = BinaryHeap::new();
    let mut rng = rand::thread_rng();
    let exp = Exp::new(1.0/8.0);

    let mut v = exp.ind_sample(&mut rng);
    println!("rand: {}", v);

    let req = Rc::new(RefCell::new(Request { id: 89, arrival_time: v as usize, total_service: 4, remaining_service: 4 }));
    let mut e = Event { _type: Arrival(req.clone()), timestamp: req.borrow().arrival_time };
    events.push(e);

    v += exp.ind_sample(&mut rng);
    e = Event { _type: Timeout(req.clone().downgrade()), timestamp: v as usize };
    events.push(e);

    let mut e = events.pop().unwrap();
    println!("{:?}", &e);
    e = events.pop().unwrap();
    println!("{:?}", &e);
    assert_eq!(events.pop(), None);
}
