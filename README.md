# verse

**Programmable spatial storytelling.**

<details><summary>My Notes</summary>

> So we have a **^ scene ^**. Scene is composed of a **^ set ^** or a **^
> location ^**. The scene contains actors and a camera. The sets, locations and
> actors are available as github repos. Each set or location can contain one or
> more objects that can be moved around or hidden etc. We also have separate a
> concept of prop, which are also maintained in separate github repos, and they
> can be added to a scene too. Actors are the most interesting, they have one or
> more variants, so like an actor designed at high res can not be placed in a
> scene with other actors only in low res, so same actor should be designed in
> both high and low res. Actors can have even more variants, say actor in
> younger age vs older age, or actors that change forms. Actors have voices, and
> way of speaking, emotion, say things in anger or dejection etc. And similarly
> actors can have poses for standing and sitting. And actors can have body
> movements like how they walk, how they stand up or sit down etc. Actors have
> emotional variations in face, and body movements.

> the "gameplay", you can walk around in a scene, location, and in the location
> we have one or more scenes playing. So say when you walk into a street you
> see various gatherings of people, performing different scenes, and you can
> walk up to them. Each scene can have different timespans so scenes finish and
> then repeat, so when you walk up to a scene it can be anywhere as the scene
> started playing the moment you walked into the location. As soon as you get
> close to a scene, the camera movement is now decided by the scene director/
> creator, so you lose motion ability, rather, you get dragged along with
> camera. Seeing things the way director / creator wanted you to see. You can
> press escape to exit the camera follow along mode and are placed back in the
> location, and you can move on to any other scene in that location.

> We have a concept of portals. In any location, one can place one or more
> portals. The portals show a static image of the other side, look like a door.
> You can walk up to them and walk through them after opening them, and arrive
> at another location (or a different portal on the same location).

> The scene, location, set, actors, etc can be used as file formats and an
> engine
> for playing the scene, for creating movies / videos. The multiple scene
> happening in the same set is for creating a rich universe of knowledge where
> the community / curator is creating rich knowledge verse, where people can
> go and learn by following the scenes.

</details>

Verse is a Git-backed engine and file format for building explorable narrative
worlds.

A Verse is composed of **locations**.  
Locations contain **scenes**, **actors**, **sets**, **props**, and
**portals**.    
Users walk through locations. Scenes play around them.  
Approach a scene → camera control transfers to the director.  
Press escape → regain free movement.

-----

## Core Concepts

### 1. Verse

Top-level container.

```txt
Verse
└── Location*
```

A Verse can represent:

- A cinematic universe
- Knowledge world
- An explorable book
- A playable film

-----------

### 2. Location

Walkable spatial container.

Properties:

- Render profile (LOW / MEDIUM / HIGH)
- Set reference
- Scenes
- Portals
- Ambient objects

```txt
Location
├── set
├── scene*
├── portal*
└── object*
```

Scenes begin playing when the user enters the location.

------

### 3. Scene

Directed time capsule inside a location.

Scene orchestrates:

- Actors
- Props
- Camera
- Dialogue
- Emotion curves
- Duration

Scene modes:

- Ambient Mode → user free movement
- Directed Mode → camera authority transfers to scene

```txt
Scene
├── actors*
├── props*
├── camera
├── timeline
└── loop_mode
```

Time model:

```py
current_time = now - location_entry_time
```

Scenes may:

- LOOP
- ONE_SHOT

-----

### 4. Set

Visual environment design.

Reusable across locations.

```txt
Set
├── geometry
├── lighting
├── atmosphere
└── default objects
```

Sets are Git repos.


------

### 5. Actors

Actors are composable identity units.

```txt
Actor
├── visual_variants*
├── voice_profiles*
├── pose_library*
├── motion_library*
├── emotion_map*
└── speech_style
```

Constraints:

- Actor must match Location.render_profile
- Multiple variants: age, form, LOD

Actors are maintained in independent Git repositories.

----

### 6. Props

Lightweight movable objects.

```txt
Prop
├── mesh
├── states
└── interactions
```

Props:

- Can attach to scenes
- Can attach to actors
- Can be hidden / moved

Props are independent Git repos.

-----

### 7. Camera Model

Two authority modes:

- Explorer Camera (user controlled)
- Scene Camera (director controlled)

Switch logic:

```py
if distance_to_scene < threshold:
    camera = scene.camera
else:
    camera = explorer.camera
```

Escape key restores explorer camera.

### 8. Portals

Spatial connectors between locations.

```txt
Portal
├── source_location
├── target_location
├── preview_image
└── trigger
```

Portals display static previews.
Activation moves player to target location.

-----

## Repository Model

Everything is modular and Git-backed:

```txt
location/
set/
scene/
actor/
prop/
portal/
```

Each unit is independently versioned.

Verse composes them at runtime.

## Use Cases

- Interactive films
- Knowledge worlds
- Historical simulations
- Philosophical dialogues in spatial form
- Educational walk-through universes

-----

## Design Principles

- Deterministic by default
- Modular composition
- Resolution-aware assets
- Camera authority separation
- Loopable narrative time
- Spatial knowledge over linear chapters

-----

## Roadmap

- Formal file format spec
- Runtime execution engine
- Streaming location loader
- Branching scene support
- LLM-driven actors (optional layer)
- Collaborative world curation tools

----

## License

BSD

----

Verse turns narrative into space.

People don’t read chapters.  
They walk through ideas.
