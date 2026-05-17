#![no_std]

#[cfg(not(any(unix, windows)))]
mod rt;

mod sys;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_add(left: u64, right: u64) -> u64 {
    add(left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
