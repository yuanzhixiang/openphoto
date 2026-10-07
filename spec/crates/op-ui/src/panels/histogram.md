# panels/histogram.rs: Histogram panel

## Component responsibilities

Compact view: the luminance histogram of the composite image (`DocState::composite_histogram`, cached by revision number, counting only pixels with opacity greater than 0), drawn as light vertical lines on a dark gray background, with heights normalized to the maximum count.

## Known limitations

- No expanded view, channel selection (RGB / Red / Green / Blue / Colors), statistics, or cache warning.
