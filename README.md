## hdlman

A CLI-based HDL project management tool.

### Supported commands

`hdlman new`

Creates boilerplate top-file, Makefile, and Yosys script.  Also includes any LPF files for your given target / dev-board.

Example:

```
$ hdlman new --project-name blinky_project --target ecp5-85k --dev-board ulx3s
    Created new HDL project 'blinky_project' with target 'ecp5-85k' and dev-board 'ulx3s'
$ cd blinky_project
$ make prog
Snip output. This synthesizes your source files, place-and-routes the result,
and uploads the bitstream to your target (i.e programs, or "prog"s, the FPGA).
$ tree
.
├── blinky_project.v
├── blinky_project.ys
├── build
│   ├── blinky_project.json
│   ├── blinky_project_out.config
│   └── out.bit
├── Makefile
└── resources
    └── ulx3s_v20.lpf
```

### Config

`hdlman` can be configured by creating a file named `.hdlman.toml` and placing it in the following directory: 

|Platform | Value                | Example        |
| ------- | -------------------- | -------------- |
| Linux   | `$HOME`              | /home/alice    |
| macOS   | `$HOME`              | /Users/Alice   |
| Windows | `{FOLDERID_Profile}` | C:\Users\Alice |

Here is a comprehensive list of respected configurations:
* `default-target = "<TARGET>"`
    * If this is specified, the `new` command will use this target if one is not passed via `--target`
* `default-dev-board = "<DEV_BOARD>"`
    * If this is specified, the `new` command will use this dev-board if one is not passed via `--dev-board`

### Targets vs. dev-boards

`hdlman` has the notion of "targets" and "dev-boards", where the actual FPGA is the target (e.g. ECP5-85k LUTs) and the
dev-board is a host to an FPGA plus other goodies.  The [ULX3S](https://radiona.org/ulx3s/) is an example of a dev-board.
This distinction is important for some commands such as `hdlman new`.


### Supported targets

* `ecp-85k`

### Supported dev-boards

* `ulx3s`

### Build and install

There are no official builds of `hdlman`, so you'll have to build it yourself to use it.

1. Install `git`: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
1. Clone this repository: `git clone git@github.com:twilco/hdlman.git && cd hdlman`
1. Download Rust: https://www.rust-lang.org/tools/install
1. Build `hdlman`: `cargo build --release`
1. This creates an `hdlman` executable in `target/release/hdlman`.  Place it somewhere globally accessible in your environment.
    * Linux: `sudo cp target/release/hdlman /usr/local/bin`

### Project goals

I created `hdlman` for my personal use as a way to automate some of the boilerplate involved in HDL
development.  [Vivado](https://www.xilinx.com/products/design-tools/vivado.html),
[ISE](https://www.xilinx.com/products/design-tools/ise-design-suite.html), etc. would normally be the tools doing this work, but I'm
running a [ULX3S](https://radiona.org/ulx3s/) with the [fully open-source toolchain](https://github.com/ulx3s/ulx3s-toolchain),
so I wanted a lightweight project management tool to make my life easier.

Right now, this project is heavily opinionated towards my workflow, tools, and hardware (Verilog, Verilator, Yosys, the ULX3s, ...).
However, I'm not opposed to making `hdlman` more generic, and PRs are welcome!
    
### License

GPLv3.0