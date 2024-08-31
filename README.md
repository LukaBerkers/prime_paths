# Prime Paths

This program generates all prime paths[^1] from a given control flow graph.
It takes a Graphviz dot file containing the control flow graph as input, and for every prime path of the graph,
outputs a new dot file of that path using the node date from the input file.

The program is currently only tailored to my own specific use case,
does not support the entire dot language, and the expected input format is not documented.
I might work on this in the future.

Let me know if you want me to expand this program, or you have any questions or suggestions!

[^1]: It is hard to find a good explanation online of prime path coverage.
    I recommend the book *Introduction to Software Testing* by Paul Ammann and Jeff Offutt.