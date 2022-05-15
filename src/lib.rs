//! A crate for targeting and accessing _actual implementation_.
//!
//! Take an example trait:
//!
//! ```rust
//! # struct Website;
//! trait ScrapeTheInternet {
//!     fn scrape_the_internet(&self) -> Vec<Website>;
//! }
//! ```
//!
//! The trait represents some abstract computation. The trait exports a method signature that
//! can be implemented by types. In this case, we can imagine what a true implementation of the
//! trait will do: _Actually scrape the internet_.
//!
//! `implementation` provides the [Impl] type as an implementation target for traits having the following semantics:
//!
//! * The trait has only one actual, true implementation.
//! * Other implementations of the trait _may exist_, but these are interpreted as _fake_, _mocked_ in some way.
//!
//! `implementation` enables a standardized way of writing these actual implementations in a way
//! that allows the actual `Self`-receiver type to be unknown.
//!
//! # Usage
//! To define the actual, generic implementation of `ScrapeTheInternet`, we can write the following impl:
//!
//! ```rust
//! # struct Website;
//! # trait ScrapeTheInternet {
//! #    fn scrape_the_internet(&self) -> Vec<Website>;
//! # }
//! impl<T> ScrapeTheInternet for implementation::Impl<T> {
//!     fn scrape_the_internet(&self) -> Vec<Website> {
//!         todo!("find all the web pages, etc")
//!     }
//! }
//! ```
//!
//! This code implements the trait for [Impl], and by doing that we have asserted
//! that it is the actual, true implementation.
//!
//! The implementation is fully generic, and works for any `T`.
//!
//! ```no_run
//! # struct Website;
//! # trait ScrapeTheInternet {
//! #    fn scrape_the_internet(&self) -> Vec<Website>;
//! # }
//! # impl<T> ScrapeTheInternet for implementation::Impl<T> {
//! #     fn scrape_the_internet(&self) -> Vec<Website> {
//! #         todo!("find all the web pages, etc")
//! #     }
//! # }
//! use implementation::Impl;
//!
//! struct MyType;
//!
//! let websites = Impl::new(MyType).scrape_the_internet();
//! ```
//!
//! ## Trait bounds
//! The advantage of keeping trait implementations generic, is that the self type might
//! live in a downstream crate. Let's say we need to access a configuration parameter
//! from `scrape_the_internet`. E.g. the maximum number of pages to scrape:
//!
//! ```rust
//! use implementation::Impl;
//!
//! # struct Website;
//! # trait ScrapeTheInternet {
//! #    fn scrape_the_internet(&self) -> Vec<Website>;
//! # }
//! trait GetMaxNumberOfPages {
//!     fn get_max_number_of_pages(&self) -> Option<usize>;
//! }
//!
//! impl<T> ScrapeTheInternet for Impl<T>
//!     where Impl<T>: GetMaxNumberOfPages
//! {
//!     fn scrape_the_internet(&self) -> Vec<Website> {
//!         let max_number_of_pages = self.get_max_number_of_pages();
//!         todo!("find all the web pages, etc")
//!     }
//! }
//! ```
//!
//! Now, for this to work, `Impl<T>` also needs to implement `GetMaxNumberOfPages` (for the same `T` that is going to be used).
//!
//! `GetMaxNumberOfPages` would likely be implemented for a specific `T` rather than a generic one,
//! since that `T` would typically be some configuration holding that number:
//!
//! ```rust
//! # trait GetMaxNumberOfPages {
//! #     fn get_max_number_of_pages(&self) -> Option<usize>;
//! # }
//! struct Config {
//!     max_number_of_pages: Option<usize>
//! }
//!
//! impl GetMaxNumberOfPages for implementation::Impl<Config> {
//!     fn get_max_number_of_pages(&self) -> Option<usize> {
//!         self.max_number_of_pages
//!     }
//! }
//! ```
//!
//! # Explanation
//!
//! This crate is the solution to a trait coherence problem.
//!
//! Given the trait above, we would like to provide an actual and a mocked implementation.
//! We might know what its actual implementation looks like as an algorithm, but
//! _not what type it should be implemented for_. There could be several reasons
//! to have a generic Self:
//!
//! * The `Self` type might live in a downstream crate
//! * It is actually designed to work generically
//!
//! If we had used a generic Self type (`impl<T> DoSomething for T`), the trait
//! would be unable to also have distinct fake implementations, because that would break
//! the coherence rules: A generic ("blanket") impl and a specialized
//! impl are not allowed to exist at the same time, because that would lead to ambiguity.
//!
//! To solve that, a concrete type is needed as implementation target. But that
//! type is allowed to be generic _internally_. It's just the root level that
//! needs to be a concretely named type.
//!
//! That type is the [Impl] type.
//!
//! When we use this implementation, we can create as many fake implementations as we want.
//!

/// Wrapper type for targeting and accessing actual implementation.
///
/// [Impl] has smart-pointer capabilities, as it implements [std::ops::Deref] and [std::ops::DerefMut].
/// You may freely choose what kind of `T` you want to wrap. It may be an owned one or it could be
/// a `&T`. Each have different tradeoffs.
///
/// An owned `T` is the most flexible in implementations, but that requires always owning "sub-implementations"
/// through an `Impl`:
///
/// ```rust
/// use implementation::Impl;
///
/// struct MyConfig {
///     param1: i32,
///     sub_config: Impl<SubConfig>,
/// }
///
/// struct SubConfig {
///     param2: i32,
/// }
/// ```
///
/// A referenced `&T` makes it possible to _borrow_ an `Impl` from any `T`, but that _could_ prove to be
/// more troublesome in some implementations. This also will require a reference-within-reference
/// design in trait methods with a `&self` receiver, and some more boilerplate if it needs to be cloned:
///
/// ```
/// use implementation::Impl;
///
/// trait DoSomething {
///     fn something(&self);
/// }
///
/// impl<'t, T> DoSomething for Impl<&'t T>
///     where T: Clone + Send + 'static
/// {
///     // self is an `&Impl<&T>`:
///     fn something(&self) {
///
///         // it will require some more code to make a proper clone of T:
///         let t_clone = self.into_inner().clone();
///
///         let handle = std::thread::spawn(move || {
///             let implementation = Impl::new(&t_clone);
///
///             // Do something else with Impl<&T>
///         });
///
///         handle.join().unwrap();
///     }
/// }
/// ```
#[derive(Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Impl<T>(T);

impl<T> Impl<T> {
    /// Construct a new [Impl].
    pub fn new(value: T) -> Impl<T> {
        Impl(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Impl<T> {
    fn from(value: T) -> Impl<T> {
        Impl(value)
    }
}

impl<T> std::ops::Deref for Impl<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Impl<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<T> for Impl<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
