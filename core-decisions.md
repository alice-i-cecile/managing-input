## Core Design Questions

At the heart of the design, a few key decisions about data structures dominate the shape of our solution.

The questions that drive these decisions follow below, with the following structure:

**Status:** Open/Proposed/Consensus/Accepted

**Problem:** Why this arises, and why it breaks the naive solution.

**Prior art:** Notes on how (and why) previous input management solutions within the Bevy ecosystem and outside it addressed this problem.

**Solution:** The solution that we think Bevy's first-party input management should take, roughly sketched.

### D1: What is an input?

#### WI1: What kinds of inputs exist?

**Status:** Proposed

**Problem:** Not all inputs are created equal. Questions that you might ask about a button make no sense to ask about a throttle, and questions about a gamepad stick don't apply. See I18 for questions about treating certain classes of input "as if" they belonged to another class however.

**Prior art:** LWIM and BEI split these into three categories based on the dimensionality of the input, calling them "buttonlike" (key presses, gamepad buttons, mouse clicks), "axislike" (triggers, throttles, scroll wheels, steering wheel) and "dual-axislike" (gamepad sticks).

**Solution:** Extend this three-way split but split mouse motion and add two more categories:

- scrolling and mouse movement belong to a fourth category, deltas, as they are not semantically equivalent to something like a gamepad stick, and are not similarly normalized
- analog buttons like triggers belong to a fifth category, throttle-likes, as they vary from 0 to 1, not -1 to 1
  
Extend as needed by real users (e.g. 3D inputs for VR).

#### WI2: How should we model input across device types?

**Status:** Proposed

**Problem:** Bevy (as of 0.20) provides a large number of input message types, split by devices. This is very inconvenient for input mapping libraries, as they must recombine these streams and pass them around, and any relative ordering is lost (where was the mouse when it was clicked?).

We can't bind input *messages* to actions: they contain information such as device identity and magnitude that simply isn't appropriate.

**Prior art:** LWIM's former [`InputStreams`](https://docs.rs/leafwing-input-manager/0.9.0/leafwing_input_manager/input_streams/struct.InputStreams.html) type is indicative of the kind of messy glue code required. You can do the same thing in various other ways, but this core input unification problem must be solved in *every* input mapping library.

LWIM allows users to bind `KeyCode`, `GamepadButton`, `MouseButton` and so on, through e.g. `Buttonlike` trait impls, taking an `impl Buttonlike` at the call sites. Previous versions of LWIM took an `impl Into<UserInput>` argument, allowing similar ergonomics.

**Solution:** Create a unified `InputMessage` type that libraries which care about holistic views of user input read:

```rust
/// A unified input message, composited from all connected input devices.
///
/// Within a frame, events are sorted by timestamp.
#[derive(Message)]
struct InputMessage {
  /// Which control was pressed, moved or otherwise activated.
  control: Control,
  /// The normalized value of the message.
  ///
  /// This varies by the "shape" of the input: buttons, throttles and gamepad sticks all store different information.
  ///
  /// Any unit scaling (e.g. for mouse scrolling) or device-specific correction (e.g. for gamestick drift) has already been done,
  /// and values are clamped to the appropriate range.
  value: InputValue,
  /// The input device that created this message. 
  device: Entity,
  /// The time that this message was created.
  ///
  /// This uses OS timestamps where available, and falls back to time of reception when this information is not provided.
  timestamp: InputTimestamp,
}

/// The normalized value of an input device.
///
/// This can either represent a single reported *change* in a device's state or its accumulated level.
/// Be sure to check the documentation for the containing type (e.g. [`InputMessage`]).
enum InputValue {
  /// The state of a button. `true` means the button is/was pressed, `false` means it is/was released.
  Button(bool),
  /// The degree to which an analog button (like most gamepad triggers) or similar is depressed, in the range of 0 to 1
  /// 
  /// 1.0 means the button is fully pressed, while 0.0 means the button is fully released.
  Throttle(f32),
  /// A normalized position along a single axis, in the range of -1 to 1.
  Axis(f32),
  /// A normalized position on two axes, where both axes are in the range of -1 to 1.
  DualAxis(Vec2),
  /// A positional change that has occurred, e.g. via accumulated mouse motion.
  /// 
  /// Unlike the other variants of this enum, changes are rescaled but not clamped to a range.
  Delta(Vec2)
}
```

Use unifying enums with `impl Into` ergonomics, defined one level below `Control`, split by shape.
This keeps nice ergonomics, but prevents users from accidentally binding e.g. mouse motion to jump.

To sketch this more fully:

```rust
/// A kind of user input.
///
/// This type unifies all supported input kinds across Bevy, for storage in cross-device data structures such as [`InputMessage`].
///
/// Each variant represents a different shape of input, corresponding to a variant on [`InputValue`].
enum Control {
  Button(ButtonControl),
  Throttle(ThrottleControl),
  Axis(AxisControl),
  DualAxis(DualAxisControl),
  Delta(DeltaControl)
}

/// A kind of buttonlike input, which can be pressed or released.
enum ButtonControl {
  PhysicalKey(KeyCode),
  Gamepad(GamepadButton),
  Mouse(MouseButton),
  ..
}
```

Note that I18 and WI4 discuss this problem further: covering converted and more complex kinds of controls.

#### WI3: How do we handle multiple input devices of the same kind?

**Status:** Consensus

**Problem:** Multiple input devices of the same kind may exist. Gamepads are the most common, but touch pointers are important, and some OS's support multiple mice or keyboards (even if winit does not).

**Prior art:** In Bevy, each gamepad has its own entity, which stores any metadata about the device. Similarly, `PointerId` is a stable identifier for pointer entities across spawn/despawn cycles.

**Solution:** Do the same for mice and keyboards, assigning each an entitity. Where no identifying information is provided, treat all devices of the same class as a single device.

#### WI4: How are composite inputs, such as chords, represented?

**Status:** Proposed

**Problem:** Being able to bind things to arbitrary combinations of buttons is very convenient, and a common paradigm for GUI applications (think Ctrl+S). This has a few interesting variants: gamepad chords, chords of things other than modifier keys, and cross-device chords, commonly modifier key + mouse button, and non-buttonlike chords, commonly modifier key + mouse wheel. WASD "virtual DPad" controls suffer many of the same pitfalls.

**Prior art:** LWIM originally stored a `UserInput` variant specifically for chords. This worked well conceptually, but meant you could have chords of chords in confusing ways. Later, it swapped to [`ButtonlikeChord`](https://docs.rs/leafwing-input-manager/latest/leafwing_input_manager/user_input/chord/struct.ButtonlikeChord.html) and its `Axislike` equivalents. These are concrete types that hold trait objects.

BEI has a distinct `Binding` enum, storing e.g. `Keyboard { key: KeyCode, mod_keys: ModKeys }` as a variant. Gamepad actions cannot be chorded. This is limited but clear, and handles the common cases well.

However, users can instead use the [`Chord`](https://docs.rs/bevy_enhanced_input/latest/bevy_enhanced_input/condition/chord/struct.Chord.html) condition, a component which references any number of additional action entities, triggering when all of the composite actions fire.
This allows arbitrary flexibility, at the cost of helper action entities and complexity.

Handling of the virtual d-pad class of inputs is covered in I18.

**Solution:** Bindings have a distinct data type from the `Control` types, which track chords via a dedicated modifier field.

```rust
struct Binding<C> {
  primary: C,
  modifiers: SmallVec<[ButtonControl; 2]>,
}
```

There are a few subtle advantages of this design to note here:

1. The same pattern is shared across control types `C`: `ButtonControl`, `DeltaControl` and so on.
2. In every case, the modifier needs to be some form of `Button`, because it must be *held*.
3. We can use a `SmallVec` for the modifiers, making the common case of zero modifiers fast, and a couple of modifiers allocation free.
4. This keeps composite inputs out of `Control` (avoiding self-referential nesting and size issues).
5. The order of the modifiers needs to be *consistent* for display and serialization, but doesn't really matter. Canonicalize and dedupe on construction.

Helper functions and methods are added to make the common case of modifier keys (see I16) plus a primary button terse and easy.

These bindings are then unified into a single enum, `InputBinding` (with the obvious `From` impls), as discussed in `I19`.
See WM1 for more discussion on storing `*Binding` types.
Arbitration of clashing (overlapping) inputs in chords is discussed in D6.
The exact details of "when do chords fire" is in M5.

### D2: What is an action?

Conceptually, an action is a "semantic input".
Freed from the tedium of physical devices and input bindings,
the game logic layer reads actions and responds, changing the game world accordingly.

#### WA1: How do games define actions?

**Status:** Proposed

**Problem:** That conceptual definition is all well-and-good, but concretely, it is not very useful. How can we store it as a value in a map? How can it be serialized?

**Prior art:** LWIM used enums for this: a nice simple list of possible actions that can be taken, with the option to define multiple distinct enums for different non-overlapping sets of actions. BEI on the other hand uses one Rust type per action, typically data-less unit types, allowing for clean integration with observers.

In both cases, we need an `Action` trait to mark this type and carry assorted customized data.

Other libraries, especially outside of Rust, might choose to use strings here: perfectly flexible, run-time definable,
and broadly offer terrible tooling integration (go-to-definition, docs, rename).

**Solution:** The observers UX dominates: we need to use one Rust type per action. That implies an `Action` trait + derive, and some form of action registration.

For registration, we have four options:

1. An explicit `app.register_action` extension method
2. Registration on first mutating access, as Bevy does for components. 
3. Registration on *any* access, via interior mutability.
4. Reflection style auto-registration.

The first is a nice clear API, but presents annoying boilerplate.
Both interior mutability and auto-registration are too complicated (or costly, or brittle) to be worthwhile.
However, first-mutation auto-registration is extremely convenient, and low cost.
This implies that we must store the registry in the `InputMap`.
However, we must supplement it with a rarely used manual registration API,
for unusual setups where the user does not define a default input map in code.

However, strongly typed actions alone are not enough. While they are very convenient for end-user code,
they can complicate the internals substantially (boxed trait objects T_T), often require multiple plugin instantiations and make serialization and advanced use cases (e.g. dynamic actions, highly generic code like networking) more frustrating.

Learning from `bevy_ecs` and `bevy_action_map`, we can use a type-erased `ActionId` as our standard identifier, hashed from the unique path (e.g. `player::jump` or `bevy::ui::activate`) given to each action.
This path serves as a structured human-readable identifier, mostly in type-erased contexts like persistence, networked multiplayer or BRP.
To ensure that this complexity unfolds slowly, the path for each action defaults to its type path, but can be overwritten in the `Action` trait impl.
Use cases that rely heavily on stable identifiers (such as serialization) warn when implict paths are found to avoid headaches caused by incidental refactoring.

#### WA2: How do we record the shape of each action and the acceptable inputs to bind to it?

**Status:** Proposed

**Problem:** Not every input makes sense for every action.
"Move the gamepad" doesn't have an obvious meaning for "jump",
and trying to aim with a single key press simply won't work.

Every action and every input has a shape, and we should ensure that these line up
to avoid nonsensical, broken input bindings.

**Prior art:** LWIM annotates action variants with their shape: buttonlike, axislike etc, via a `input_control_kind(&self)` method on `Action`.
This has typed helper methods, but enforcement is only checked in debug mode.
Why? I don't remember!

BEI records the shape using an `Output` associated type on its trait.
It handles mismatches by automatically converting inputs into the correct shape for the action:
by zero-padding or discarding later values.

`bevy_action_map` records both an [Output](https://docs.rs/bevy_action_map/latest/bevy_action_map/action/trait.InputAction.html#associatedtype.Output) associated type (which determines the type returned when the action is checked) and an  `Intent` type, which determines the acceptable inputs to be bound to it.

Inputs are automatically coerced, more carefully than in BEI, to the correct shape, with errors
on failure.

**Solution:** The LWIM approach of using a method is clearly not suitable here,
as unit-type actions (ala BEI) are dominant.

Instead, we should use a simple associated constant enum (`InputShape`, which can be computed for our various `Control` types to ensure a match).
This is fine: we have no need for object safety, and there's only one kind of action per type defined.

Derive macro helpers are perfectly pleasant, and should be added at some point.

See WM5 for handling shape mismatches during binding.

Inputs can be converted from one shape to another (e.g. a virtual D-pad) (I18).
However, as discussed there, this proposal suggests we should be quite explicit and conservative there,
to avoid quiet user-space footguns and confusion.

### D3: How do we record the mapping between inputs and actions?

#### WM1: What data structure do we use to hold the mapping?

**Status:** Proposed

**Problem:** We need to actually record the bindings somewhere.
This data structure has a few requirements:

1. The input management code needs to read this to map inputs to actions,
2. Players need to be able to read and customize these bindings via a GUI.
3. Games commonly segment the input bindings: usually KBM vs gamepad.
4. Players can often map more than one (most commonly, exactly two) input to the same action.
5. These bindings need to be persisted, including across devices.

**Prior art:** At its heart, this is basically just a serializable map.

```rust
#[derive(Serialize, Deserialize)]
struct InputMap {
  pub action_bindings: HashMap<Action, Input>
}
```

Whenever you want to know if an action is pressed, you look up the bindings by the action key,
then check if the corresponding inputs are pressed.

Supporting multiple bindings requires you to make the value collection-like in some form:
```rust
#[derive(Serialize, Deserialize)]
struct InputMap {
  pub action_bindings: HashMap<Action, Vec<Input>>
}
```

This is, in essence, what [LWIM's InputMap](https://docs.rs/leafwing-input-manager/latest/leafwing_input_manager/input_map/struct.InputMap.html) type is.

`bevy_action_map` takes a slightly different approach;
storing a dense array of actions x slots,
constructed as a shared, compiled `Plan` from an outer map-like datastructure.

`bevy_enhanced_input` by contrast represents these bindings implicitly,
directly in the ECS.
Each binding is stored as a component on its own entity,
joined to action entities via relations,
which are then collected into context entities.
Rebinding is handled by despawning or mutating the corresponding components.

**Solution:** In its simplest form, we want something that looks like:

```rust
/// The input bindings for the application.
pub struct InputMap {
    /// One entry per action.
    ///
    /// Position within the list is the binding's slot,
    /// allowing for multiple redundant bindings for a single action.
    ///
    /// These are Option-wrapped in order to allow empty slots,
    /// and this field is private, in order to allow users to enforce
    /// invariants around the number of "slots" or "columns" on bindings.
    /// 
    /// Making this field private is also useful because it allows us
    /// to keep `ActionId` as a private mechanism, see WM4's
    /// discussion on `ActionPath`.
    /// 
    // Note that this is complicated by WM2
    action_bindings: BTreeMap<ActionId, Vec<Option<BindingRow>>>,
    // See WM4 for other fields on this type
    ..
}

/// A single binding for an action, as recorded in an [`InputMap`].
#[derive(Reflect, Serialize, Deserialize)]
pub struct BindingRow {
    /// The input that should be pressed (etc) in order to activate the action.
    pub binding: InputBinding,
    // See WM4 for other fields on this type
    ..
}
```

Very simple, with room to expand by adding fields (see WM4).
The core model, that bindings are basically just a `HashMap`,
is not *wrong*!

Because this is just an ordinary data structure, with no trait objects or entities involved,
serialization remains very simple, as does editing and learning about the structure.

Entity-based approaches such as in BEI offer integration with Bevy's existing scene tooling
(e.g. the inspector, BSN), but are fundamentally more fragile and harder to reason about.
The ECS does not have the same level of invariant enforcement as plain-old Rust:
enums and fields are incredibly powerful.
Even in a future with high-powered relationships and ECS invariants, those rules will not
be as easy to reason about (or as clearly documentable) as a struct with a bunch of enums.

The choice of map-value in the `action_bindings` field is worth further discussion.
A `Vec<BindingRow>` is both simple and flexible.
That supports a wide array of patterns in downstream projects,
but doesn't structurally enforce the "slots" data model that is popular in games.

Something like a `[Option<BindingRow>; 4]` collection would map cleanly onto the standard "2 slots each for KBM and gamepad", improving robustness at the cost of flexibility.
Note that the choice of `N` in that array cannot easily be made generic,
as that would propagate into the `InputMap` type, and make it impossible to fetch in a single ECS system
without a whole whack of incidental complexity.

The selected `Vec<Option<BindingRow>>` design, with private fields, configuration,
and runtime-enforced invariants. 
This offers a reasonable compromise between enforced strictness and flexibility, at the cost of some complexity.
However, because we want to make this field private *anyways* (see WM4),
the cost is amortized across these design choices.

#### WM2: What about local multiplayer? 

**Status:** Proposed

**Problem:** Much to the dismay of input mapping library authors everywhere, sometimes people want to play a game together
on the same computer, using distinct sets of inputs to control different characters simultaneously.

This complicates *everything*, by removing the assumption that there is exactly one input mapping to worry about.

While the most common case is seperate input devices (mostly gamepads) per player,
local multiplayer that shares a keyboard is a real niche use case that would be nice to support.

Note that action-driven bots (A8) pose similar challenges: they require distinct action states,
but do not require additional input maps.

**Prior art:** In LWIM, the canonical solution to this is to insert multiple input maps and action state components,
one per player. This works smoothly, but attaching the input maps to the player entity has the lifecycle difficulties discussed in WM3.

BEI gives each player their own entity, and then attaches the context components to them, with actions and bindings nested underneath in the usual fashion.
This allows for full customization per player, but like usual, persistence and patterns for setting these bindings are left to the library user.

BAM declares a single set of input bindings across all local multiplayer players, with optional per-device overrides.
Clashing inputs (D6) are resolved at a per-device level, rather than aggregated.

Each library has its own solution to restricting player input across gamepads:

- LWIM allows inputs to be restricted to a single gamepad, defaulting to any gamepad
- BEI tracks this on the context entity, swapping between `Any`, `Single(pad_entity)` and `None`
- BAM supports this for more than just keyboards, allowing distinct e.g. mice and keyboards per player, although `winit` does not support this setup

**Solution:** There are a few moving parts here:

1. Where and how do we store the input bindings?
2. How do we manage multiple action states?
3. How do we assign input devices to specific players?

Conceptually, the answer to the first question is simple: you put it in the input map,
which, per WM3, is stored in a simple resource.
This is essential for ease of serialization (WM4), and allows you to readily detect conflicts between
bindings across players.

Doing so requires keying each action binding (see WM1) by its owner first, and then by action,
as we want to look up "all bindings for each player" in order to resolve conflicts and set up binding GUIs:

```rust
pub enum Owner {
    /// One binding set for the whole application, used by game-agnostic actions.
    Shared,
    /// A distinct binding set for each player.
    Player(PlayerId),
}

// Because this is a BTreeMap, we can fairly cheaply fetch all actions belonging to a specific owner,
// without another layer of actions.
action_bindings: BTreeMap<(Owner, ActionId), Vec<Option<BindingRow>>>,
```

Whether an action uses the shared owner or one owner per player is declared in the `Action` trait:

```rust
pub enum ActionScope {
    Shared,
    PerPlayer,
}
```

Registration rejects rows whose owner does not match the action's scope (see WM5).

`PlayerId` can be defined using exactly the same patterns as `ActionId` etc (WA1),
supporting both type safety and runtime flexibility as desired,
while allowing us to serialize per-player input bindings.

In contrast to the shared input map, each player needs their own distinct `ActionState` (D4),
tracking which actions are pressed, their value, what's disabled (D5) etc.
This is stored as a component on each entity, as  the `PlayerId` required to map players to their bindings.

The device-to-player mapping is discussed in I21, while stable device identity is complication I12.

#### WM3: Where in the ECS do we store the mapping?

**Status:** Proposed

**Problem:** Once we have the mapping, we need to *put* it somewhere. Does it belong in a resource?
A component on the player entity?
A component on *each* player entity (see WM2)?

Do we have a single canonical input map?

**Prior art:** LWIM, historically, allowed users to store this mapping as both a component (e.g. on each player),
and as a single global resource. Resources-as-components made maintaining that flexible setup more challenging,
and it currently only supports the component-based input map, although abstract entities are commonly used.

While storing the input map directly on the player entity makes for very simple examples, and tracking action-state on the same entity as the player is convenient,
this practice tends to lead to serious problems with respect to the player entity lifecycle.
What happens when the player dies and their entity is despawned?
How can we check and set our keybindings in menus, when the player is not spawned?

`bevy_action_map` uses a more elaborate design, splitting input map storage, to accomodate its context-centric binding compilation approach:

1. `InputContextPlan<C>`: one resource per context type, tracks the default input bindings declared for that context.
2. `Overrides`: a struct that tracks player-specific customization of the input map, intended to be stored in your own settings resource.
3. `AppliedPlan<C>`: one resource per context type, stores computed final bindings (defaults + overrides) for that context type. Only exists if overrides have been applied.
4. `InputContextState<C>`: holds an `Arc<Plan>` cloned from an `InputContextPlan<C>` (or `AppliedPlan<C>` if overrides were used), as a component on each player entity, one per context.

There is no single source of truth for "what key bindings are set": we must aggregate across the different registered contexts, and account for the maybe-existence of `AppliedPlan` based on overrides.

BEI takes yet another approach, storing the mapping in the entity graph:

1. At the top level, we have a context entity, with context of type `C` holding the actions that are associated with it via an `Actions<C>` relationship. 
2. Beneath that, we have our action entities, with `Action<A>`, tracking the bindings for that action via a `Bindings` relationship.
3. At the bottom, we have each of our binding entities, with a `Binding` enum component.

This allows us to readily customize and manipulate each of our context/actions/binding entities (e.g. with modifiers and conditions),
but forces every lookup to walk the entity hierarchy and this hierarchy can be invalidated (e.g. by pointing to not-action entities)
as it's simply a convention in the ECS, rather than type-checked.

**Solution:** We do not need to overcomplicate this. Just use a resource.

Many critical operations (settings menus, persistence, checking for "Space is already bound to Jump!"),
rely on fundamentally global knowledge, operating across contexts and players.
Keeping all of the information in a single place makes this very straightforward,
while the standard Rust nature of the setup improves robustness and makes documentation and comprehension easy.

Changing the input bindings is a simple struct mutation (no seperate `Overrides` concept required, no ECS operations),
and these mutations made can be serialized directly (see WM4, R8 and R9).

However, being able to check the state of each action (e.g. "is Jump pressed") on the same entity as the player
is incredibly convenient, both in systems and observers.
To make this work, we split the difference between LWIM's component and resource-based design:
tracking `ActionState` as a component on the entities that actually exist in the game,
while input mapping remains a global resource.

Shared or game-agnostic actions (e.g. `bevy_ui::Cancel`) belong to a spawned-by-default entity,
called `InputRoot`. See D6 for more discussion on this point.

Under a resource-based design, special care is needed to cover local multiplayer, see WM2.
The per-player, component-based input maps simplify questions around local multiplayer (see WM2),
but the lifecycle issues created by that approach make it a footgun in practice.
Storing the *input map* in a resource avoids that,
while tracking *action state* on entity-bound components makes sense: despawned entities should not be taking actions.

Similar care must be taken for the actions of bots (A8), to ensure that input move the active player, not their synthetic peers!

#### WM4: How is this data structure serialized?

**Status:** Proposed

**Problem:** Saving and loading input bindings is a critical part of the lifecycle in any real application.
On paper, this is simple: just slap some `serde` derives on your types and you're done.

Unfortunately, if you complicate the design of your core data structure, this can get much, much harder.

**Prior art:** Early versions of LWIM did exactly that: just serialize some plain old data.
However, later versions, in an attempt to solve input-device extensability (I17),
swapped to a trait-based design.
This worked great for the runtime logic, but badly complicated both performance and serialization.

`bevy_action_map` uses a dedicated `SavedOverrides` type,
recording deviations from the default settings (see R8),
and a version field (more in R9).

BEI by contrast mostly leaves serialization up to the end user.
The [keybinding_menu example](https://github.com/simgine/bevy_enhanced_input/blob/master/examples/keybinding_menu.rs)
suggests creating a simple, parallel `InputSettings` struct,
bespoke to the actions in your game,
integrated with `bevy_settings`.

Each action gets its own field,
with bindings logged as `[Binding; BINDING_COUNT]`.
This is flexible and simple, but leaves many of the hardest problems in this space to the user:
conflict detection, persistence, and mapping from a simple data structure
to the correct collection of entities needed to make BEI work.

Notably, at the serialization level, all libraries appear to converge on string-like keys,
and much of the internal complexity across libraries needs to be collapsed down to allow for persistence,
in one form or another.

**Solution:** Add `Serialize` and `Deserialize` derives on `InputMap`,
echo them down through the tree, and because it's all just enums, everything is smooth sailing.
Well, mostly.

Unfortunately, our requirements for our *runtime* representation of our input map
and its serialized representation vary significantly.

Most notably, in the key of each binding.
At runtime, we want something compact and fast to look up.
While serialized, we want something with a stable, human readable identity, so that:

1. It's human-readable, both when reading/editing the raw file, and when inspecting diffs.
2. It plays nicely with tooling, such as specialized GUI editors.
3. It's stable across changes to your games.

The obvious temptation is to attempt to have your cake and eat it too.
LWIM did this by recording actions as `enum`s: with custom `PartialEq` and `Hash` requirements
for actions which store data (otherwise `Move(-1.)` does not give you the same lookup as `Move(1.)`).

This is something of a mess, and because of our choice in WA1, we cannot do the same here.
Instead, we are forced into two different representations of our keys: the ephemeral `Actionid` of  a tiny `ActionId`, as defined in WA1, and at rest, an `ActionPath`, following `bevy_action_map`.

An `ActionPath` is a simple newtype over a string,
generated, by default, from the type path of the `Action`,
with an override (complete with macro support) for custom, stable identifiers
to avoid churn as the codebase is reorganized.

At first, you might expect that this model meaningfully complicates serialization and deserialization,
forcing the use of some registration scheme.
However, we can do much better.

Because we don't particularly care about what `ActionId` is assigned to each action at runtime,
we can hand them out freely on demand, as actions are registered to our `InputMap`,
obtained by locking down access to the `action_bindings` field.

Users register actions by `ActionPath`, and `ActionId` is incremented and handed out.
That solves deserialization: simply add the bindings one at a time via `ActionPath` in the deserialization path,
not worrying about whether or not the internal `ActionId`s are the same as last time.

However, if we store that `ActionPath` <-> `ActionId` mapping (bidirectionally),
*inside* of the `InputMap`, we can directly serialize the data *back* into `ActionPath` form,
no external registry required.

#### WM5: How do users bind inputs to actions?

**Status:** Proposed

**Problem:** Data structures are all well-and-good, but at some point users need to actual *set* this data.
What should that API look like?

**Prior art:** LWIM uses simple methods on the `InputMap` data structure: one method per shape of input.
At one point, LWIM swapped between `(action, input)` and `(input, action)` ordering.
The order doesn't matter much: pick one and stick to it.

In BEI, bindings are defined by spawning entities with the correct relationship and components.
Both the `bindings!` and `actions!` macros exist to make this easier.

BAM requires users to define a context first, then uses a typed builder pattern:

```rust
app.add_context::<C>(|c|
  c.bind::<Jump>(KeyCode::Space)
  .dead_zone(...)
  .mappable().hold(0.2)
)
```

**Solution:** At the heart of the API, we have a single method: `InputMap::bind(action, input)`.
To make this pleasant to use, `action` is an `impl Action` (WA1) argument, while `input` is an `impl Into<Control>` variant (see WI2).
This gives us a very terse, simple API:

```rust
input_map.bind(Jump, KeyCode::Space);
```

`bind` defines bindings across *all* registered players (or once, for shared actions);
use `bind_for` to bind actions to only a single player.

Unfortunately, it does not make sense to bind arbitrary inputs to each action: each action has its own shape (WI1, I18).
Similarly, some input bindings might overlap existing bindings, in a way that we should reject at the time of binding.
We should return a `Result` from our `.bind` APIs, diverging from existing input management crates, which are more permissive,
and lock down access to the internal fields of `InputMap` to ensure that these invariants are correctly upheld.

This may seem nit-picky, but the actual cost of doing so is not bad!
With the help of chained builders, `BevyError` and the ever-trusty `?`, the final API is genuinely pleasant:

```rust
fn setup_default_keybindings(mut input_map: ResMut<InputMap>) -> Error {
  input_map
    .bind(Jump, KeyCode::Space)?
    .bind(Walk, VirtualDpad::WASD)?
    .bind(Shoot, MouseButton::Primary)?;
}
```

If we want to eliminate an entire class of errors at compile time (e.g. for menu-driven bindings), we can use one of the specialized binding methods, like `bind_button`,
which would take an `impl Into<ButtonControl>` instead, and validate that the action is of the appropriate shape (WA2).

Actions are not grouped into contexts at input binding time (see D5),
nor are the properties of each action (conditions, in BEI's terms) defined there.
Instead, this information about each action belongs to is defined on that *action*,
registered in the `InputMap` with the action.
See I20 for more discussion on this point.

### D4: How does the game logic layer respond to actions?

**Status:** Proposed

**Problem:** "Actions" on their own don't actually *do* anything in the game. Instead, they're a useful,
sanitized abstraction over input: representing the *intent* of a player.

To actually translate them into changes in the game world, the application author must write code
that takes actions as inputs, and turns it into mutataions of the ECS state.

What should that look like?

**Prior art:** LWIM's model here is quite simple: sanitized information about actions is available in an `ActionState<A>`
component/resource.

With an `ActionState` in hand, you can check its state directly in a system, asking questions like "is the Jump action pressed?" or "what is the 2D value of "Move" right now?".
These are straightforward methods on a struct, and operate under a pull-style (polling) paradigm.

With the introduction of observers, BEI took this one step further, allowing users to register observers listening to various events,
responding to actions in a push-style (notification-like) approach, in addition to a state-based polling API (on e.g. `Action<A>`).
Each action emits a number of events across its lifecycle: `Start<A>`, `Ongoing<A>`, `Fire<A>`, `Cancel<A>`, `Complete<A>`.

This vocabulary is *shared* across different shapes of inputs, or the conditions (e.g. hold, repeat) attached,
although it maps most naturally to button-like inputs.

**Solution:** Both push and pull APIs are convenient: pull-style APIs offer a simple model of state and a great deal of flexibility,
while push-style APIs are very convenient and allow users to write complex logic in a very uniform way.

To avoid divergence between these APIs, we should stick to the state-based pull-style API as the primary record of truth,
with push-style events derived from it using systems which run just after it is updated during the input-mapping step.

This is recorded in the `ActionState` of each actor, tracked as a component on the entity that is actually acting (see WM2, WM3).

```rust
/// A collection of [`ActionStatus`], representing the state of each action for a given actor.
///
/// These values are typically populated based on player inputs, processed according to an [`InputMap`],
/// but the status of each action can be driven directly as well.
///
/// Disabled actions remain present, but are [`ActionLifecycle::Disabled`].
#[derive(Component)]
struct ActionState {
  status_map: BTreeMap<ActionId, ActionStatus>,
}

/// The status of a single action, stored in an [`ActionState`].
// The fields on this type are private with getters and setters to ensure consistency
struct ActionStatus {
  /// Where this action is in its lifecycle;
  /// principally, whether or not this action is currently active
  life_cycle: ActionLifecycle,
  /// The value contained in each action, if the action is active.
  ///
  /// Actions can be thought of as "virtual inputs", abstracted over input devices.
  /// 
  /// To treat inactive actions as if they were their "released" or null form,
  /// call `.unwrap_of_default()` on this field's value.
  value: Option<InputValue>,
}
```

The lifecycle design of these events is excellent, and much should be borrowed from BEI.
We can clarify things somewhat though, by:

1. Explicitly modelling the state of each action using a state machine.
2. Only emitting events for *edge* transitions (removing `Ongoing`)

The simplified state machine looks like:

```rust
enum ActionLifecycle {
  /// The action has been disabled.
  Disabled,
  /// No relevant inputs have been detected.
  Inactive,
  /// Some relevant inputs have been detected.
  ///
  /// This might be an input that must be held, a gesture arena,
  /// a multi-step action or something 
  Tentative,
  /// All of the relevant inputs are active
  Active,
}
```

For each of the main edges:

- Going from `Inactive` to `Tentative` emits a `Started` event.
- Going from `Tentative` to `Active` emits a `Fired` event.
- Going from `Active` to `Inactive` emits a `Completed` event.
- Going from `Inactive` to `Active` directly *also* emits a `Fired` event, as distinguishing these cases for a single action is never needed.
- Going from `Tentative` to `Inactive` emits a `Cancelled` event.
- Going from `Active` to `Tentative` is impossible.

Disabling requires its own set of events:

- Going to `ActionLifecycle::Disabled` from any other state emits a `Disabled<A>` event, while the reverse emits an `Enabled<A>` event.
- Going from `Tentative` or `Active` to `Disabled` emits a `Canceled` event
See D5 for more details.

Transitions from a state to the same state never emit events.

This state machine may be complicated later (in terms of library development) by tracking progress through each of these states: things like hold durations or active durations after the action is pressed (see I19),
but the central states and the events emitted on each edge can remain consistent.

Each of these action events are entity-targeted, and contain the following information:

```rust
#[derive(Event, EntityEvent)]
struct Fired<A: Action> {
  /// The entity whose actions this event is about
  entity: Entity,
  /// When, by the Time clock, this event occured.
  ///
  /// This information is derived from timing information carried by the input 
  /// events that created this event, see I2.
  timing: Instant,
  _phantom: PhantomData<A>,
}
```

Users can watch for these events using either entity-specific observers or global observers.
Global observers are the simpler, more robust choice, and should be recommended, especially for games with more than one acting entity.

### D5: How do we control when actions are active?

**Status:** Proposed

**Problem:** As games grow in scope, they often need a *lot* of actions.
These can typically be meaningfully organized into logically related groups,
making them easier to work with and reason about.

Enabling and disabling actions is inherently tied up in this:
being able to toggle whole *categories* of actions contextually is a core job of an input manager in many complex applications.
For example, you might want to swap to "car controls" in a FPS game, or a set of mesh-editing hot keys in an editor,
or fully disabling all player actions in a multiplayer game while the menu screen is up, as the game cannot simply be paused.

**Prior art:** LWIM groups actions by type, with each set of action getting its own enum, input map, copy of the plugin and so on.
This is simple and would allow heterogenous action definition (e.g. libraries), but leads to difficulties, as clashing inputs from across definitions
cannot be detected.

Enabling and disabling actions within LWIM can be done in any of *four* different ways, from coarse to fine:
 
1. Disable all action types by adding a run condition on `InputManagerSystem::Update`, which skips updating every `ActionState` in the app.
2. Disable a single action type by adding a run condition on `TickActionStateSystem::<A>`. This prevents actions of type `A` from getting any new updates to their state.
3. Disable a single instance of `ActionState<A>`, by calling `ActionState::disable()`.
4. Disable a single action within a single instance, by calling `ActionState::disable_action(&action)`.

Similar levels of complexity and nuance are found in BEI and BAM.
The idea of a "context" lie at the core of both crates, defining a set of related actions that can be toggled on or off as a set.

In both cases, every action is defined as belonging to exactly one context, and contexts cannot be nested.
Each action in a context can be enabled/disabled as a group,
and individual actions can be toggled seperately as well.

Neither offer any additional helpers to make enabling and disabling actions and contexts based on game state easier,
although BAM has extremely clear semantics for what exactly disabling *does*.

Both crates also have a notion of "context priority", which controls which actions take effect *first*,
with inputs "consumed" by the actions that used them, making them appear unavailable to other lower-priority.
See D6 for more discussion on this point.

**Solution:** The primary requirements here are simple:

1. Actions can be enabled and disabled.
2. Every action belongs to at most one context.
3. Every action in a context can be enabled or disabled easily.

To make this work, we need to record how actions correspond to contexts, 
Tracking this on `InputMap` is the most natural: simply maps `ActionId` to a `ContextId`.
Actions belong to at most one context, and missing values represent actions that do not belong to a specific context,
allowing users to opt out of this concept completely if desired (e.g. for learning).

The other thing that we must track is the disabled state of each action and context.
That's simple enough, but belongs to each action *instance*, not the action as a whole.
As a result, it belongs on the `ActionState` component for each entity (see D4 and WM2).
Tracking both disabled *actions* seperately from disabled *contexts* is useful,
as it allows us to ensure that actions disabled for reasons *other* than their context do not become re-activated when their context activates.
Consider a passive that disables one of your conditional moves, for example.

Note that local multiplayer adds another wrinkle: the disabling maps must be also be keyed by owner, see WM2.

Contexts use a runtime `ContextId` identifier, with a stable path and a custom type,
following the exact same pattern as actions (WA1) and players (WM2).

The exact semantics of "what does it mean for an action to be disabled" are inspired by BAM:

- When an action is re-enabled, the transition is determined by the `require_reset` property set for that action, in combination with the currently active inputs
  - if `require_reset` is true, the value must return to its "neutral" state before the action can be re-activated
  - this prevents actions from having multiple effects when transitioning across game states, most commonly encountered in a game where you press `Escape` to pause, and then can press `Escape` again to close the menu
  - `require_reset` should default to on for button-like actions, and off for all other action shapes

### D6: How do we ensure that a pressed input has only the desired effect?

**Status:** Proposed

**Problem:** Suppose we're building an RTS. Left click is bound to "tell unit to move", and the player clicks on a GUI element. The unit should not move! A similar problem arises for hotkeys, when elements are focused or overlays are active.

**Prior art:** The traditional solution to this problem is "input consumption". Once an action takes effect, we mark its inputs as "consumed" for the frame, and can no longer trigger. In our example, the GUI click "consumes" the left mouse button, and so the unit movement system never sees the click at all.

This can also be used, somewhat implicitly, to handle clashing inputs in the case of chords (M1), by e.g. consuming the `S` from `Ctrl+S` to prevent "walk backwards" from happening when state is saved.

However, input consumption has a number of serious problems:

- the priority of actions must be defined relative to another
- input consumption is not always desired, and needs to be able to be turned on or off on a per-action basis
  - BEI swapped from [on-by-default to off-by-default](https://github.com/simgine/bevy_enhanced_input/pull/283)
- this is particularly confusing and frustrating if you allow arbitrary rebinding
- the mapping algorithm is no longer a single pass, and we instead walk back to consider other actions "as if" a given input was never pressed
- the "state" of actions is no longer a pure function of the inputs pressed for a step, and changes as actions are evaluated
  - the game logic layer "reaches up", modifying the input mapping layer
  - depending on the implementation, this may or may not be observable, introducing timing headaches

**Solution:** The cardinal sin of input consumption is that it bundles three central concerns of any input management design:

1. **Routing:** who is this input for?
2. **Arbitration:** for a single receiver (e.g. the GUI, the game world), which action should take effect?
3. **Exclusivity:** "no one else is allowed to use this input for a bit".

Routing should be handled by `bevy_picking` integration for pointer events, and `bevy_input_focus` integration for keyboard and gamepad events.
To make this work, inputs should *always* be treated as if they target a specific entity,
and inputs must bubble up *beyond* the existing window root into an abstract `InputRoot` entity.
When non-picking inputs are sent, they target the `InputRoot` entity directly if `InputFocus` is cleared or does not exist.
Library actions (e.g. `Open Inspector`) should bind their actions to this entity.

When we get an input event, we first check whether our active entity can respond to actions, by checking for an `ActionState` component.
If it can, we check to see if input matches any of its actions by checking the `InputMap` resource.
If an input event does not match any action, they are bubbled up, using Bevy's existing event-bubbling paradigm.
Inputs that match disabled actions (directly or by context) are not bubbled up, to avoid surprising behavior as actions are enabled/disabled.

This pattern maps closely to DOM event routing, and `bevy_picking`'s existing solution.

In order to determine which actions take effect for a given set of inputs,
we must solve the arbitration ("clashing inputs") problem: what happens if both `S` and `Ctrl+S` are bound?

The results of this arbitration can be cached and computed incrementally,
whenever the `InputMap` changes, although the `Owner` of each action should be considered (see WM2).
Centralizing this gives games the ability to simply reject pathological input bindings (e.g. "F is already bound to Pay Respects")
through the use of `Result`-ful input binding APIs (WM5).

Arbitration should default to a simple, deterministic set of rules, with "prioritize the longest chord" at its core,
but we should allow actions to define their own optional priority values (with conventions for standard values),
to offer an escape hatch for frustrating bugs or protect against pathological rebinding schemes.
This is much clearer and more robust than relying on action registration order.

More complex derived actions can use something akin to a gesture arena (see Flutter's design), attempting to match the "closest" pattern.

Exclusivity is the most niche use of input consumption, and comes from explicit capture mechanisms that capture a device or input for a period. This already exists in ad-hoc form as Bevy's [`PointerCaptureMap`](https://docs.rs/bevy/latest/bevy/prelude/struct.PointerCaptureMap.html).