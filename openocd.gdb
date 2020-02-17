target extended-remote :3333

set print asm-demangle on

monitor arm semihosting enable

# ITM tracing
# send captured ITM to the file (or fifo) /tmp/itm.fifo
# (the microcontroller SWO pin must be connected to the programmer SWO pin)
# 16000000 must match the core clock frequency
# in this case the speed of SWO will be negotiated 
monitor tpiu config internal /tmp/itm.fifo uart off 16000000

# OR: make the microcontroller SWO pin output compatible with UART (8N1)
# 16000000 must match the core clock frequency
# 2000000 is the frequency of the SWO pin
# monitor tpiu config external uart off 16000000 2000000

# enable ITM ports
monitor itm port 0 on 
monitor itm port 1 on
monitor itm port 2 on

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
stepi

# un-comment to start and continue
# continue
