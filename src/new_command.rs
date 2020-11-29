use std::fs::{create_dir, File};
use std::io::Write;
use std::path::Path;
use crate::hardware::{Target, DevBoard};

pub fn run_new_command(name: String, target: Target, dev_board: Option<DevBoard>) -> Result<(), std::io::Error> {
    if Path::exists(name.as_ref()) {
        colour::red_ln!("dir named '{}' already exists", name);
        panic!("`new` command failed")
    } else {
        create_dir(&name).unwrap_or_else(|_| {
            colour::red_ln!(
                "insufficient permissions for creating dir '{}'",
                name.clone()
            );
            panic!("`new` command failed")
        })
    }

    create_top_file(name.clone())?;
    create_yosys_script(name.clone(), target)?;
    // LPF (logical preference files) describe available hardware for mapping abstract ports to
    // physical pins / hwrdware.  Search for "lpf" in this document:
    // https://www.latticesemi.com/-/media/LatticeSemi/Documents/UserManuals/1D/DiamondUserGuide33.ashx?document_id=50781
    // TODO: Make `name` variable better
    create_lpf_file(name.clone(), dev_board)?;
    create_makefile(name, target, dev_board)
}

fn create_top_file(name: String) -> Result<(), std::io::Error> {
    let mut top_file = File::create(format!("{0}/{0}.v", name))?;
    // This currently simply hardcodes the ULX3s blinky example code from:
    // https://github.com/ulx3s/blink/blob/4f25f454300b54e797416a7fd7b5c88d252d8d82/blinky.v
    // TODO: Make this a lighter and non-target specific top-file
    top_file.write_all(
        formatdoc! {r#"
`ifdef VERILATOR
/* verilator lint_off UNUSED */
module {FMT_TOP_FILE_NAME}(input i_clk, input [6:0] btn, output [7:0] o_led);
/* verilator lint_on UNUSED */
    wire i_clk;
    wire [6:0] btn;
    wire [7:0] o_led;
`else
module top(input clk_25mhz,
           input [6:0] btn,
           output [7:0] led,
           output wifi_gpio0);

    wire i_clk;

    // Tie GPIO0, keep board from rebooting
    assign wifi_gpio0 = 1'b1;
    assign i_clk= clk_25mhz;
    reg [7:0] o_led;
    assign led= o_led;
`endif

    localparam ctr_width = 32;
    reg [ctr_width-1:0] ctr = 0;

    always @(posedge i_clk) begin
               ctr <= ctr + 1;
          o_led[7] <= 1;
          o_led[6] <= btn[1];
        o_led[5:0] <= ctr[23:18];
    end
endmodule
    "#,
        FMT_TOP_FILE_NAME = name
        }
            .as_bytes(),
    )
}

fn create_yosys_script(name: String, target: Target) -> Result<(), std::io::Error> {
    let synth_command = match target {
        Target::ECP5_85k => "synth_ecp5"
    };
    let mut script_file = File::create(format!("{0}/{0}.ys", name))?;
    script_file.write_all(
        formatdoc! {r#"
read_verilog {FMT_TOP_FILE_NAME}.v
{FMT_SYNTH_COMMAND} -noccu2 -nomux -nodram -json {FMT_TOP_FILE_NAME}.json
    "#,
            FMT_SYNTH_COMMAND = synth_command,
            FMT_TOP_FILE_NAME = name
        }
            .as_bytes(),
    )
}

fn create_lpf_file(name: String, dev_board: Option<DevBoard>) -> Result<(), std::io::Error> {
    match dev_board {
        Some(DevBoard::ULX3S) => {
            let mut lpf_file = File::create(format!("{}/ulx3s_v20.lpf", name))?;
            lpf_file.write_all(include_bytes!("../resources/ulx3s_v20.lpf"))
        },
        None => Ok(())
    }
}

fn create_makefile(name: String, target: Target, dev_board: Option<DevBoard>) -> Result<(), std::io::Error> {
    let mut makefile = File::create(format!("{0}/Makefile", name))?;
    makefile.write_all(formatdoc! {r#"
.PHONY: all
.DELETE_ON_ERROR:
TOPMOD  := {FMT_TOP_FILE_NAME}
VLOGFIL := $(TOPMOD).v
BINFILE := $(TOPMOD).bin
VDIRFB  := ./obj_dir
all: $(VCDFILE)

GCC := g++
CFLAGS = -g -Wall -I$(VINC) -I $(VDIRFB)
#
# Modern versions of Verilator and C++ may require an -faligned-new flag
# CFLAGS = -g -Wall -faligned-new -I$(VINC) -I $(VDIRFB)

VERILATOR=verilator
VFLAGS := -O3 -MMD --trace -Wall

## Find the directory containing the Verilog sources.  This is given from
## calling: "verilator -V" and finding the VERILATOR_ROOT output line from
## within it.  From this VERILATOR_ROOT value, we can find all the components
## we need here--in particular, the verilator include directory
VERILATOR_ROOT ?= $(shell bash -c '$(VERILATOR) -V|grep VERILATOR_ROOT | head -1 | sed -e "s/^.*=\s*//"')
##
## The directory containing the verilator includes
VINC := $(VERILATOR_ROOT)/include

$(VDIRFB)/V$(TOPMOD).cpp: $(TOPMOD).v
	$(VERILATOR) $(VFLAGS) -cc $(VLOGFIL)

$(VDIRFB)/V$(TOPMOD)__ALL.a: $(VDIRFB)/V$(TOPMOD).cpp
	make --no-print-directory -C $(VDIRFB) -f V$(TOPMOD).mk

$(SIMPROG): $(SIMFILE) $(VDIRFB)/V$(TOPMOD)__ALL.a $(COSIMS)
	$(GCC) $(CFLAGS) $(VINC)/verilated.cpp				\
		$(VINC)/verilated_vcd_c.cpp $(SIMFILE) $(COSIMS)	\
		$(VDIRFB)/V$(TOPMOD)__ALL.a -o $(SIMPROG)

test: $(VCDFILE)

$(VCDFILE): $(SIMPROG)
	./$(SIMPROG)

## 
.PHONY: clean
clean:
	rm -rf $(VDIRFB)/ $(SIMPROG) $(VCDFILE) {FMT_TOP_FILE_NAME}/ $(BINFILE) $(RPTFILE)
	rm -rf {FMT_TOP_FILE_NAME}.json {FMT_TOP_FILE_NAME}_out.config out.bit

##
## Find all of the Verilog dependencies and submodules
##
DEPS := $(wildcard $(VDIRFB)/*.d)

## Include any of these submodules in the Makefile
## ... but only if we are not building the "clean" target
## which would (oops) try to build those dependencies again
##
ifneq ($(MAKECMDGOALS),clean)
ifneq ($(DEPS),)
include $(DEPS)
endif
endif


out.bit: {FMT_TOP_FILE_NAME}_out.config
	ecppack {FMT_TOP_FILE_NAME}_out.config out.bit

{FMT_TOP_FILE_NAME}_out.config: {FMT_TOP_FILE_NAME}.json
		{FMT_NEXTPNR_CMD}
		{FMT_LPF_ARG}
		--textcfg {FMT_TOP_FILE_NAME}_out.config 

{FMT_TOP_FILE_NAME}.json: {FMT_TOP_FILE_NAME}.ys {FMT_TOP_FILE_NAME}.v
	yosys {FMT_TOP_FILE_NAME}.ys 

prog: out.bit
	fujprog out.bit
    "#,
    FMT_TOP_FILE_NAME = name,
    FMT_NEXTPNR_CMD = match target {
        Target::ECP5_85k => format!("nextpnr-ecp5 --85k --json {}.json \\", name)
    },
    FMT_LPF_ARG = match dev_board {
        Some(DevBoard::ULX3S) => "--lpf ulx3s_v20.lpf \\",
        None => ""
    },
    }.as_bytes())
}
