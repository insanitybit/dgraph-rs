// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait Dgraph {
    fn login(&self, o: ::grpc::RequestOptions, p: super::api::LoginRequest) -> ::grpc::SingleResponse<super::api::Response>;

    fn query(&self, o: ::grpc::RequestOptions, p: super::api::Request) -> ::grpc::SingleResponse<super::api::Response>;

    fn alter(&self, o: ::grpc::RequestOptions, p: super::api::Operation) -> ::grpc::SingleResponse<super::api::Payload>;

    fn commit_or_abort(&self, o: ::grpc::RequestOptions, p: super::api::TxnContext) -> ::grpc::SingleResponse<super::api::TxnContext>;

    fn check_version(&self, o: ::grpc::RequestOptions, p: super::api::Check) -> ::grpc::SingleResponse<super::api::Version>;
}

// client

pub struct DgraphClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_Login: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::LoginRequest, super::api::Response>>,
    method_Query: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::Request, super::api::Response>>,
    method_Alter: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::Operation, super::api::Payload>>,
    method_CommitOrAbort: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::TxnContext, super::api::TxnContext>>,
    method_CheckVersion: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::Check, super::api::Version>>,
}

impl ::grpc::ClientStub for DgraphClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        DgraphClient {
            grpc_client: grpc_client,
            method_Login: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/api.Dgraph/Login".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Query: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/api.Dgraph/Query".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Alter: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/api.Dgraph/Alter".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CommitOrAbort: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/api.Dgraph/CommitOrAbort".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CheckVersion: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/api.Dgraph/CheckVersion".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl Dgraph for DgraphClient {
    fn login(&self, o: ::grpc::RequestOptions, p: super::api::LoginRequest) -> ::grpc::SingleResponse<super::api::Response> {
        self.grpc_client.call_unary(o, p, self.method_Login.clone())
    }

    fn query(&self, o: ::grpc::RequestOptions, p: super::api::Request) -> ::grpc::SingleResponse<super::api::Response> {
        self.grpc_client.call_unary(o, p, self.method_Query.clone())
    }

    fn alter(&self, o: ::grpc::RequestOptions, p: super::api::Operation) -> ::grpc::SingleResponse<super::api::Payload> {
        self.grpc_client.call_unary(o, p, self.method_Alter.clone())
    }

    fn commit_or_abort(&self, o: ::grpc::RequestOptions, p: super::api::TxnContext) -> ::grpc::SingleResponse<super::api::TxnContext> {
        self.grpc_client.call_unary(o, p, self.method_CommitOrAbort.clone())
    }

    fn check_version(&self, o: ::grpc::RequestOptions, p: super::api::Check) -> ::grpc::SingleResponse<super::api::Version> {
        self.grpc_client.call_unary(o, p, self.method_CheckVersion.clone())
    }
}

// server

pub struct DgraphServer;


impl DgraphServer {
    pub fn new_service_def<H : Dgraph + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/api.Dgraph",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/api.Dgraph/Login".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.login(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/api.Dgraph/Query".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.query(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/api.Dgraph/Alter".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.alter(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/api.Dgraph/CommitOrAbort".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.commit_or_abort(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/api.Dgraph/CheckVersion".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.check_version(o, p))
                    },
                ),
            ],
        )
    }
}
