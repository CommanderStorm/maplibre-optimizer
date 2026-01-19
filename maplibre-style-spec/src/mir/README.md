# WHY a mir?

The answer is simple:
The `v8.json` is not generally designed for either

- a compiler or
- codegen (the two things we are trying to do).

If one tries to construct from v8.json, one will quickly run into issues that it is

- not really valid and
- has no real self-consistent logic.

Example:
One needs to "know" that

- `layers` (which descirbes the general fields for) need code-gen-ing togeter with
- `layout` (which is the enum of possible options and type restrictions)