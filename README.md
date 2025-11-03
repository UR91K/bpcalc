# bpcalc

a harmonic pickup position calculator for stringed instruments.

## what does it do?

this tool helps find optimal pickup positions on stringed instruments (like guitars or basses) by analyzing where harmonic content is greatest. it calculates these positions by summing weighted sine waves representing different harmonics and identifying peaks using basic peak detection.

## how it works

1. calculate anti node positions for harmonics 2-7 along the string
2. combine together sine waves with adjustable weights for each harmonic
3. find optimal pickup positions where harmonic content is maximized (1st and 2nd peaks)
4. show both a heat map and individual harmonic patterns

## Features

- GUI made with egui
- adjustable string length
- adjustable harmonic weights
- heat map visualization showing optimal positions
- automatic calculation of optimal bridge and neck pickup positions
- search limit control to refine search area

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Usage

launch the application and adjust the parameters:

- set your instrument's scale length in millimeters
- control how far from the bridge to search for optimal positions
- adjust the weight of each harmonic (2-7) to emphasize different tonal characteristics

the app displays optimal positions for both bridge and neck pickups, showing distances from the bridge in millimeters and percentages.

## technical

the algorithm uses cosine based falloff to model how harmonic intensity decreases with distance from anti nodes, which gives smooth physically based results.