# Work plan

Our goals here are:

- small-to-medium, reviewable PRs
- add core functionality first, increasing sophistication over time
  - ship a working MVP ASAP: bind inputs, check actions, buttons only, single player, not serialized
- design with future plans in mind, so more complex features can be added without breakage (or with manageable breakage)
  - go straight for the final user-facing API where possible
  - new enum variants, struct fields, optional annotations etc are totally fine

The various `Step` work items should be done roughly sequentially, as they form the core of the library, and often build on each other.

The `Sidequest` items offer useful but niche supporting functionality, and can typically be done at any point after the MVP.

## Step 1: Unified input

1. Create unified input enums inside of `bevy_input`. See WI2 for details.
2. Give each input device its own entity, like `Gamepad`, to improve conistency and flexibility.

## Step 2: MVP

Create:

- `bevy_actions` crate, with good library docs that explain the core model
- `Action` trait (just a marker for now), with a trivial derive macro and action registration (WA1)
- `InputMap` resource (WM1) and fallible binding API (WM5)
- `ActionState` component (D4)
- simple `Active`/`Inactive` `ActionLifecycle` enum (D4)
- core input mapping systems, running in `PreUpdate`
- basic usage example

Omit:

- chords
- local multiplayer
- serialization
- conflict resolution
- disabling and contexts

## Phase 3: Events

Add the basic state-machine events for moving through the action lifecycle discussed in D4.

Make sure that observers work as expected.

## Phase 4: Chords

1. Add support for chorded inputs (WI5).
2. Handle clashing inputs simply, according to the longest input, then by `ActionId` (stable but arbitrary).
3. Add an optional priority value to `Action`.
4. Optimize checking for clashes using caching.

See D6 for rationale, and check LWIM + BAM for the actual algorithms.

## Phase 5: Beyond Buttons

Add support for binding and accessing shapes of actions beyond buttons.

Add derive macro helpers for controlling the shape of inputs.

## Phase 6: Presets

Add a preset concept, as discussed in R8.

## Phase 7: Serialization

1. Add feature-flagged serde derives *everywhere*.
2. Add versioning fields, per R9.
3. Add a clear and simple example of how to do this.

## Phase 8: Disabling actions

1. Add contexts and associated API.
2. Add disabled / enabled events, per D4.
3. Add `require_reset`.
4. Add an example.

## Phase 9: Local multiplayer

1. Add `Owner` and `PlayerId`, per WM4.
2. Add an `InputRoot` entity.
3. Refactor `InputMap` data structure and systems to make these work with multiple actors.
4. Add an example.

## Phase 10: Universal input bubbling

1. Make picking events bubble up to `InputRoot`, after the `Window`.
2. Move core `InputFocus` types out of `bevy_input_focus` and into `bevy_input`. 
3. Create `InputEvent`, an event form of `InputMessage` (WI2).
4. Refactor input map resolution to process inputs one acting entity at a time.

See D6 for discussion on this approach, and its advantages over input consumption.

## Side Quest T: Timestamps

1. Add fields for timestamps to `bevy_input` events, populated with `Time` stamps by default.
2. Propagate these into `bevy_actions` events.
3. Try to get better timestamps out of our hardware/OS-level input libraries.

## Side Quest E: Examples

1. Basic example of usage.
2. Examples for each of the core functions: disabling, contexts, modifiers, dead zones etc.
3. Polished keybinding example for people to copy-paste as a base, which works with serialization.

## Side Quest N: Networked Multiplayer

Make it easy to send actions across the network.
See LWIM's design for this.
Include an example.

## Side Quest C: Device calibration and tunables

1. Add tools for defining device-specific deadzones and drift compensation to `bevy_input`. Ensure this information is serializable.
2. Add a ready-to-copy example for easy calibration.
3. Add a simple, standardized collection of tunables (see I20, BAM's tunables and BEI's conditions). Ensure they can be bound and serialized correctly.

Add examples for fancier, game-specific effects like oddly-shaped dead zones.

## Side Quest V: Virtual Inputs

Add enum variants and helpers for constructing inputs out of inputs of other shapes (e.g. virtual D-pads from WASD). See I18 for the design.

## Side Quest A: Automation

1. Add helpers to send actions over BRP.
2. Add an example demonstrating this, and a simple integration test of `bevy_actions`.
3. Add an example demonstrating how to use action-based bots in a very simple game.

See I6 for more discussion on automation.

## Side Quest M: Migration

Add an example of migrating a serialized input map from one version of a game to the next, using the tools outlined in R9.

## Side Quest H: Held Inputs (and friends)

Add more complex, timing-based inputs (e.g. `Hold`) that can be bound.
Update the state machine to add a `Tentative` variant,
and emit `Cancelled` and `Started` events.

See D4 and I19 for discussion.

## Side Quest B: Action Buffering

Add action buffering (A10).

## Side Quest R: Reflection

Make sure everything is reflectable, with appropriate feature-gating.

## Side Quest W: Windowing Loss of Focus

Ensure that inputs are released when window focus is lost.
Reuse `require_reset` flag for actions from D4.

See I5 for discussion.

## Side Quest L: Layered Architecture

Move and adapt the [Layered Architecture](layered-architecture.md) design doc into the library docs of `bevy_input`.
