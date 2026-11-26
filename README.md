Input mapping is a vitally important part of a game engine, but deceptively hard.

To facilitate clear thinking and thoughtful discussion, I've opted to write a *detailed* design doc, laying out both the problems in the space and what I think the solutions should be, with extensive focus on prior art.

There are three main sources of prior art, each a well-made and beloved input management library for Bevy:

1. [`leafwing_input_manager`](https://github.com/Leafwing-Studios/leafwing-input-manager) (LWIM): straightforward and battle-tested, but with overcomplicated serialization and no observer support
2. [`bevy_enhanced_input`](https://github.com/simgine/bevy_enhanced_input) (BEI): widely used with loads of game-oriented features, but serialization is left to the end user and the learning curve is steep
3. [`bevy_action_map`](https://docs.rs/bevy_action_map/latest/bevy_action_map/) (BAM): efficient, with careful attention to edge cases, but complex and heavily LLM-assisted

The hope is that, with the benefit of hindsight, we can take the best elements of each library, and incrementally build and ship an input management library that meets Bevy's needs: straightforward to get started with, powerful, and correct.

This repo is divided into multiple files to make digesting (and editing) the design work less painful:

- [Summary](summary.md): a high-level overview of the goals and key design decisions.
- [Layered Architecture](layered-architecture.md): a normative description of how input data should flow through Bevy applications
- [Core Decisions](core-decisions.md): a detailed breakdown of the core challenges in building an input management library, especially for Bevy
- [Data Structures](data-structures.rs): an at-a-glance collection of the intended final form of the most important data structures, as they would appear in our new library
- [Complications](complications.md): assorted *additional* frustrations that should be considered during design and implementation
- [Work Plan](work-plan.md): a loose plan for how we can turn this design into library code, in reviewable chunks that get a working (if basic) input manager into user hands ASAP

## Helping out

This is a collaborative design doc!
Because this is very complex and large-scope,
I've opted for a git repo, rather than the traditional HackMD,
which should make both collaborating and detailed discussion easier.

Don't be intimidated though!
I'd like your fixes, complaints, thoughts, etc.
Just submit a PR so we can debate designs and .

Broadly I would like to avoid fine-grained planning (just start writing code),
but critical complications or fundamental disagreements should be resolved before we begin, even if we're simply agreeing to disagree.
