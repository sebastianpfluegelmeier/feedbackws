# FeedbackWS
A waveshaper for people who like weird math functions, feedback cycles and hate their ears and speakers.

## What you need to install FeedbackWS
- git
- cargo
- the (https://github.com/overdrivenpotato/rust-vst2/blob/master/osx_vst_bundler.sh "osx_vst_bundler.sh") script if you are on a mac

## How to install FeedbackWS
- open a terminal
- type `git clone https://github.com/sebastianpfluegelmeier/feedbackws'
- enter the directory with `cd feedbackws`
- type `cargo build`
### On Linux
- copy the artifact into the vst folder `sudo cp target/debug/libfeedbackws.so /usr/lib/lxvst`
### On OSX
- copy the osx_vst_bundler.sh script to your current working directory
- run the script `./osx_vst_bundler.sh FeedbackWS target/release/plugin.dylib` and copy
the FeedbackWS.vst folder to your vst folder
