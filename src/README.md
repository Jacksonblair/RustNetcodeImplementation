# Using protocol

- So we have an object we want to serialize, like a players position or something
- We create a packet that implements Packet, which contains various data we want to send over the network (like the above object)
    - Our packet has to implement getType, to tell the packet type
    - And our packet has to implmene the read and write methods from the Object trait, which the Packet trait enforces implementation of


- We can create a function 