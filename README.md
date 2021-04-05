# emu6502
This is a "code along" project that im creating as I follow [Dave Poo's 6502 Emulator](https://www.youtube.com/watch?v=qJgsuQoy9bc) series. With the only difference being that this implementation is in Rust instead of C++. 

# Progress 
- Episode 1 Done
  - Basic CPU structure
  - Basic Memory
  - Execute OpCodes from Memory
  - Implement LDA_IM (Load Acc - Immediate Address mode)
  - Implement LDA_ZP (Load Acc - Zero Page Address mode)
  - Implement LDA_ZP_X (Load Acc - Zero Page w/ X offset Address mode)
  - Implement JSR (Jump to Subroutine)
  - Unit Test the above
  
# Reference Docs 
[6502 Architecture Reference Docs](http://www.obelisk.me.uk/6502/index.html)
