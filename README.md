# Aspen

A simple (in progress) WGPU-based game engine built in Rust and focused on customization especially in the realm of simulation. It relates entities and components as archetypes.

## Goals

- **Customizable**: Every task completed by the user should be customizable and all engine code (with exception of the core ECS system) should be replaceable.
- **Simple**: Tasks should be simple to complete and intuitive to learn or well-documented when unintuitive. 
- **Data-driven**: Data should be the focus of all tasks as dictated by the ECS paradigm.

## Features

The core features are based on the goals of the engine as follows:

### Customizable

- [x] **Modular**: Everything except for the base logic of the engine should be customizable and replaceable by the user. When possible, functions should accept traits rather than objects to allow for maximum flexibility. 
- [x] **System Types**: Multiple types of systems are easy to create and run at varying times (e.g. at fixed intervals or with the frame rate).
- [x] **Render Systems**: Render systems should be completely creatable by the user although one should be provided. It should be easy to customize and work around.

### Simple

- [ ] **Config**: Users should be able to set up simple programs with json-like files that minimize code where possible for redundant tasks and allow for future extension.
- [ ] **Documentation**: Documentation should be thorough albeit easy to read and understand.
- [ ] **Reduction**: Code should be reduced wherever possible to make repetitive tasks easier to complete. Macros and pre-defined structs that are interoperable with user-defined structs should be used to reduce boilerplate.

### Data-driven

- [ ] **Non-Direct Mutability**: It should be impossible to single out an entity and mutate it directly. Instead, all changes should be done with logic that operates on a list of matches to a query across the world.
- [ ] **Archetypes**: Entities should be organized in archetypes that automatically re-organize to increase performance.
- [ ] **Type System**: The type system should be utilized to its fullest extent to make the process safer and more robust.

## Getting Started

You really shouldn't be using this library at the moment as it is in a very limited beta state. Please check back later or read over the examples to look for anything interesting.

