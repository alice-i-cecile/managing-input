//! Data structures taken from across the assorted design documents,
//! sketched out in Rust pseudocode and collected here for quick perusal.
//!
//! Only the rough final form is shown, while the intermediate forms are discussed
//! in the design documents to aid understanding and work planning.

mod input {
    enum Control {
        Button(ButtonControl),
        Throttle(ThrottleControl),
        Axis(AxisControl),
        DualAxis(DualAxisControl),
        Delta(DeltaControl),
    }

    enum ButtonControl {
        PhysicalKey(KeyCode),
        Gamepad(GamepadButton),
        Mouse(MouseButton),
        Modifier(ModifierKey),
        FromDualAxis {
            control: DualAxisControl,
            wedge_start: Rot2,
            wedge_end: Rot2,
        },
        FromThrottle {
            control: ThrottleControl,
            threshold: Option<f32>,
            hysteresis: Option<f32>,
        },
    }

    enum ModifierKey {
        Ctrl,
        Alt,
        Option,
        Shift,
        Os,
    }

    #[derive(Message)]
    struct InputMessage {
        control: Control,
        value: InputValue,
        device: Entity,
        timestamp: InputTimestamp,
    }

    enum InputValue {
        Button(bool),
        Throttle(f32),
        Axis(f32),
        DualAxis(Vec2),
        Delta(Vec2),
    }

    struct Binding<C> {
        primary: C,
        modifiers: SmallVec<[ButtonControl; 2]>,
    }
}

mod mapping {
    struct InputMap {
        action_bindings: BTreeMap<(Owner, ActionId), Vec<Option<BindingRow>>>,
        app_version: Version,
        schema_version: Version,
        preset: Option<String>,
    }

    #[derive(Reflect, Serialize, Deserialize)]
    struct BindingRow {
        binding: InputBinding,
        binding_origin: BindingOrigin,
    }

    #[derive(Default)]
    enum BindingOrigin {
        #[default]
        Inherited,
        User,
    }

    enum Owner {
        Shared,
        Player(PlayerId),
    }

    enum ActionScope {
        Shared,
        PerPlayer,
    }

    enum Version {
        Release {
            major: u32,
            minor: Option<u32>,
            patch: Option<u32>,
        },
        Identifier(String),
    }

    #[derive(Reflect, Serialize, Deserialize)]
    struct BindingRecord {
        binding: InputBinding,
        action: ActionPath,
        owner: Owner,
        slot: u16,
        origin: BindingOrigin,
    }

    #[derive(Asset, Reflect, Serialize, Deserialize)]
    struct BindingPreset {
        path: PresetPath,
        rows: Vec<InputBinding>,
    }
}

mod action_state {
    #[derive(Component)]
    struct ActionState {
        status_map: BTreeMap<ActionId, ActionStatus>,
    }

    struct ActionStatus {
        life_cycle: ActionLifecycle,
        value: Option<InputValue>,
    }

    enum ActionLifecycle {
        Disabled,
        Inactive,
        Tentative,
        Active,
    }

    #[derive(Event, EntityEvent)]
    struct Fired<A: Action> {
        entity: Entity,
        timing: Instant,
        _phantom: PhantomData<A>,
    }
}
