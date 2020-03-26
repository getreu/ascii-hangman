---
title: ASCII-Hangman - hangman game for children with ASCII-art rewarding
---

Hangman is a paper and pencil guessing game for two or more players.  One player
thinks of a word, phrase or sentence and the other tries to guess it by
suggesting letters or numbers, within a certain number of guesses. In this
version for children the computer selects a word, phrase or sentence randomly
out of a word-list defined in a configuration file. 

```
ASCII-ART HANGMAN FOR KIDS

          ,.
         (_|,.
        ,' /, )_____
     __j o``-'
    (")
     `-j
       `-._(
          |_\  |--^.
         /_]'|_| /_
            /_]'  /

Lives:	7	Last guess: k

 g o o d   l u _ k

Type a letter, then press [Enter]: 
```

Read more in [ASCII-Hangman's user manual](https://blog.getreu.net/projects/ascii-hangman/ascii-hangman--manual.html).

## Documentation

User documentation:

* User manual:

  [ASCII-Hangman user manual - html](https://blog.getreu.net/projects/ascii-hangman/ascii-hangman--manual.html)

  [ASCII-Hangman user manual - pdf](https://blog.getreu.net/_downloads/ascii-hangman--manual.pdf)


Developer documentation:

* [API documentation](https://blog.getreu.net/projects/ascii-hangman/_downloads/doc/ascii_hangman/)


## Source code

Repository:

* [ASCII-Hangman on Github](https://github.com/getreu/ascii-hangman)


## Distribution

* Binaries for latest release (Linux, Windows, iOS)

    1. Open: [Releases · getreu/ascii-hangman](https://github.com/getreu/ascii-hangman/releases)

    2. Open the latest release.

    3. Open *assets*.

    4. Download the packed executable for your operating system.

    5. Installation: see below.

* Binaries and packages (usually built from latest commit):

  - Executable for Windows:

    [x86_64-pc-windows-gnu/release/ascii-hangman.exe](https://blog.getreu.net/projects/ascii-hangman/_downloads/x86_64-pc-windows-gnu/release/ascii-hangman.exe)

  - Binary for Linux:

    [x86_64-unknown-linux-gnu/release/ascii-hangman](https://blog.getreu.net/projects/ascii-hangman/_downloads/x86_64-unknown-linux-gnu/release/ascii-hangman)

    [x86_64-unknown-linux-musl/release/ascii-hangman](https://blog.getreu.net/projects/ascii-hangman/_downloads/x86_64-unknown-linux-musl/release/ascii-hangman)

  - Package for Debian and Ubuntu:

    [x86_64-unknown-linux-gnu/debian/ascii-hangman_4.10.0_amd64.deb](https://blog.getreu.net/projects/ascii-hangman/_downloads/x86_64-unknown-linux-gnu/debian/ascii-hangman_4.10.0_amd64.deb)


* Zipfile with all binaries and documentation:

  - [ascii-hangman all](https://blog.getreu.net/_downloads/ascii-hangman.zip)



## Building and installing

1. Install *Rust* with [rustup](https://www.rustup.rs/):

       curl https://sh.rustup.rs -sSf | sh

   The fast-track procedure:

       cargo install ascii-hangman
       sudo cp ~/.cargo/bin/ascii-hangman /usr/local/bin

   If it works for you, you are done. Otherwise continue the next step.

2. Download [ascii-hangman](#ascii-hangman):

       git clone git@github.com:getreu/ascii-hangman.git

3. Build:

   Enter the *ascii-hangman* directory where the file `Cargo.toml`
   resides:

       cd ascii-hangman


   Then execute:

       cargo build --release
       ./doc/make--all

4. Install:

   a.  Linux:

           # install binary
           sudo cp target/release/ascii-hangman /usr/local/bin/

   b.  Windows:

       Copy the binary `target/release/ascii-hangman.exe` on your desktop.

   See the user manual for a detailed installation description.

This project follows [Semantic Versioning](https://semver.org/).



## About

Author:

* Jens Getreu

Copyright:

* Apache 2 licence or MIT licence

<!--
Build status:

* ![status](https://travis-ci.org/getreu/ascii-hangman.svg?branch=master)  
-->
