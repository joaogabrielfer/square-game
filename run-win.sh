#!/usr/bin/env bash
cargo xwin build --target x86_64-pc-windows-msvc --release && ./target/x86_64-pc-windows-msvc/release/square-game.exe
