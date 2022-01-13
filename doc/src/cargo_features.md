# Cargo features

## Default features

| name            | description                                                                         |
| --------------- | ----------------------------------------------------------------------------------- |
| custom-protocol | Generates front-end source modules                                                  |
| standalone      | Runtime-server supports rendering in a local window, as an alternative to streaming |

## Optional features

| name                    | description                                                 |
| ----------------------- | ----------------------------------------------------------- |
| lgn_ci_testing          | Continuous integration testing                              |
| max_level_off           | In development builds, will disable tracing                 |
| max_level_error         | In development builds, will trace only errors               |
| max_level_warn          | In development builds, will trace up to warning entries     |
| max_level_info          | In development builds, will trace up to information entries |
| max_level_debug         | In development builds, will trace up to debug entries       |
| max_level_trace         | In development builds, will trace everything                |
| max_lod_off             | In development builds, disable tracing                      |
| max_lod_min             | In development builds, minimum tracing verbosity            |
| max_lod_med             | In development builds, average tracing verbosity            |
| max_lod_max             | In development builds, maximum tracing verbosity            |
| offline                 | Enable support for offline/editor data (resources)          |
| release_max_level_off   | In release builds, will disable tracing                     |
| release_max_level_error | In release builds, will trace only errors                   |
| release_max_level_warn  | In release builds, will trace up to warning entries         |
| release_max_level_info  | In release builds, will trace up to information entries     |
| release_max_level_debug | In release builds, will trace up to debug entries           |
| release_max_level_trace | In release builds, will trace everything                    |
| release_max_lod_off     | In development builds, disable tracing                      |
| release_max_lod_min     | In release builds, minimum tracing verbosity                |
| release_max_lod_med     | In release builds, average tracing verbosity                |
| release_max_lod_max     | In release builds, maximum tracing verbosity                |
| run-codegen             |                                                             |
| run-codegen-validation  |                                                             |
| runtime                 | Enable support for runtime data (assets)                    |
| serialize               |                                                             |
| track-device-contexts   |                                                             |
| vulkan                  | Vulkan graphics backend                                     |
