# Managing Input in Bevy: walking a series of complications

An input manager fundamentally has one job:

"The user has pressed some buttons: what should the player do?"

This seems like it should be an easy problem!
We can define **inputs** are "real buttons that a human pressed",
while **actions** are effectively "virtual inputs",
abstracting over the messy reality of rebinding and hardware.

Then surely basically all you need is a map-like lookup:

```rust
#[derive(Resource)]
struct InputMap(HashMap<Input, Action>);
```

Unfortunately, this is but a beautiful, simple lie.
The reality of the domain is much more complicated.

As we work through these complications, add features, and design Bevy's ideal input management library, we want `bevy_actions` to embody Bevy's deceptively simple philosophy: approachable and pleasant, but rich and reliable enough to actually *trust*.

Input mapping serves a critical functional role in accessibility and customization.
But beyond that, it should allow developers mostly ignore the complex, often irrelevant details of working with input.

Our central design challenge is preserving the illusion,
letting our users believe that beautiful simple lie,
and only unfolding machinery as the user encounters problems in their own projects.
Let's begin!

## Key decisions (TL;DR)

- **Do not upstream** an existing input management solution directly, instead rebuild incrementally
  - the problems and limitations are not cleanly isolated to the input mapping layer
  - none of the existing input mapping libraries do precisely what we want: challenges with complexity and serialization are dominant
  - the core logic is typically simple (if heavy on edge cases): rebuilding will not be more expensive than incremental upstreaming. Design and review costs, not coding costs, dominate.
  - features, tests, and edge case handling should all be borrowed ad hoc
  - severe breakage of existing users is bad, and creates conflict with both happy users and ecosystem maintainers
  - we should make a good solution for *Bevy's* needs, rather than trying to obsolete other crates
- **Layered architecture** with unidirectional data flow
  - empower `bevy_input` as the home of input unification
    - input device hardware corrections, like gamepad drift
    - unified buttonlike / axislike etc types, defined as enums
    - derived inputs like "hold" and "tap"
- **State-machine model of actions**, emitting observable events on transitions
  - users can read the action's state (using systems) or listen to events caused by the change in state (using observers)
- **Simple core, unfolding complexity**
  - common cases never require knowing about advanced features
  - fully optional, lightweight contexts for managing the set of active actions
  - opt-in local multiplayer
  - prefer simple helpers and usage examples over adding more features where viable
- **Serialization of the input map is the most important technical concern**
    - key requirement, but very easy to complicate by being clever
    - no entity hierarchies for actions, no trait objects, just maps and enums
- **No input consumption**; instead decouple routing, arbitration and exclusivity
  - send input events to focused entities, then bubble up, with a default `InputRoot`
  - integrate `bevy_picking` into broader input paradigm
- **Use `bevy_actions` inside of Bevy itself** to allow for standardized rebinding
  - applications like `bevy_inspector` and `bevy_editor`
  - devtools like the render overlay or camera controllers
  - foundational libraries such as `bevy_ui`: `Activate`, `Cancel` etc as actions
  - random hotkeys in assorted examples

These decisions yield the following basic architecture:

- one central `InputMap` resource
- enums for inputs, like early versions of LWIM
- types for actions, like in BEI
- per-actor `ActionState` components
- listen for actions using observers; check action state using systems
- contexts are simple and optional, used for enabling and disabling groups of actions

See [layered-architecture](layered-architecture.md) for a more philosophical treatment of the design,
and [core-decsions](core-decisions.md) for rationale and prior art.
