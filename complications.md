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
Implementation, however, is somewhat tricky.

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

**Solution:** These can be represented as derived `ButtonControl` inputs, per I18.

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

See WI4 for discussion on how chords are defined more broadly.

#### I17: How do we make the set of possible input devices extensible?

**Status:** Proposed

**Problem:** The world of input devices is limitless. VR gesture recognition? Dance pads? Telepathic theremin? What if our users want to do something *weird*?

**Prior art:** LWIM badly complicated its data model by moving to e.g. `Buttonlike` traits and trait objects to try and support this eventuality. `winit` and `gilrs` both expose `Custom(u16)` variants in their enum to account for this sort of problem.

**Solution:** Just add `Custom` variants to each control type, with custom inputs that can be registered to provide the appropriate metadata.

Key these by `InputId`, hashed from a path, following the design in `WA1`.

#### I18: How can we construct inputs from inputs of a different shape?

**Status:** Proposed

**Problem:** It's common to want to emulate input devices of a different shape: treating WASD as if it were a `DualAxis` control, using gamepad quadrants as if they were buttons, treating gamepad triggers like ordinary buttons.

**Prior art:** LWIM defines custom input types for this, e.g. `VirtualDpad`, which implement the correct trait for the shape of the input. BEI implicitly converts, using [`ActionValue::convert`](https://docs.rs/bevy_enhanced_input/latest/bevy_enhanced_input/action/value/enum.ActionValue.html#method.convert), following Unreal. These mappings are constructed using modifiers, e.g. four `Swizzle` and `Negate` modifiers for the WASD case.

**Solution:** Add explicit enum variants for each one-to-one conversion in the `*Control` enums, storing the original source data. These conversions are for authoring bindings only: they are never found in `InputMessage` streams. For example:

```rust
enum ButtonControl {
  ..
  FromDualAxis {
    control: DualAxisControl,
    wedge_start: Rot2,
    wedge_end: Rot2
  },
  FromThrottle {
    control: ThrottleControl,
    /// The input-specific threshold, below which the button is considered "pressed".
    ///
    /// If `None`, global settings will be used.
    threshold: Option<f32>,
    hysteresis: Option<f32>,
  }
}
```

Add helpers for common cases, such as WASD or arrow keys.

`impl Into` conversions (see WI2) are provided, but only for obvious, lossless cases in the same dimension, to avoid typos or mistakes.

Many-to-one inputs, such as virtual DPads or chords, cause challenges with memory usage and complexity for a high-frequency type, and are deliberately excluded from the `Control` types. See WI4 for more discussion!

#### I19: How do we recognize more complex inputs? Where does this logic live?

**Status:** Proposed

**Problem:** Distinguishing between tapped, held, and flicked controls can produce rich gameplay. These types of control variations are also sometimes used for accessibility purposes, and gestures (both touch and XR) can be considered an extension of this idea.

**Prior art:** LWIM does not handle these directly, but instead exposes information about the action (e.g. timing information) for gameplay logic to consume directly. BEI, drawing from Unreal's Enhanced Input, has an excellent catalogue of [conditions](https://docs.rs/bevy_enhanced_input/latest/bevy_enhanced_input/condition/index.html), which can be applied to both input bindings and actions.

Combining these conditions can be tricky, with the exact behavior given by the fixed [`ConditionKind`](https://docs.rs/bevy_enhanced_input/latest/bevy_enhanced_input/condition/enum.ConditionKind.html) associated with each condition (e.g. `Down` or `Press`).

The core problem here is that we need a way for a flat, data-driven list of conditions to create a boolean algebra:

- **Explicit:** The action fires if any explicit condition fired, so explicit conditions form an OR group. This is the default behavior: `Down`, `Press`, `Hold`, `Tap`, and so on are explicit conditions.
- **Implicit:** The action only fires if every implicit condition fired, so they form an AND group. `Chord`, `Combo` and `Cooldown` are implicit.
- **Blocker:** Blockers can stop the action, regardless of any other condition. `BlockBy` is the only example.

Unity's interactions (`Press`, `Hold`, `Tap`, `SlowTap`, `MultiTap`) can also be attached to either a binding or to an action, parameterized by things like "hold time".

Gesture recognition typically operates by giving each gesture its own state machine, which tracks inputs and advances before reporting that a gesture has been made. See D6 for a brief discussion of gesture arenas to resolve conflicts between gestures.

Fighting games have complex inputs (timing windows, directional inputs), but implementation tends to live at the game logic layer.

Game-level accessibility config of this sort, when offered, typically allows users to convert holds to rapid presses or vice versa, swap between holds and toggles, or control windows for timing.

`bevy_picking` various [`PointerPress`](https://docs.rs/bevy/latest/bevy/picking/events/struct.PointerPress.html) entity-targeted events for the entire lifecycle.

**Solution:** Fundamentally, BEI's conditions are trying to do too much. They represent an excellent coverage of *use cases*, but they should not be solved with a single complex design.

Instead, split them like so:

| Category             | Examples                               | Implementation                                                          | Layer             | Why                                                                                                               |
| -------------------- | -------------------------------------- | ----------------------------------------------------------------------- | ----------------- | ----------------------------------------------------------------------------------------------------------------- |
| Chords               | Ctrl + S                               | Baked into the `Binding` type                                           | Input mapping     | Foundational input primitive, and clashes must be resolved holistically                                           |
| Press patterns       | Press, hold, tap, release              | Declared on the action at registration, with enum variants              | Input mapping     | Very complex and flexible, and hold vs tap vs release etc is tied to the *meaning* of each action, not rebindable |
| Combos and cooldowns | Fighting games                         | Game-specific logic, derived from action events                         | Game logic        | Timing details etc are very sensitive; can be derived entirely from action-level events                           |
| Input thresholds     | Gamepad trigger thresholds, deadzones | Throttle and Axis control variants, with global and per-device settings | Input unification | Accessibility setting, and often driven by device normalization                                                   |
| Input filters        | Sticky keys, repeat suppression        | Global setting in bevy_input                                            | Input unification | Accessibility setting, should be uniform across inputs                                                            |
| Gestures             | Swipe left, pinch-to-zoom              | Gesture state machines                                                  | Raw input         | Needs to be fed *in* to bevy_input, shape of gestures varies                                                      |

Accessibility options such as hold-versus-toggle are a per-owner override of the action's pattern, stored beside the map (I20), not binding data.

Overloaded actions on the same control that vary by press pattern require careful consideration for rebinding workflows: see R7 for more discussion.

#### I20: How should we handle clamping, rescaling, flipping etc inputs?

**Status:** Proposed

**Problem:** Modifying raw input values is useful for device calibration (drift, deadzones), sanitization (clamping), player preferences (inverted vertical aim, sensitivity) and gameplay effects.

**Prior art:** BEI has a collection of component-based [`InputModifier`s](https://docs.rs/bevy_enhanced_input/latest/bevy_enhanced_input/modifier/trait.InputModifier.html), which perform these operations on both actions and inputs. `bevy_input` has basic deadzone support, and LWIM has a collection of [input processors](https://docs.rs/leafwing-input-manager/latest/leafwing_input_manager/input_processing/dual_axis/enum.DualAxisProcessor.html).

**Solution:** These are performing three different roles, and do not need or warrant a shared abstraction with its associated complexity.
Splitting these value-modifying operations structurally also helps ensure that the order in which they are applied is automatically reasonable: sanitize first!

Device calibration and sanitization should be handled by `bevy_input`: deadzones, stick drift, clamping, unit conversions for mouse scroll wheel etc, often with per-device settings.

Player preferences should be serialized beside the input map, as a bounded set of "tunables" (borrowing from `bevy_action_map`'s [`Tunable`](https://docs.rs/bevy_action_map/latest/bevy_action_map/mapping/struct.Tunable.html)) for values such as "invert axis", "sensitivity" or `PressPattern` changes for accessibility (see I19).
This allows them to be saved and loaded smoothly, using the same serialization mechanisms.

More gameplay-specific effects, such as applying smoothing or fancy deadzone and snapping shapes, belong at the gameplay logic level, and should be hand-authored by application devs.
These should be taught through practical, scoped usage examples.

#### I21: Mapping input devices to players

**Status:** TODO

**Problem:** TODO

**Solution:** The mapping between input devices and players discussed in WM2 must also be maintained, likely in its own resource, `DeviceMapping`, which could live in `bevy_input` itself.

We should follow BAM's strategy of expanding this beyond gamepads, as the cost is relatively low and the consistency benefits are nice.
Persistent device identifiers (which work to at least some degree across disconnects) would also help improve robustness.

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

#### R2: Should we be saving full input maps, or just the diff from a preset?

**Status:** Proposed

**Problem:** `bevy_action_map`'s design brings up an interesting question:
should we serialize the *difference* from a preset,
or the entire input map?

**Prior art:** TODO

**Solution:** Serializing the difference shrinks the storage,
and allows us to distinguish between user-set and default values (see R8).
This distinction is quite helpful when attempting to migrate (see R9).

By contrast, serializing the entire map is simpler,
allows for easier inspection (just read the whole file),
and does not require us to store and track old defaults to handle migrations.

We think that we should prefer the simpler, more robust design and simply serialize
the entire input map, as the size costs are insignificant.

As discussed in R9, we can preserve the benefits of provenance information by logging this
explicitly in the input map.


#### R3: How should we handle added or removed actions when serializing and deserializing?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R4: How can libraries expose their actions to applications in order to make them rebindable?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R5: With respect to input rebinding menus, how much should Bevy itself ship?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R6: How can we dynamically generate input binding menus?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R7: Is there a one-to-one correspondence between actions and what players can rebind?

**Status:** TODO

**Problem:** TODO

**Prior art:** TODO

**Solution:** TODO

#### R8: How do we handle input binding presets?

**Status:** Proposed

**Problem:** Default input bindings are omnipresent. How should users define these defaults?

Beyond that, it's reasonably common to have *multiple* presets to start with.
Southpaw, Blender-style controls, ESDF inputs and so on.

**Prior art:** LWIM handles this very simply: simply construct an `InputMap` however you want,
typically via a method or three on your `Action` enum.

BEI's examples suggest using a settings struct (see WM4 prior art), with a `Default` trait impl,
although constructors could obviously be used there too.

`bevy_action_map` has a distinction between the default set of action bindings,
and a separate [`Preset`](https://docs.rs/bevy_action_map/latest/bevy_action_map/preset/struct.Preset.html) type,
which overrides the default settings, and can later be overridden in turn by the player's own settings.

This distinction seems odd at first, but BAM's defaults are created by the bindings added to each context via code.
During this binding process, tunables and the "shape" of each action are set, causing it to function more like a schema.
By contrast, BAM's presets are a set of diffs, like user changes to bindings, applied to *modify* the defaults.

In all three cases, these defaults are typically defined in code, rather than assets.

**Solution:** Code-first defaults are really convenient for getting started,
but struggle with more complex use-cases: editor-driven changes to the defaults,
user-defined presets, presets for mods and extensions.

For those, you really want *asset*-based presets, for a *data-driven* model.

Thankfully, we already need a way to serialize and deserialize an input binding:

```rust
  /// An [`InputBinding`] as written to a file.
  ///
  /// Used for both the serialization of [`InputMap`] data structures
  /// and the construction of [`BindingPreset`] types.
  ///
  /// Contains stable paths only for ease of serialization; no runtime ids.
  /// Like usual, default field values can be skipped during serialization,
  /// and redundant enums can be collapsed down to only their unambiguous interior form for brevity.
   #[derive(Reflect, Serialize, Deserialize)]
   pub struct BindingRecord {
      /// The input bound to an action.
      pub binding: InputBinding,
      /// The action that this input is bound to.
      pub action: ActionPath,
      /// The owner whose actions these control bindings belong to.
      ///
      /// See WM2 for more information on local multiplayer.
      pub owner: Owner,
      /// The "slot" (e.g. first vs second binding) that stores this binding.
      pub slot: u16,
      /// Where this input binding came from.
      ///
      /// In a preset, this is e.g. `BindingOrigin::Preset("default")`.
      /// See R9 for more information on why this field is useful.
      pub origin: BindingOrigin,
   }
```

Then, the preset is basically just a pile of serializable bindings:

```rust
/// A set of input bindings, defined in advance for users to customize.
///
/// The most common form is a simple default set of input bindings,
/// but these can be used to good effect to offer a number of popular control schemes.
#[derive(Asset, Reflect, Serialize, Deserialize)]
pub struct BindingPreset {
  /// A stable identity for this preset,
  /// which should map onto where it is stored.
  ///
  /// For example: "player/default", "southpaw", "mod.steam_deck".
  pub path: PresetPath,
  /// The input bindings set by this preset.
  ///
  /// Note that this is sparse:
  pub rows: Vec<InputBinding>,
}
```

On the data-driven path, you load the `BindingPreset` like any other asset,
then call `InputMap::apply_preset(path: impl Into<PresetPath>, presets: &Assets<BindingPreset>)`
to set up your input map with the selected set of defaults.

This may seem fussy for the code-only path, but that's nothing a couple helpers can't solve.
If you want to declare this as an asset to enable later selection / extension:

```rust
fn define_default_input_bindings(mut presets: ResMut<Assets<BindingPresets>>){
  let default_bindings = BindingPresets::new("default")
    .bind(KeyCode::Space, Jump)
    .bind(KeyCode::Enter, Interact)
    .bind(MouseButton::Primary, Shoot);

  presets.add(default_bindings);
}
```

Or to bypass the preset system completely:

```rust
/// This system must run *before* any saved player modifications to the keybindings are loaded,
/// or they will be overwritten.
fn define_default_input_bindings(mut input_map: ResMut<InputMap>){
  input_map
    .bind(KeyCode::Space, Jump)
    .bind(KeyCode::Enter, Interact)
    .bind(MouseButton::Primary, Shoot);
}
```

#### R9: How can app developers migrate their input maps between versions of the game?

**Status:** Proposed

**Problem:** Suppose a player defines some custom keybindings in your game.

Your game releases a new version, removing an action that was added, renaming another, and adding a new one.
How do you avoid forcing the user to remember and reset all of their key bindings?

This is a minor problem for games, but *much* more important for complex, long-lived applications, like editors.

**Prior art:** LWIM does not particularly consider this problem. BEI leaves serialization to the end user.

`bevy_action_map` does best, with a numeric version field for the `action_map_version`.
This covers the version of the *action mapping* crate (useful, in case the schema/strategy changes),
but does not cover the more common case where the actions themselves change.

**Solution:**

The most important thing we need to do, immediately, is record the exact version of both the user app *and* the library that was used,
in the serialized data.
With that in place, migration *can* be solved without guessing,
but if it's only added later, earlier versions are orphaned.

```rust
pub struct InputMap {
    /// Which version of the application produced this input map.
    ///
    /// This field is written by the application: release numbers or commit hashes are typical,
    /// although encoding more complex information such as the set of active extensions can also be useful.
    /// It is serialized, and used to aid migrations.
    pub app_version: Version,
    /// The version of the `bevy_actions` crate used to record this hash.
    pub schema_version: Version,
    /// The preset input map that these bindings were originally derived from.
    ///
    /// Serializing this allows us to make a good guess for how to handle renamed or added actions.
    pub preset: Option<String>,
    ..
}

/// We need more flexibility than a simple number,
/// but surely we can do better than a raw string.
enum Version {
  /// A numbered, released version
  Release {
    major: u32,
    minor: Option<u32>,
    patch: Option<u32>,
  },
  /// An arbitrary identifier string for a release.
  ///
  /// Commit hashes are common here, but stranger versioning schemes can be shoe-horned in here too.
  Identifier(String),
}
```

It's also helpful to record *where* bindings came from (inspired by BAM's preset concept).
This should be tracked on both the input map itself (see the `preset` field above),
and on individual rows so migration schemas can make smarter decisions about how to upgrade.

```rust
/// A single binding for an action, as recorded in an [`InputMap`].
#[derive(Reflect, Serialize, Deserialize)]
pub struct BindingRow {
    /// Where the binding comes from.
    ///
    /// This information is useful when determining how to handle migrations,
    /// both between versions of the game, and when swapping presets.
    pub binding_origin: BindingOrigin,
    ..
}

/// The source of a value in a [`BindingRow`].
#[derive(Default)]
pub enum BindingOrigin {
    /// Derived from the preset.
    #[default]
    Inherited,
    /// Explicitly set by the user or player.
    ///
    /// When migrating, you should avoid changing these values.
    User,
}
```

Like usual, we can skip `BindingOrigin` data if it's `Inherited` to cut down on noise.

Actually handling migrations is a fairly bespoke operation that is best left to users,
or later in this library's lifecycle.
To start, explicitly logging the versions and providing an example of how such a migration *could* be handled
is sufficient.
