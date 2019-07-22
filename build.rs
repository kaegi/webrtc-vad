// build.rs

use std::path::Path;
use std::process::Command;

extern crate cc;

fn main() {
    if !Path::new("resources/libfvad/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }

    cc::Build::new()
        .include("resources/libfvad/src")
        .include("resources/libfvad/src/vad")
        .include("resources/libfvad/src/signal_processing")
        .file("resources/libfvad/src/signal_processing/division_operations.c")
        .file("resources/libfvad/src/signal_processing/get_scaling_square.c")
        .file("resources/libfvad/src/signal_processing/resample_48khz.c")
        .file("resources/libfvad/src/signal_processing/resample_by_2_internal.c")
        .file("resources/libfvad/src/signal_processing/resample_fractional.c")
        .file("resources/libfvad/src/signal_processing/spl_inl.c")
        .file("resources/libfvad/src/signal_processing/energy.c")
        .file("resources/libfvad/src/vad/vad_core.c")
        .file("resources/libfvad/src/vad/vad_filterbank.c")
        .file("resources/libfvad/src/vad/vad_gmm.c")
        .file("resources/libfvad/src/vad/vad_sp.c")
        .file("resources/libfvad/src/fvad.c")
        .compile("libfvad");
}
