/*
 * SAVE-LOAD pack to file.
 * used for mudule system
 * pack is INTERFACE to module
 * so we can save in header info about module
 * when module compiled to C file
 */

use pack::*;
use std::io::{Read, Write};
use std::io;

fn read_file(path : &str) -> Pack {
}

fn write_file(pack : &str, path : &str) -> io::Result<()> {
}
