//! Types in this module are mostly internal and automatically generated. You
//! shouldn't need to interact with these types during normal usage, other than
//! the methods on [`Table`](/diesel/query_source/trait.Table.html)
#[doc(hidden)]
pub mod joins;
mod peano_numbers;

use std::error::Error;

use backend::Backend;
use expression::{Expression, NonAggregate, SelectableExpression};
use query_builder::*;
use row::NamedRow;
use types::{FromSqlRow, HasSqlType};

pub use self::joins::JoinTo;
pub use self::peano_numbers::*;

/// Trait indicating that a record can be queried from the database.
///
/// Types which implement `Queryable` represent the result of a SQL query. This
/// does not necessarily mean they represent a single database table.
///
/// This trait can be derived automatically using `#[derive(Queryable)]`. This
/// trait can only be derived for structs, not enums.
///
/// Diesel represents the return type of a query as a tuple. The purpose of this
/// trait is to convert from a tuple of Rust values that have been deserialized
/// into your struct.
///
/// When this trait is derived, it will assume that the order of fields on your
/// struct match the order of the fields in the query. This means that field
/// order is significant if you are using `#[derive(Queryable)]`. Field name has
/// no affect.
///
/// # Examples
///
/// If we just want to map a query to our struct, we can use `derive`.
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// #[derive(Queryable, PartialEq, Debug)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// # fn main() {
/// #     run_test();
/// # }
/// #
/// # fn run_test() -> QueryResult<()> {
/// #     use schema::users::dsl::*;
/// #     let connection = establish_connection();
/// let first_user = users.first(&connection)?;
/// let expected = User { id: 1, name: "Sean".into() };
/// assert_eq!(expected, first_user);
/// #     Ok(())
/// # }
/// ```
///
/// If we want to do additional work during deserializaiton, we can implement
/// the trait ourselves.
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// use schema::users;
/// use diesel::query_source::Queryable;
///
/// # /*
/// type DB = diesel::sqlite::Sqlite;
/// # */
///
/// #[derive(PartialEq, Debug)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// impl Queryable<users::SqlType, DB> for User {
///     type Row = (i32, String);
///
///     fn build(row: Self::Row) -> Self {
///         User {
///             id: row.0,
///             name: row.1.to_lowercase(),
///         }
///     }
/// }
///
/// # fn main() {
/// #     run_test();
/// # }
/// #
/// # fn run_test() -> QueryResult<()> {
/// #     use schema::users::dsl::*;
/// #     let connection = establish_connection();
/// let first_user = users.first(&connection)?;
/// let expected = User { id: 1, name: "sean".into() };
/// assert_eq!(expected, first_user);
/// #     Ok(())
/// # }
pub trait Queryable<ST, DB>
where
    DB: Backend + HasSqlType<ST>,
{
    /// The Rust type you'd like to map from.
    ///
    /// This is typically a tuple of all of your struct's fields.
    type Row: FromSqlRow<ST, DB>;

    /// Construct an instance of this type
    fn build(row: Self::Row) -> Self;
}

// Reasons we can't write this:
//
// impl<T, ST, DB> Queryable<ST, DB> for T
// where
//     DB: Backend + HasSqlType<ST>,
//     T: FromSqlRow<ST, DB>,
// {
//     type Row = Self;
//
//     fn build(row: Self::Row) -> Self {
//         row
//     }
// }
//
// (this is mostly a reference for @sgrif so he has a better reference every
// time he thinks he has a breakthrough on this problem).
//
// See the comment under `FromSqlRow`. All of the same impls conflict there that
// would here. If we had `#[derive(FromSqlRow)]`, we would also have that
// implement `Queryable`. I know it doesn't look like the `Option` impl
// conflicts, but it definitely does -- It covers types which implement
// `Queryable` but not `FromSqlRow`, while this impl wouldn't.
//
// The same "we could remove one of these traits" applies. Really `FromSqlRow`
// is the only trait that *needs* to exist. At the end of the day, `FromSql` is
// meant to be "I only want to deal with deserializing a single field" case, and
// `Queryable` is meant to be easier to implement by hand than `FromSqlRow`
// would be.

/// Deserializes the result of a query constructed with [`sql_query`].
///
/// # Deriving
///
/// To derive this trait, Diesel needs to know the SQL type of each field. You
/// can do this by either annotating your struct with `#[table_name =
/// "some_table"]` (in which case the SQL type will be
/// `diesel::dsl::SqlTypeOf<table_name::column_name>`), or by annotating each
/// field with `#[sql_type = "SomeType"]`.
///
/// If you are using `#[table_name]`, the module for that table must be in
/// scope. For example, to derive this for a struct called `User`, you will
/// likely need a line such as `use schema::users;`
///
/// If the name of a field on your struct is different than the column in your
/// `table!` declaration, or if you are deriving this trait on a tuple struct,
/// you can annotate the field with `#[column_name = "some_column"]`. For tuple
/// structs, all fields must have this annotation.
///
/// If a field is another struct which implements `QueryableByName`, instead of
/// a column, you can annotate that struct with `#[diesel(embed)]`
///
/// [`sql_query`]: ../fn.sql_query.html
pub trait QueryableByName<DB>
where
    Self: Sized,
    DB: Backend,
{
    /// Construct an instance of `Self` from the database row
    fn build<R: NamedRow<DB>>(row: &R) -> Result<Self, Box<Error + Send + Sync>>;
}

/// Represents a type which can appear in the `FROM` clause. Apps should not
/// need to concern themselves with this trait.
///
/// Types which implement this trait include:
/// - Tables generated by the `table!` macro
/// - Internal structs which represent joins
/// - A select statement which has had no query builder methods called on it,
///   other than those which can affect the from clause.
pub trait QuerySource {
    /// The type returned by `from_clause`
    type FromClause;
    /// The type returned by `default_selection`
    type DefaultSelection: SelectableExpression<Self>;

    /// The actual `FROM` clause of this type. This is typically only called in
    /// `QueryFragment` implementations.
    fn from_clause(&self) -> Self::FromClause;
    /// The default select clause of this type, which should be used if no
    /// select clause was explicitly specified. This should always be a tuple of
    /// all the desired columns, not `star`
    fn default_selection(&self) -> Self::DefaultSelection;
}

/// A column on a database table. Types which implement this trait should have
/// been generated by the [`table!` macro](../macro.table.html).
pub trait Column: Expression {
    /// The table which this column belongs to
    type Table: Table;

    /// The name of this column
    const NAME: &'static str;
}

/// A SQL database table. Types which implement this trait should have been
/// generated by the [`table!` macro](../macro.table.html).
pub trait Table: QuerySource + AsQuery + Sized {
    /// The type returned by `primary_key`
    type PrimaryKey: SelectableExpression<Self> + NonAggregate;
    /// The type returned by `all_columns`
    type AllColumns: SelectableExpression<Self> + NonAggregate;

    /// Returns the primary key of this table.
    ///
    /// If the table has a composite primary key, this will be a tuple.
    fn primary_key(&self) -> Self::PrimaryKey;
    /// Returns a tuple of all columns belonging to this table.
    fn all_columns() -> Self::AllColumns;
}

/// Determines how many times `Self` appears in `QS`
///
/// This trait is primarily used to determine whether or not a column is
/// selectable from a given from clause. A column can be selected if its table
/// appears in the from clause *exactly once*.
///
/// We do not allow the same table to appear in a query multiple times in any
/// context where referencing that table would be ambiguous (depending on the
/// context and backend being used, this may or may not be something that would
/// otherwise result in a runtime error).
pub trait AppearsInFromClause<QS> {
    /// How many times does `Self` appear in `QS`?
    type Count;
}
