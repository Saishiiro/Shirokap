@echo off
cargo build --release && copy /Y target\release\shirokap.exe shirokap.exe && echo OK : shirokap.exe est pret
pause
