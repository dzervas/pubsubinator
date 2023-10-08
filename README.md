# PubSubinator

An firmware for various input hardware devices

## Why the name?

GitHub co-pilot chat suggested it because:

It combines "Rust" and "rover" to suggest a sense of exploration and control,
which could be fitting for a keyboard and mouse firmware.

## Compile and view logs

```bash
DEFMT_LOG=debug cargo flash --chip nRF52840_xxAA && probe-rs attach --chip nRF52840_xxAA target/thumbv7em-none-eabi/debug/pubsubinator
```
