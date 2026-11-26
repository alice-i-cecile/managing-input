## Complications

Beyond the central questions discussed in [core-decisions](core-decisions.md), there are a number of frustrating complications; real-world problems that previous attempts have stumbled on.
We need to be mindful of these during both design and implementation.

We can think about these problems by organizing them by what they affect:
the raw **input**, the runtime **mapping** step, **actions** with gameplay semantics, and **rebinding the map**.

### Input

#### I1: How do we correct for input-device-specific quirks?

GAMEPAD DRIFT.

KEYBOARD LANGUAGES.

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I2: How do we handle cases where we need precise timing information?

**Status:** TODO

**Problem:** Rhythm games, and even high-precision networked games like fighting games, desperately want to know *when* input events are from. This also applies to things like drawing applications, and timing information is essential to let us chronologically order our `InputMessage` stream.

**Prior art:** TODO

**Solution:** Way back in WI2, we promised an `InputTimestamp` type.

In theory, this is a good, uncontroversial idea.
Implementation however is somewhat tricky.

There are two core problems:

1. Not all input events have native timestamps, especially across platforms.
2. The various clocks that assign timestamps are not *synchronized*.

TODO

#### I3: Which `Time` variant should be used for timing information?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I4: What if an input is pressed and released in the same frame?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I5: How do we deal with missed release events, such as due to window focus loss?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I6: How do we test our applications using synthetic inputs?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I7: Where do we draw the line between "derived inputs" and gameplay logic?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I8: How can `bevy_input` better support alternate raw input layers?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I9: How should Bevy's solution deal with below-game input mapping layers?

Steam Input, OpenXR, etc.

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I10: How do we detect the "currently active" input device?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I11: How do we manage differing units across input devices?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I12: How does input device output (e.g. rumble) fit in?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I13: How do we attempt to maintain a stable device identity across game sessions and disconnects?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I14: How should local multiplayer per-player input focus be handled?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I15: How should we handle the `Key` vs `KeyCode` distinction?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### I16: How do we handle chords that should be agnostic to left vs right modifier keys?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** These can be represented as derived `ButtonControl` inputs, per WI6.

```rust
enum ButtonControl {
  ..
  Modifier(ModifierKey),
}

enum ModifierKey {
  Ctrl,
  Alt,
  Option,
  Shift,
  Os,
}
```

See WI7 for discusssion on how chords are defined more broadly.

### Mapping

#### M1: How do we deal with overlapping inputs?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### M2: How do we deal with the same input meaning different things to two active contexts? 

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### M3: How do we handle multiple inputs driving the same action simultaneously?

What happens when opposing directions are held at once?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### M4: What happens to actions when inputs are rebound, or actions are enabled / disabled?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** EVALUATE ONCE PER FRAME TO AVOID CASCADES. This is the same idea and pattern as Bevy's dedicated state transition schedule.

#### M5: What are the exact semantics of when chords fire?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### M6: How do we provide insights to developers on why their action did / did not fire?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

### Actions

#### A1: How do we support users who want to process actions inside of the `FixedMain` loop?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A2: How do we control the order in which the game logic layer responds to actions?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A3: How do we configure the input related behavior of individual actions?

SENSITIVITY CURVES, PRESS DURATION, DEAD ZONES AND SNAPPING.

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A4: Where do we draw the line between "dynamically controlling active actions" and gameplay logic?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A5: What do we send over the wire for networked multiplayer games?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A6: How do GUIs relate to inputs and actions?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A7: How does `bevy_picking` fit into all of this?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A8: How should tests or game AI drive actions?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A9: How do we model parameterised or runtime-defined actions?

E.g. ARPGs

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### A10: How do we handle action buffering, so slightly-mistimed actions still count?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

### Rebinding the Map

#### R1: How do we save the mapping to disk? 

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R3: Should we be saving full input maps, or just the diff from a preset?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R4: How should we handle added or removed actions when serializing and deserializing?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R5: How can libraries expose their actions to applications in order to make them rebindable? 

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R6: With respect to input rebinding menus, how much should Bevy itself ship? 

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R7: How can we dynamically generate input binding menus?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R8: Is there a one-to-one correspondence between actions and what players can rebind? 

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO
