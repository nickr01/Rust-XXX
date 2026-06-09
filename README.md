
# RustXXX README

## Status

transport is working
proto_FT8 is WIP and incomplete

## Acknowledgement

This project, standing on the shoulders of giants, builds on the amazing work of Joe Taylor - K1JT, Jordan Sherer - KN4CRD, and many other contributions.

- Reference to Wjst-X code and documentation is acknowledged.
- Reference to JS8Call development source particularly the C++ library ports is also acknowledged.
- ft8-lib by Kārlis Goba is a low resource C implementation library was referenced and is acknowledged. [ft8_lib](https://github.com/kgoba/ft8_lib "FT8 (and now FT4) library")
- RustFT8 by Minoru Tomobe JL1NIE is also acknowledged. [RustFT8](https://github.com/jl1nie/RustFT8). This is an implementation of ft8-lib in Rust, which has been forked into this project and then substantially modifed with addition of small translated code contributions from the other sources where required.

## Aims

- Explore, document, and test low bandwidth noise tolerant link layers based on FT4/FT8/JS8 type modulation.

- Begin to move from dependancies on code as documentation.

- Provide a reference library. The focus is internal and loopback testability, and parametrisation to allow relative performance measurements.

- Clarity trumps optimisation. Optimise last (and in forks if any!).

- Time synchronised encoding and decoding to be optional and selectable, allowing use in applications with very long latencies and low clock provision resources.

- Allow input and output via WAV files as well as streaming audio for repeatable testing.

- Automatic samplerate and channel number conversion is provided internally.

- Float32 for all internal signal formats. The dynamic range removes the need for critical input level settings and can in turn fully accept the wider sample formats available from some USB streaming receivers and digitisers.

- Cross platform and light-weight dependencies for audio streams and debug.

## Internal

- The modulator and demodulator stack have been separated into layers, with up and downstream arms (Demodulate/Modulate), but also loopback within layers.

- The intention is only to provide a transport layer library with protocol agnostic binary messages being consumed and produced at the API boundary.

- The Modem has been separated into a sublayer stack for clarity and to allow round trip tests at each layer:
    1. audio
    2. FSK8
    3. gray code
    4. sync - costas
    5. ecc
    6. crc
    7. API

- The receiver pipeline code concept is:
    1. Pipeline Input
    2. Receiver - orchestrate...
    3. Detector - (FFT)
    4. Correlator - find sync candidates
    5. Decoder
    6. -> Demodulator arm of the Modem

## External

- Higher level external layers should encode/decode the FT4, JS8 variants and other development protocols. Encode and decode of FT8 messages is currently linked for development convenience but will be removed.  

- Architecture allows consideration of use to implement Delay Tolerant Networks leveraging the large amount of work in this field, with Amateur Radio callsigns as end point designators.

## Side Benefits

- Rust learning for me. I've had 35 years of Fortran, C and C++. So it's time to move on, and yes, at the moment it's all about me.
- The code here is not entirely Rust idiomatic (yet).
- The toolchain is refreshingly simple.
- Pairing with many IDEs means refactoring is a joy. There has been much refactoring and there will be more.
- The memory safety and low resource use in embedded arm-core systems will likely be competitive with C and assembler.
- Stream based vs block based decoding may better spread CPU load.

## Current State

- Unit testing is passing except for some device name parsing.
- Streaming non time-sychronised decoding is working and tested, but only for FT8.
- Roundtrip audio loopback test is nearly there. Currently this loop is being short circuited from FSK input translated to into likelihood ratios inserted into the demodulator chain.
- Protocol and runtime parameters are contained in rustxxx.rs

## TODO

- Think of a better project name!!!
- Document how to BUILD & RUN.
- Actually re-document protocols.
- Refactor and layout as a library rather than an app. The main app becomes an Example sub-project.
- FT8 audio modulation is working is not fully tested yet via audio loopback. (Loopback via likeliehood ratios is tested).
- Test other protocols.
- Generate the ECC tables as required for other protocols.
- Audio output provision is incomplete.
- Audio sample format conversion is incomplete.
- Some execution paths need to be tidied up.
- Add runtime subtractor pass functionality.
- Add some threading for architectural reasons.
- The code remains messy with lots of residual commented sections.
- Add noise and propagation simulation to audio test paths.
- Alternative correlator strategies.
- Better documentation.
- More I haven't thought of.

Hope this is all useful in some way.

73 de Nick VK2ZTY
