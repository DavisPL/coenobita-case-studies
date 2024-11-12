# Coenobita
## Case Studies

What impact does Coenobita have on build and runtime? How many lines must be changed for a crate to be capability safe? Those are the questions we intend to answer with these case studies.

## Benchmarking

You can measure Coenobita's impact on build and runtime by running the `benchmark.py` script. For example, the command below will measure the build time of every crate in this directory using Coenobita.

```
python3 benchmark.py build --wrapper coenobita --iters 5
```

### Usage
```
python3 benchmark.py [build|test] [OPTION]

-w, --wrapper | Which `rustc` wrapper to use
-i, --iters   | How many iterations to perform (default is 10)
-r, --root    | The root directory (default is '.')
-l, --log     | Which log level to use (default is none)
```