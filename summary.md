# Managing Input in Bevy: walking a series of complications

An input manager fundamentally has one job:

"The user has pressed a button: what should it do?"

This seems like it should be an easy problem!
Surely the heart of this is just a map-like lookup:

```rust
#[derive(Resource)]
struct InputMap(HashMap<Input, Action>);
```

Unfortunately, the reality is much more complicated.

As we work through these complications and design our ideal input management library, we want to try to preserve the beautiful simple lie as best as possible,
only unfolding the machinery needed to address each as the user encounters them in their own projects.

## Key decisions (TL;DR)

- do not upstream an existing input management solution directly, instead fix and build incrementally
  - the problems and limitations are not cleanly isolated to the input mapping layer
  - severe breakage of existing users is bad, and creates conflict
- empower `bevy_input` as the home of input unification
  - input device hardware corrections, like gamepad drift
  - unified buttonlike / axislike etc types, defined as enums
  - derived inputs like "hold" and "tap"
- timestamped input messages drive stateful actions
  - users can read the action's state (using systems) or listen to events caused by the change in state (using observers)
- each input device gets its own abstract entity, like gamepads, with `bevy_input`-layer support for multiple KBM etc.
- simple core, layered complexity, unfolding as needed
  - prioritize clear, reliable serialization of the input map as the dominant structural concern
  - fully optional, lightweight contexts for managing the set of active actions
  - no entity hierarchies for actions, no trait objects, just maps and enums
  - no input consumption, instead decouple routing and arbitration
  - prefer simple helpers and teaching examples over adding more features
- keep `bevy_actions` lightweight: reuse in other engine crates to unify mental model and rebinding
  - applications like `bevy_inspector` and `bevy_editor`
  - devtools like the render overlay or camera controllers
  - foundational libraries such as `bevy_ui`: `Activate`, `Cancel` etc as actions
  - integrate `bevy_picking` into broader input paradigm, rather than standing alone

Basic architecture:

- one central `InputMap` resource
- enums for inputs, like early versions of LWIM
- types for actions, like in BEI
- per-actor `ActionState` components
- listen for actions using observers; check action state using systems
- contexts are simple and optional, used for enabling and disabling groups of actions

## Architecture

In Bevy, input data flows through a pipeline, as we transform data from its raw form into more abstract and semantically meaningful forms.

Architecturally, there are four layers that Bevy cares about:

- **raw input layer** (`bevy_winit`, `bevy_gilrs`)
  - faithfully passes through information from the operating system, read from physical input devices
  - sends only events: press, release, joystick moved, gamepad connected etc
- **input unification layer** (`bevy_input`)
  - reads raw input events from the raw input layer
  - corrects for hardware quirks like gamepad drift
  - abstracts across operating systems
  - provides convenient helpers to evaluate the *state* of the input devices
  - *New:* provides a **unified input event** interface over inputs which have the same *shape*: buttonlike, axislike, etc.
    - reflectable, serializable, plain-old data, likely a set of enums
  - *New:* processes message streams into more complex input states, such as gestures or held buttons
- **input mapping layer** (`bevy_actions`, `leafwing-input-manager`, `bevy_enhanced_input`)
  - maps unified input events to end-user-defined actions
    - passes input-derived information such as "direction", "magnitude" or "duration" into user defined actions
  - provides an interface for the game logic layer to both check the state of actions, and respond to changes in the state of actions
  - provides tools and patterns for managing when actions are active
  - provides a serializable mapping data structure, suitable for persistence
    - provides logic for verifying correctness of this map, and makes rebinding easy
- **game logic layer** (user games, `bevy_editor`, `bevy_input_focus`)
  - defines a set of actions
  - controls when various actions are active
  - translates "this action has occurred" into actual gameplay effects like "jump"
  - implements rebinding GUI and persistence strategy, learning from examples given by the input mapping layer
  - looks up the inputs bound to a given action when needed for user-facing prompts (e.g. hotkey hints)

Input mapping is a critical accessibility and customization tool.
Yet more than that, it is a way to manage the complex reality of the domain.

As you move up the layers, the lower layers should become invisible to you:
instead of worrying about touch support and chords and conflicting inputs and keyboard layouts in every widget you write, simply make an action and trust that it has been handled well. 

By moving more of the logic traditionally handled by input mappers into our input unification layer (`bevy_input`),
we can simplify their design substantially, and make it easy to both create and swap input mapping solutions to fit the tastes and needs of each project.
