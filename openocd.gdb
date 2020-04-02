target extended-remote :3333

# set print asm-demangle on

monitor arm semihosting enable

# *try* to stop at the user entry point (it might be gone due to inlining)
break main

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# un-comment to check that flashing was successful
# compare-sections

# make sure the processor is reset before loading (flashing)
monitor reset init

load

# un-comment to start and immediately halt the processor
# stepi

# un-comment to start and continue
continue
