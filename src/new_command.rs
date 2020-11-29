use crate::hardware::{DevBoard, Target};
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::Path;

pub fn run_new_command(
    project_name: String,
    target: Target,
    dev_board: Option<DevBoard>,
) -> Result<(), std::io::Error> {
    if Path::exists(project_name.as_ref()) {
        colour::red_ln!("dir named '{}' already exists", project_name);
        panic!("`new` command failed")
    } else {
        create_dir(&project_name).unwrap_or_else(|_| {
            colour::red_ln!(
                "insufficient permissions for creating dir '{}'",
                project_name.clone()
            );
            panic!("`new` command failed")
        })
    }

    create_top_file(project_name.clone())?;
    create_yosys_script(project_name.clone(), target)?;
    // LPF (logical preference files) describe available hardware for mapping abstract ports to
    // physical pins / hwrdware.  Search for "lpf" in this document:
    // https://www.latticesemi.com/-/media/LatticeSemi/Documents/UserManuals/1D/DiamondUserGuide33.ashx?document_id=50781
    create_lpf_file(project_name.clone(), dev_board)?;
    create_makefile(project_name, target, dev_board)
}

fn create_top_file(project_name: String) -> Result<(), std::io::Error> {
    let mut top_file = File::create(format!("{0}/{0}.v", project_name))?;
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
        FMT_TOP_FILE_NAME = project_name
        }
        .as_bytes(),
    )
}

fn create_yosys_script(project_name: String, target: Target) -> Result<(), std::io::Error> {
    let synth_command = match target {
        Target::ECP5_85k => "synth_ecp5",
    };
    let mut script_file = File::create(format!("{0}/{0}.ys", project_name))?;
    script_file.write_all(
        formatdoc! {r#"
read_verilog {FMT_TOP_FILE_NAME}.v
{FMT_SYNTH_COMMAND} -noccu2 -nomux -nodram -json {FMT_TOP_FILE_NAME}.json
    "#,
            FMT_SYNTH_COMMAND = synth_command,
            FMT_TOP_FILE_NAME = project_name
        }
        .as_bytes(),
    )
}

fn create_lpf_file(
    project_name: String,
    dev_board: Option<DevBoard>,
) -> Result<(), std::io::Error> {
    match dev_board {
        Some(DevBoard::ULX3S) => {
            let mut lpf_file = File::create(format!("{}/ulx3s_v20.lpf", project_name))?;
            lpf_file.write_all(include_bytes!("../resources/ulx3s_v20.lpf"))
        }
        None => Ok(()),
    }
}

fn create_makefile(
    project_name: String,
    target: Target,
    dev_board: Option<DevBoard>,
) -> Result<(), std::io::Error> {
    let mut makefile = File::create(format!("{0}/Makefile", project_name))?;
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
	rm -rf $(VDIRFB)/ $(SIMPROG) $(VCDFILE) $(TOPMOD)/ $(BINFILE) $(RPTFILE)
	rm -rf $(TOPMOD).json $(TOPMOD)_out.config out.bit

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


out.bit: $(TOPMOD)_out.config
	ecppack $(TOPMOD)_out.config out.bit

$(TOPMOD)_out.config: $(TOPMOD).json
		{FMT_NEXTPNR_CMD}
		{FMT_LPF_ARG}
		--textcfg $(TOPMOD)_out.config 

$(TOPMOD).json: $(TOPMOD).ys $(TOPMOD).v
	yosys $(TOPMOD).ys 

prog: out.bit
	fujprog out.bit
    "#,
    FMT_TOP_FILE_NAME = project_name,
    FMT_NEXTPNR_CMD = match target {
        Target::ECP5_85k => format!("nextpnr-ecp5 --85k --json {}.json \\", project_name)
    },
    FMT_LPF_ARG = match dev_board {
        Some(DevBoard::ULX3S) => "--lpf ulx3s_v20.lpf \\",
        None => ""
    },
    }.as_bytes())
}
// TODO: Add build dir
