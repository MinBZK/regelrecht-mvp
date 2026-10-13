//! A cell that records an application in its chronicle, as a dated fact
//! (chronolexography, RFC-022 and RFC-046).
//!
//! The law contains no recording vocabulary. It says that an article
//! establishes an application (`produces.submission`, RFC-045) and which hooks
//! apply to it. The cell derives the record from that ([`shape`]): the fields
//! are the parameters of the establishing article and of those hooks, the
//! moment that counts is the one the law names (`produces.moment`) and has its
//! own key in the chronolexogram. Recording executes the article, so the hooks
//! run, and appends one chronolexogram to the chronicle ([`chronicle`]).
//!
//! The decision on an application is a second chronolexogram ([`decision`],
//! RFC-047). It refers to the application; where each value comes from
//! follows from the law and that reference. A policy that fills an open term
//! may read the chronicle back ([`reading`], RFC-048): the cell answers its
//! list without a source with the earlier decisions. Objections are a later
//! slice.

pub mod cell;
pub mod chronicle;
pub mod decision;
pub mod error;
pub mod reading;
pub mod shape;

pub use cell::Cell;
pub use chronicle::{Chronicle, Gram, GramMoment};
pub use decision::{ChronicleInput, DecisionShape, Input, Source};
pub use error::{Error, Result};
pub use shape::{Field, Moment, Shape};
