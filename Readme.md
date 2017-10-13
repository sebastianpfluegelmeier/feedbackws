# FeedbackWS
A waveshaper for people who like weird math functions and hate their ears.

## What you need to install FeedbackWS
- git
- cargo
- the [osx_vst_bundler.sh] https://github.com/overdrivenpotato/rust-vst2/blob/master/osx_vst_bundler.sh script if you are on a mac

## How to install FeedbackWS
- open a terminal
- type `git clone https://github.com/sebastianpfluegelmeier/feedbackws'
- enter the directory with `cd feedbackws`
- type `cargo build`
- on linux copy the artifact into the vst folder `sudo cp target/debug/libfeedbackws.so /usr/lib/lxvst`
- on osx run the script `./osx_vst_bundler.sh FeedbackWS target/release/plugin.dylib` and copy
the FeedbackWS.vst folder to your vst folder