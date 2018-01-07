#![no_std]
#![feature(lang_items)]
#![cfg_attr(feature = "strict", deny(warnings))]

#![feature(alloc)]

extern crate alloc;
use alloc::vec::Vec;

extern crate pwasm_libc;
extern crate pwasm_alloc;

#[lang = "panic_fmt"]
#[no_mangle]
pub fn panic_fmt() -> ! {
	  loop {}
}

extern "C" {
	fn ext_print(utf8_data: *const u8, utf8_len: i32);
	fn ext_print_num(value: u64);
	fn set_storage(key_data: *const u8, key_len: i32, value_data: *const u8, value_len: i32);
	fn get_allocated_storage(key_data: *const u8, key_len: i32, written_out: *mut i32) -> *mut u8;
}

pub mod state {
	use alloc::vec::Vec;
	use super::{get_allocated_storage, set_storage as super_set_storage};

	pub fn storage(key: &[u8]) -> Vec<u8> {
		let mut length: i32 = 0;
		unsafe {
			let ptr = get_allocated_storage(&key[0], key.len() as i32, &mut length);
			Vec::from_raw_parts(ptr, length as usize, length as usize)
		}
	}

	pub fn set_storage(key: &[u8], value: &[u8]) {
		unsafe {
			super_set_storage(
				&key[0] as *const u8, key.len() as i32,
				&value[0] as *const u8, value.len() as i32
			);
		}
	}

	pub fn code() -> Vec<u8> {
		storage(b"\0code")
	}

	pub fn set_code(new: &[u8]) {
		set_storage(b"\0code", new)
	}

	fn value_vec(mut value: usize, initial: Vec<u8>) -> Vec<u8> {
		let mut acc = initial;
		while value > 0 {
			acc.push(value as u8);
			value /= 256;
		}
		acc
	}

	pub fn set_validator(index: usize, validator: &[u8]) {
		set_storage(&value_vec(index, b"\0validator".to_vec()), validator);
	}

	pub fn validator(index: usize) -> Vec<u8> {
		storage(&value_vec(index, b"\0validator".to_vec()))
	}

	pub fn set_validator_count(count: usize) {
		(count..validator_count()).for_each(|i| set_validator(i, &[]));
		set_storage(b"\0validator_count", &value_vec(count, Vec::new()));
	}

	pub fn validator_count() -> usize {
		storage(b"\0validator_count").into_iter().rev().fold(0, |acc, i| (acc << 8) + (i as usize))
	}

	pub fn validators() -> Vec<Vec<u8>> {
		(0..validator_count()).into_iter().map(validator).collect()
	}

	pub fn set_validators(validators: &[&[u8]]) {
		set_validator_count(validators.len());
		validators.iter().enumerate().for_each(|(v, i)| set_validator(v, i));
	}
}

pub fn print(utf8: &[u8]) {
	unsafe {
		ext_print(&utf8[0] as *const u8, utf8.len() as i32);
	}
}

/// Test passing of data.
#[no_mangle]
pub fn exec_test_data_in(input_data: *mut u8, input_len: usize) -> u64 {
	let input = unsafe {
		Vec::from_raw_parts(input_data, input_len, input_len)
	};

	let output = test_data_in(input);
	unsafe {
		ext_print_num(&output[0] as *const u8 as u64);
		ext_print_num(output.len() as u64);
	}
	&output[0] as *const u8 as u64 + ((output.len() as u64) << 32)
}

fn test_data_in(input: Vec<u8>) -> Vec<u8> {
	print(b"set_storage");
	state::set_storage(b"input", &input);

	print(b"code");
	state::set_storage(b"code", &state::code());

	print(b"set_code");
	state::set_code(&input);

	print(b"storage");
	let copy = state::storage(b"input");

	print(b"validators");
	let mut v = state::validators();
	v.push(copy);

	print(b"set_validators");
	state::set_validators(&v.iter().map(Vec::as_slice).collect::<Vec<_>>());

	print(b"finished!");
	b"all ok!".to_vec()
}
