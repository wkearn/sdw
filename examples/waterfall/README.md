# A waterfall viewer for sidescan sonar data

This program uses sdw, [winit](https://github.com/rust-windowing/winit), [wgpu](https://github.com/gfx-rs/wgpu), and [vello](https://github.com/linebender/vello) to render sidescan sonar data in a waterfall plot.

Currently `waterfall` displays Edgetech JSF files that contain a single sidescan sonar frequency. This is not a fundamental limitation of the software, and functionality for additional data formats or multifrequency systems will be added in the future.

## Usage

```
Usage: waterfall <PATH>

Arguments:
  <PATH>  The path to a JSF file to display

Options:
  -h, --help  Print help
```



