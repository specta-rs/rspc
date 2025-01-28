//! This file was generated by [rspc](https://github.com/specta-rs/rspc). Do not edit this file manually.

pub struct Procedures;

pub struct echo;

impl rspc_client::Procedure for echo {
    type Input = String;
    type Output = String;
    type Error = ();
    type Procedures = Procedures;
    const KIND: rspc_client::ProcedureKind = rspc_client::ProcedureKind::Query;
    const KEY: &'static str = "echo";
}

pub struct error;

impl rspc_client::Procedure for error {
    type Input = ();
    type Output = String;
    type Error = ();
    type Procedures = Procedures;
    const KIND: rspc_client::ProcedureKind = rspc_client::ProcedureKind::Query;
    const KEY: &'static str = "error";
}


pub mod hello {
	pub use super::Procedures;
pub struct hello;

impl rspc_client::Procedure for hello {
    type Input = ();
    type Output = String;
    type Error = ();
    type Procedures = Procedures;
    const KIND: rspc_client::ProcedureKind = rspc_client::ProcedureKind::Query;
    const KEY: &'static str = "hello";
}

}
pub struct pings;

impl rspc_client::Procedure for pings {
    type Input = ();
    type Output = String;
    type Error = ();
    type Procedures = Procedures;
    const KIND: rspc_client::ProcedureKind = rspc_client::ProcedureKind::Subscription;
    const KEY: &'static str = "pings";
}

pub struct sendMsg;

impl rspc_client::Procedure for sendMsg {
    type Input = String;
    type Output = String;
    type Error = ();
    type Procedures = Procedures;
    const KIND: rspc_client::ProcedureKind = rspc_client::ProcedureKind::Mutation;
    const KEY: &'static str = "sendMsg";
}

pub struct transformMe;

impl rspc_client::Procedure for transformMe {
    type Input = ();
    type Output = String;
    type Error = ();
    type Procedures = Procedures;
    const KIND: rspc_client::ProcedureKind = rspc_client::ProcedureKind::Query;
    const KEY: &'static str = "transformMe";
}

pub struct version;

impl rspc_client::Procedure for version {
    type Input = ();
    type Output = String;
    type Error = ();
    type Procedures = Procedures;
    const KIND: rspc_client::ProcedureKind = rspc_client::ProcedureKind::Query;
    const KEY: &'static str = "version";
}

