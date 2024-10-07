# Improvements TODO

## Immutable Queue Optimization

The Immutable follower reads from disk, inline.
Disk IO is relatively expensive.
Decoding blocks is also expensive, it's better to do that in parallel with an application processing a previous block.

What we should do is have a read ahead queue, where a second task is reading ahead of the application following,
reading the next blocks from disk, and decoding them.

The main follower used by the application then reads from this red ahead queue.
This would help us better utilize disk and CPU resources, which would result in improved sync times.
