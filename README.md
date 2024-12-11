# About
Clap command line application to simulate Barabási-Albert models and two variants, the following models can be run
- Vertex growth + Preferential attachement
- Vertex growth + Random attachement
- No growth + Preferential attachement

This simulation was made to support a lab work for the _Complex and Social Networks_ course of the Facultad de Informática de Barcelona

## Usage
```sh
Usage: barabasi_albert_simulation [OPTIONS] --n <N> --m <M> --model <MODEL>
    Options:
  -n, --n <N>

  -m, --m <M>

  -t, --t-max <T_MAX>
          [default: 100000]
  -i, --iterations <ITERATIONS>
          [default: 100]
  -s, --starting-graph <STARTING_GRAPH>
          [default: complete] [possible values: complete, star, disconnected]
      --model <MODEL>
          [possible values: growth_preferential, no_growth_preferential, growth_random]
  -h, --help
          Print help
  -V, --version
          Print version
```
