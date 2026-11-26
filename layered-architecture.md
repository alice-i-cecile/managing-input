# Layered Architecture

In Bevy, input data flows through a pipeline, as we transform data from its raw form into more abstract and semantically meaningful forms.

We can think about this as four distinct layers, each building on the last:

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

As you move up the layers, the lower layers should become invisible to you:
instead of worrying about touch support and chords and conflicting inputs and keyboard layouts in every widget you write, simply make an action and trust that it has been handled well.

By moving more of the logic traditionally handled by input mappers into our input unification layer (`bevy_input`),
we can simplify their design substantially, and make it easy to both create and swap input mapping solutions to fit the tastes and needs of each project.
