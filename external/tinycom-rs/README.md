# tinycom-rs

Tiny COM implements the tiny subset necessary to consume IUnknown interfaces, it does so in a platform independent way and with 0 dependencies, so shared libraries complying with COM can be loaded on other systems (like the Direct X shader compiler).

## Alternatives

This crate covers a tiny area, other COM related crated might be more suitables depending on your use case

* Windows only com support: https://github.com/microsoft/com-rs
* Ability to write COM components in rust: https://github.com/Rantanen/intercom

## Credits

This crate was originally developped by Lee Jeffery, and has been depracated since to be replaced by com-rs, the official microsoft COM bindings. We decided to revive the crate to serve our small use case and the idea is to not extend it's functionnality.
