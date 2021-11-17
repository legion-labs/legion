set LEGION_ROOT_DIR=%~dp0..\..
set DATA_RUNTIME_DIR=%LEGION_ROOT_DIR%\target\data-analytics-srv
rmdir /Q /S %DATA_RUNTIME_DIR%
mkdir %DATA_RUNTIME_DIR%
copy /Y %~dp0test\data\telemetry.db3 %DATA_RUNTIME_DIR%
set LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY=%DATA_RUNTIME_DIR%
cargo run --release
