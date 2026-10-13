//! A cell that records an application in its chronicle, as a dated fact
//! (chronolexography, RFC-022 and RFC-046).
//!
//! The law contains no recording vocabulary. It says that an article
//! establishes an application (`produces.submission`, RFC-045) and which
//! hooks apply to it. The cell derives the record from that ([`shape`]): the
//! fields are the parameters of the establishing article and of those hooks,
//! the moment that counts is the one the law names (`produces.moment`) and
//! has its own key in the gram. Recording executes the article, so the hooks
//! run, and appends one gram to the chronicle ([`chronicle`]).
//!
//! Decisions, lexostatuses and objections are later slices.

pub mod cell;
pub mod chronicle;
pub mod error;
pub mod shape;

pub use cell::Cell;
pub use chronicle::{Chronicle, Gram, GramMoment};
pub use error::{Error, Result};
pub use shape::{Field, Moment, Shape};
