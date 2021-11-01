@rem scoop install protoc-gen-grpc-web
protoc --js_out=import_style=commonjs,binary:../../client/analytics-web/src/proto analytics.proto block.proto stream.proto process.proto --grpc-web_out=import_style=commonjs,mode=grpcwebtext:../../client/analytics-web/src/proto
