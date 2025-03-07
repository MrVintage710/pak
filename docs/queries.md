Module for all of the query types.

# Query Basics

Queries are used to get the data you need from a pak file. A query takes a key, a value, and an operator. Here are the supported operators:

## Equal To Operator

```rust
use pak::query::PakQuery;

let query = "key".equals("value");
let query = PakQuery::equals("key", "value");
```

## Less Than Operator

```rust
use pak::query::PakQuery;

let query = "key".less_than("value");
let query = PakQuery::less_than("key", "value");
```

## Less Than or Equal To Operator

```rust
use pak::query::PakQuery;

let query = "key".less_than_or_equal("value");
let query = PakQuery::less_than_or_equal("key", "value");
```

## Greater Than Operator

```rust
use pak::query::PakQuery;

let query = "key".greater_than("value");
let query = PakQuery::greater_than("key", "value");
```

## Less Than or Equal To Operator

```rust
use pak::query::PakQuery;

let query = "key".greater_than_or_equal("value");
let query = PakQuery::greater_than_or_equal("key", "value");
```

**Note:** In all of the operators above, the comparisons are done using the [Ord](std::cmp::Ord) trait.

# Query Expressions

A Query expression is one or more queries combined together using logical operators. These can be used to make more complex queries. Here are the supported logical operators:

## Union Operator

This will take the results of 2 querys and combine them. Here is an example:

```rust
//Unions are done by using a binary or operation on 2 queries.
let query = "first_name".equals("John") | "age".less_than(35);
```

In this case, the query will return all records where the first name is John or the age is less than 35.

## Intersection Operator

This will take the results of 2 querys and select only the records that match both queries. Here is an example:

```rust
//Intersections are done by using a binary and operation on 2 queries.
let query1 = "first_name".equals("John") & "last_name".less_than("Doe");
```

This query will return all records where the first name is John and the last name is less than Doe.

## Chaining Queries

These operations can be chained and grouped with parentheses to create more complex queries. Here is an example:

```rust
let query = ("first_name".equals("John") | "age".less_than(35)) & "last_name".greater_than("Smith");
```

This query will get all records where either the first name is John or the age is less than 35, and the last name is greater than Smith. (alphabetical order)

Since this crate is in early development, not all queries have been implemented. I plan on implementing queries like between operations, like operations and query differences.