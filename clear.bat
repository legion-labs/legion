RMDIR "target/build_db" /S /Q
RMDIR "target/content-store" /S /Q
RMDIR "target/source-control" /S /Q
RMDIR "tests/sample-data/offline" /S /Q
RMDIR "tests/sample-data/runtime" /S /Q
RMDIR "tests/sample-data/temp" /S /Q
del "tests/sample-data/VERSION" /s
Remove-Item tests\sample-data\VERSION