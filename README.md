# fat32rs

Minimal FAT32 Client written in #[no_std] Rust

This is library was designed to operate within some specific constraints imposed by running in an embedded systems environment.

Based on this original Python prototype: [fat32py](https://github.com/careyi3/fat32py)

**The following instructions assumes a Linux or Mac environment*

![Test](https://github.com/careyi3/fat32rs/actions/workflows/test.yml/badge.svg)

## Features

The only features implemented are:

- List the files in the root directory of the drive
- Append to a file on the drive
- Create a new file in the root directory of the drive

## Setup

TODO

## Running

To run the examples simply run the below:

```bash
$ cargo run --example print_files
 
```

Note: If you want to read from your own physical drive, you can pass the `--path=<your_drives_path>` argument which will run against your physical drive.

```bash
$ cargo run --example print_files --path=/dev/disk4
 
```

I recommend using an SDCard. If you plug one into you system, you can see the name for it by listing all the files in your `/dev` directory:

```bash
$ cd /dev
$ ls
 
```

## Tests

You can run tests by running:

```bash
$ cargo test
 
```

This will use a 64MB `.img` drive preloaded with test files which you can find at `./tests/data/drive.img`.  

## Generating a disk image fixture

```bash
$ cd ./tests/data
 
```

This will create an empty `.dmg` image formatted for FAT32 with the volume name `DRIVE`.

```bash
$ sudo hdiutil create -size 64m -fs FAT32 -layout MBRSPUD -volname DRIVE ./drive.dmg
 
```

This will create an empty `.dmg` image formatted for FAT32 with the volume name `DRIVE`.

```bash
$ hdiutil convert drive.dmg -format UDRW -o drive.img
 
```

```bash
$ hdiutil attach drive.img
/dev/disk6              FDisk_partition_scheme         
/dev/disk6s1            DOS_FAT_32                      /Volumes/DRIVE
# this output you will need for the next step, will be different for your system
```

```bash
$ open /Volumes/DRIVE
 

```

You can now use the disk as normal. If you are using the drive committed in this repo, you will notice it has two files `LOG-1` and `LOG-2`. These are used by the tests. However, you can add or modify the drive as you like, just make sure to update the tests!
