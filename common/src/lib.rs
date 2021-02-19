pub mod recurrence;
pub mod event;
pub mod calendar;
pub mod span;

#[macro_use] extern crate schemars;
#[macro_use] extern crate serde;
#[macro_use] extern crate thiserror;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
