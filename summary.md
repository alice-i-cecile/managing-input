# Managing Input in Bevy: walking a series of complications

An input manager fundamentally has one job:

"The user has pressed some buttons: what should the player do?"

This seems like it should be an easy problem!
We can define **inputs** are "real buttons that a human pressed",
while **actions** are effectively "virtual inputs",
abstracting over the messy reality of rebinding and hardware.

Then surely the heart of this is just a map-like lookup:

```rust
#[derive(Resource)]
struct InputMap(HashMap<Input, Action>);
```

Unfortunately, the reality is much more complicated.

As we work through these complications, add features, and design Bevy's ideal input management library, we want `bevy_actions` to embody Bevy's deceptively simple philosophy: approachable and pleasant, but rich and reliable enough to actually *trust*.

Input mapping serves a critical functional role in accessibility and customization.
But beyond that, it should allow developers mostly ignore the complex, often irrelevant details of working with input.

Our design challenge is preserving the beautiful simple lie (it's just a map!) as best as possible,
only unfolding machinery as the user encounters problems in their own projects.
Let's begin!

## Key decisions (TL;DR)

- do not upstream an existing input management solution directly, instead fix and build incrementally
  - the problems and limitations are not cleanly isolated to the input mapping layer
  - none of the existing input mapping libraries do precisely what we want
  - core logic is typically very simple: rebuilding will not be more expensive than incremental upstreaming
  - severe breakage of existing users is bad, and creates conflict with both happy users and ecosystem maintainers
  - we should make a good solution for *Bevy's* needs, not "the only input mapping crate you'll ever want"
- clear, layered architecture (see below)
  - empower `bevy_input` as the home of input unification
    - input device hardware corrections, like gamepad drift
    - unified buttonlike / axislike etc types, defined as enums
    - derived inputs like "hold" and "tap"
  - integrate `bevy_picking` into broader input paradigm
  - no input consumption, instead decouple routing and arbitration
- state-machine actions, emitting observable events on transitions
  - users can read the action's state (using systems) or listen to events caused by the change in state (using observers)
- simple core, layered complexity, unfolding as needed
  - prioritize clear, reliable serialization of the input map as the dominant structural concern
    - a key requirement, but very easy to complicate by being clever
    - no entity hierarchies for actions, no trait objects, just maps and enums
  - fully optional, lightweight contexts for managing the set of active actions
  - prefer simple helpers and usage examples over adding more features where viable
- use `bevy_actions` inside of Bevy itself to allow for standardized rebinding
  - applications like `bevy_inspector` and `bevy_editor`
  - devtools like the render overlay or camera controllers
  - foundational libraries such as `bevy_ui`: `Activate`, `Cancel` etc as actions
  - random hotkeys in all of our examples

This yields the following basic architecture:

- one central `InputMap` resource
- enums for inputs, like early versions of LWIM
- types for actions, like in BEI
- per-actor `ActionState` components
- listen for actions using observers; check action state using systems
- contexts are simple and optional, used for enabling and disabling groups of actions

See [layered-archictecture](layered-architecture.md) for a more philosophical treatment of the design.
