/**
 * @fileoverview gRPC-Web generated client stub for analytics
 * @enhanceable
 * @public
 */
/* eslint-disable */
// @ts-nocheck

// GENERATED CODE -- DO NOT EDIT!





const grpc = {};
grpc.web = require('grpc-web');


var process_pb = require('./process_pb.js')

var stream_pb = require('./stream_pb.js')

var block_pb = require('./block_pb.js')
const proto = {};
proto.analytics = require('./analytics_pb.js');

/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?grpc.web.ClientOptions} options
 * @constructor
 * @struct
 * @final
 */
proto.analytics.PerformanceAnalyticsClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options.format = 'text';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname;

};


/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?grpc.web.ClientOptions} options
 * @constructor
 * @struct
 * @final
 */
proto.analytics.PerformanceAnalyticsPromiseClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options.format = 'text';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname;

};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.analytics.RecentProcessesRequest,
 *   !proto.analytics.ProcessListReply>}
 */
const methodDescriptor_PerformanceAnalytics_list_recent_processes = new grpc.web.MethodDescriptor(
  '/analytics.PerformanceAnalytics/list_recent_processes',
  grpc.web.MethodType.UNARY,
  proto.analytics.RecentProcessesRequest,
  proto.analytics.ProcessListReply,
  /**
   * @param {!proto.analytics.RecentProcessesRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.analytics.ProcessListReply.deserializeBinary
);


/**
 * @param {!proto.analytics.RecentProcessesRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.RpcError, ?proto.analytics.ProcessListReply)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.analytics.ProcessListReply>|undefined}
 *     The XHR Node Readable Stream
 */
proto.analytics.PerformanceAnalyticsClient.prototype.list_recent_processes =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/analytics.PerformanceAnalytics/list_recent_processes',
      request,
      metadata || {},
      methodDescriptor_PerformanceAnalytics_list_recent_processes,
      callback);
};


/**
 * @param {!proto.analytics.RecentProcessesRequest} request The
 *     request proto
 * @param {?Object<string, string>=} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.analytics.ProcessListReply>}
 *     Promise that resolves to the response
 */
proto.analytics.PerformanceAnalyticsPromiseClient.prototype.list_recent_processes =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/analytics.PerformanceAnalytics/list_recent_processes',
      request,
      metadata || {},
      methodDescriptor_PerformanceAnalytics_list_recent_processes);
};


module.exports = proto.analytics;

