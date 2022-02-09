# Legion Source Control

This crate implements the client and server counterparts of the Legion-Labs
Source Control (LSC).

LSC is designed to be used by game makers, who deal on daily basis with code and
data assets, that can get can quite large. Trying to unify the strengths of both
Helix Perforce and Git, it is a VCS with locking abilities that stores its data
in a hierachical tree of commits and branches.

Backed by the Legion Labs immutable blob storage technology, LSC is extremely
cache friendly and become more and more effective as the total number of
collaborators increases.

## Developer notes

If you ever change the database layout, you will break things.

As the source control is still under heavy development, this is expected at the
time of writing these lines. In order to regenerate the test database, you can
use the following command:

```bash
cargo mbuild && cargo mrun --bin sample-data-compiler -- --clean
```

And then add the changes files inside the `tests` directory.