# *SpacetimeDSL*

SpacetimeDSL provides you a high-level [*D*omain *S*pecific *L*anguage (DSL)](https://en.wikipedia.org/wiki/Domain-specific_language) to interact in an ergonomic and type-safe way with the data in your SpacetimeDB instance.

## Limitations

The following things aren't considered during code generation yet:

- [Multi Column Indices](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/2)
- [Using IndexScanRangeBounds on get_many_by_column and delete_many_by_column](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/21)

The following SpacetimeDB features can't be used and will result in compilation errors:

- [More than one `#[table]` attribute macro on the same struct](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/10)
- [More than one struct with `#[derive(SpacetimeDSL)` in the same rust module](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/11)
- [`[#index(direct)]`](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/20)

## Features

If features contain snippets of generated code, they relate to the following input code:

```rust
pub mod entity {
    use spacetimedsl::derive::SpacetimeDSL;

    /// A Entity is a unique machine-readable identifier - it contains no data other than that and has no behavior.
    #[derive(Clone, Debug, PartialEq, SpacetimeDSL)]
    #[spacetimedb::table(name = entity, public)]
    #[plural_table_name(entities)]
    pub struct Entity {
        /// The unique ID of the Entity.
        #[primary_key]
        #[auto_inc]
        #[wrap]
        id: u128,
    }
}

pub mod component {
    pub mod identifier {
        use spacetimedsl::derive::SpacetimeDSL;

        /// A Identifier is a developer-friendly String.
        #[derive(Clone, Debug, PartialEq, SpacetimeDSL)]
        #[spacetimedb::table(name = identifier, public)]
        #[plural_table_name(identifiers)]
        pub struct Identifier {
            /// The unique ID of the Identifier.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            /// The unique ID of the Entity the Identifier belongs to.
            #[unique]
            #[wrap(crate::entity::EntityId)]
            entity_id: u128,

            // The unique value of the Identifier.
            #[unique]
            pub value: String,
        }
    }

    pub mod position {
        use spacetimedsl::derive::SpacetimeDSL;

        /// A Position in the World.
        #[derive(Clone, Debug, PartialEq, SpacetimeDSL)]
        #[spacetimedb::table(name = position, public, index(name = x_y_z, btree(columns = [x, y, z])))]
        #[plural_table_name(positions)]
        pub struct Position {
            /// The unique ID of the Position.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            /// The unique ID of the Entity the Position belongs to.
            #[unique]
            #[wrap(crate::entity::EntityId)]
            entity_id: u128,

            pub x: i128,

            pub y: i128,

            pub z: i128,
        }
    }
}
```

Which produces the following methods:

![Generated methods](dsl.png)

If you want to have a direct look into Code which uses SpacetimeDSL, look into the [example/src/lib.rs](example/src/lib.rs) file.

### Wrapper Types

SpacetimeDSL allows developers to wrap their (primitive) table fields into [unique, auto-generated alias types](https://medium.com/unil-ci-software-engineering/clean-ddd-lessons-modeling-identity-ff8bc17e0ae6#:~:text=We%20should%20not%20use%20primitives) to decrease [primitive obsession](https://refactoring.guru/smells/primitive-obsession). This allows the following:

- [Reversing the order of fields (e.g. of multi-column indices) results in compilation-errors instead of runtime-errors when accessing them](https://medium.com/@gara.mohamed/domain-driven-design-the-identifier-type-pattern-d86fd3c128b3#:~:text=Suppose%20that%20we,at%20compile%20time.) and
- [Logical dependencies become physical ones](https://medium.com/@gara.mohamed/domain-driven-design-the-identifier-type-pattern-d86fd3c128b3#:~:text=Make%20logical%20dependencies%20physical) (this is huge!)

If you add `#[wrap]` to a field of your table-struct, SpacetimeDSL will generate such a Wrapper Type.

The name of it is generated through the name of the table-struct and the name of the field. If you want to override this default, just provide a name like so: `#[wrap(GameObjectId)]`.

If they're generated, the generated code of other features is influenced, which is described in each feature's section.

You can reference Wrapper Types at other table's fields by adding their path to the attribute. In the input code, both the `identifier` and the `position` table contain a `entity_id` field with `#[wrap(crate::entity::EntityId)]`. SpacetimeDSL will require developers to provide an object of type EntityId instead of a u128, so the code won't compile if they provide something else.
More of, they can just provide a reference to a `Entity` object instead of calling `get_id()` on the entity because there is a `impl From<&Entity> for EntityId`, which reduces much boilerplate code. SpacetimeDSL will make the navigation to access the raw value for the developer instead.

If you need to reference the same Wrapper Type twice in the same table, you'll need to provide the name of the already generated Wrapper type and prefix it with `self::`.
For example, if we would want Entities to have a parent, you would add `#[wrap(self::EntityId)] parent: u128,` to your table-struct. First of all the name is needed because it would create a new Wrapper Type called `EntityParent` instead if it's not named, second it needs to be prefixed with `self::` because SpacetimeDSL doesn't check whether it has already created a Wrapper Type with the same name, instead it looks whether the name is set and contains a `::` and if so it generates no new one Wrapper Type and instead assumes it's generated while parsing another table field.

The input code produces the following output (and many side effects in other features generated code, see the feature's respective section in the docs):

```rust
#[derive(Clone, Debug, PartialEq, spacetimedb::SpacetimeType)]
pub struct EntityId {
    value: u128,
}

impl From<&Entity> for EntityId {
    fn from(value: &Entity) -> Self {
        value.get_id()
    }
}

impl spacetimedsl::Wrapper<u128, EntityId> for EntityId {
    fn new(value: u128) -> Self {
        Self { value }
    }

    fn default() -> Self {
        Self { value: u128::default() }
    }
    
    fn value(&self) -> u128 {
        self.value.clone()
    }
}


#[derive(Clone, Debug, PartialEq, spacetimedb::SpacetimeType)]
pub struct IdentifierId {
    value: u128,
}

impl From<&Identifier> for IdentifierId {
    fn from(value: &Identifier) -> Self {
        value.get_id()
    }
}

impl spacetimedsl::Wrapper<u128, IdentifierId> for IdentifierId {
    fn new(value: u128) -> Self {
        Self { value }
    }
    fn default() -> Self {
        Self { value: u128::default() }
    }
    fn value(&self) -> u128 {
        self.value.clone()
    }
}

#[derive(Clone, Debug, PartialEq, spacetimedb::SpacetimeType)]
pub struct PositionId {
    value: u128,
}

impl From<&Position> for PositionId {
    fn from(value: &Position) -> Self {
        value.get_id()
    }
}

impl spacetimedsl::Wrapper<u128, PositionId> for PositionId {
    fn new(value: u128) -> Self {
        Self { value }
    }
    fn default() -> Self {
        Self { value: u128::default() }
    }
    fn value(&self) -> u128 {
        self.value.clone()
    }
}
```

### Accessors (Getters and Setters)

For any field in the table-struct, a public getter is generated. It returns either a reference to the table field value or if `#[wrap]` it clones the value and creates a new instance of the Wrapper Type.

For any field in the table-struct *which is not private*, a setter with the visibility of the field is generated.

If the field is `#[wrap]`, developers must provide a `Wrapper Type` or `Row` instead of an `Column`.

 As you can see, in the generated code aren't any setters for fields which are private. You can use the visibility of fields to describe that a field value should never change after creating instances of the table-struct, which is useful for e. g. primary- and foreign key fields, which should never change. If <https://github.com/rust-lang/rust/issues/105077> is released, the library will use the field mutability restrictions instead of the visibility to decide whether or not to generate setters.

```rust
impl Entity {
    pub fn get_id(&self) -> EntityId {
        EntityId::new(self.id.clone())
    }
}

impl Identifier {
    pub fn get_id(&self) -> IdentifierId {
        IdentifierId::new(self.id.clone())
    }
    pub fn get_entity_id(&self) -> crate::entity::EntityId {
        crate::entity::EntityId::new(self.entity_id.clone())
    }
    pub fn get_value(&self) -> &String {
        &self.value
    }
    pub fn set_value(&mut self, value: String) {
        self.value = value
    }
}

impl Position {
    pub fn get_id(&self) -> PositionId {
        PositionId::new(self.id.clone())
    }
    pub fn get_entity_id(&self) -> crate::entity::EntityId {
        crate::entity::EntityId::new(self.entity_id.clone())
    }
    pub fn get_x(&self) -> &i128 {
        &self.x
    }
    pub fn set_x(&mut self, x: i128) {
        self.x = x
    }
    pub fn get_y(&self) -> &i128 {
        &self.y
    }
    pub fn set_y(&mut self, y: i128) {
        self.y = y
    }
    pub fn get_z(&self) -> &i128 {
        &self.z
    }
    pub fn set_z(&mut self, z: i128) {
        self.z = z
    }
}
```

### DSL Extension Methods

SpacetimeDSL wraps a `&spacetimedb::ReducerContext` and provides a more ergonomic API for it, including added capabilities to reduce boilerplate code.
Each of the following headings are methods on `spacetimedsl::DSL`. Instances of the DSL can be created through the `spacetimedsl::dsl(ctx: &ReducerContext) -> DSL;` function.

Instead of passing the DSL and the ReducerContext to functions/methods which a reducer depend on, you should only pass the DSL and use the `ctx()` method on it, which is defined by the `DSLContext` trait.

In the next sections,

- `Row` means a Object Instance of a table's struct and

- `Column Value` means a Reference to a Object Instance of a table's field type.

#### Create Row

For any table-struct, a `create_row` DSL extension method is created, which performs a `row().try_insert(row)`.

For each field which

- is not `#[auto_inc]` or
- has the name
  - `created_at` or
  - `modified_at`,

the developer needs to provide an argument to the function.

For the others the default (`0` for `auto_inc`, `self.ctx().timestamp` for `created_at`/`modified_at`) is assumed, like in the example for the `Entity`-struct, which reduces boilerplate code.

For each `#[wrap]` field the developer needs to provide a `impl Into<WrapperType>` instead of a instance of the field type. So to create a Position, you can provide either an `&Entity` (which you'll do if you have it in your scope) or an `EntityId` (which you'll do if you have it as a reducer argument provided by clients) for the `entity_id` parameter.

```rust
pub trait CreateEntity: spacetimedsl::DSLContext {
    ///Create a Entity.
    fn create_entity(
        &self,
    ) -> Result<Entity, spacetimedb::TryInsertError<entity__TableHandle>> {
        let entity = Entity {
            id: u128::default(),
            created_at: self.ctx().timestamp,
        };
        return self.ctx().db().entity().try_insert(entity);
    }
}
impl CreateEntity for spacetimedsl::DSL<'_> {}

pub trait CreateIdentifier: spacetimedsl::DSLContext {
    ///Create a Identifier.
    fn create_identifier(
        &self,
        entity_id: impl Into<crate::entity::EntityId>,
        value: String,
    ) -> Result<
        Identifier,
        spacetimedb::TryInsertError<identifier__TableHandle>,
    > {
        let identifier = Identifier {
            id: u128::default(),
            entity_id: entity_id.into().value(),
            value,
            created_at: self.ctx().timestamp,
            modified_at: self.ctx().timestamp,
        };
        return self.ctx().db().identifier().try_insert(identifier);
    }
}
impl CreateIdentifier for spacetimedsl::DSL<'_> {}

pub trait CreatePosition: spacetimedsl::DSLContext {
    ///Create a Position.
    fn create_position(
        &self,
        entity_id: impl Into<crate::entity::EntityId>,
        x: i128,
        y: i128,
        z: i128,
    ) -> Result<Position, spacetimedb::TryInsertError<position__TableHandle>> {
        let position = Position {
            id: u128::default(),
            entity_id: entity_id.into().value(),
            x,
            y,
            z,
            created_at: self.ctx().timestamp,
            modified_at: self.ctx().timestamp,
        };
        return self.ctx().db().position().try_insert(position);
    }
}
impl CreatePosition for spacetimedsl::DSL<'_> {}
```

#### Get Row by Column

For any `#[primary_key]` and `#[unique]` field on a table-struct, a `get_row_by_column` DSL extension method is created, which performs a `row().column().find(column)`.
If the field is `#[wrap]`, developers must provide a `Wrapper Type` or `Row` instead of an `Column`.

```rust
pub trait GetEntityRowOptionById: spacetimedsl::DSLContext {
    ///Get a Option<Entity> by it's id.
    fn get_entity_by_id(&self, id: impl Into<EntityId>) -> Option<Entity> {
        return self.ctx().db().entity().id().find(id.into().value());
    }
}
impl GetEntityRowOptionById for spacetimedsl::DSL<'_> {}

pub trait GetIdentifierRowOptionById: spacetimedsl::DSLContext {
    ///Get a Option<Identifier> by it's id.
    fn get_identifier_by_id(
        &self,
        id: impl Into<IdentifierId>,
    ) -> Option<Identifier> {
        return self.ctx().db().identifier().id().find(id.into().value());
    }
}
impl GetIdentifierRowOptionById for spacetimedsl::DSL<'_> {}

pub trait GetIdentifierRowOptionByEntityId: spacetimedsl::DSLContext {
    ///Get a Option<Identifier> by it's entity_id.
    fn get_identifier_by_entity_id(
        &self,
        entity_id: impl Into<crate::entity::EntityId>,
    ) -> Option<Identifier> {
        return self
            .ctx()
            .db()
            .identifier()
            .entity_id()
            .find(entity_id.into().value());
    }
}
impl GetIdentifierRowOptionByEntityId for spacetimedsl::DSL<'_> {}

pub trait GetIdentifierRowOptionByValue: spacetimedsl::DSLContext {
    ///Get a Option<Identifier> by it's value.
    fn get_identifier_by_value(&self, value: String) -> Option<Identifier> {
        return self.ctx().db().identifier().value().find(value);
    }
}
impl GetIdentifierRowOptionByValue for spacetimedsl::DSL<'_> {}

pub trait GetPositionRowOptionById: spacetimedsl::DSLContext {
    ///Get a Option<Position> by it's id.
    fn get_position_by_id(&self, id: impl Into<PositionId>) -> Option<Position> {
        return self.ctx().db().position().id().find(id.into().value());
    }
}
impl GetPositionRowOptionById for spacetimedsl::DSL<'_> {}

pub trait GetPositionRowOptionByEntityId: spacetimedsl::DSLContext {
    ///Get a Option<Position> by it's entity_id.
    fn get_position_by_entity_id(
        &self,
        entity_id: impl Into<crate::entity::EntityId>,
    ) -> Option<Position> {
        return self
            .ctx()
            .db()
            .position()
            .entity_id()
            .find(entity_id.into().value());
    }
}
impl GetPositionRowOptionByEntityId for spacetimedsl::DSL<'_> {}
```

#### Get Rows by Column in

This is the same as the feature above but allows to get multiple rows instead of one.

For any `#[primary_key]` and `#[unique]` field on a table-struct, a `get_rows_by_column_in` DSL extension method is created, which performs a `row().column().find(column)` with multiple column values.
If the field is `#[wrap]`, developers must provide a `Vec<Wrapper Type>` or `Vec<Row>` instead of an `Vec<Column>`.

```rust
pub trait GetEntityRowOptionsById: spacetimedsl::DSLContext {
    ///Get all Option<Entity> by it's id's.
    fn get_entities_by_id_in<'a>(
        &'a self,
        ids: Vec<impl Into<EntityId>>,
    ) -> impl Iterator<Item = Option<Entity>> {
        let mut result: Vec<Option<Entity>> = ::alloc::vec::Vec::new();
        for id in ids {
            result.push(self.ctx().db().entity().id().find(id.into().value()));
        }
        result.into_iter()
    }
}
impl GetEntityRowOptionsById for spacetimedsl::DSL<'_> {}

pub trait GetIdentifierRowOptionsById: spacetimedsl::DSLContext {
    ///Get all Option<Identifier> by it's id's.
    fn get_identifiers_by_id_in<'a>(
        &'a self,
        ids: Vec<impl Into<IdentifierId>>,
    ) -> impl Iterator<Item = Option<Identifier>> {
        let mut result: Vec<Option<Identifier>> = ::alloc::vec::Vec::new();
        for id in ids {
            result
                .push(self.ctx().db().identifier().id().find(id.into().value()));
        }
        result.into_iter()
    }
}
impl GetIdentifierRowOptionsById for spacetimedsl::DSL<'_> {}

pub trait GetIdentifierRowOptionsByEntityId: spacetimedsl::DSLContext {
    ///Get all Option<Identifier> by it's entity_id's.
    fn get_identifiers_by_entity_id_in<'a>(
        &'a self,
        entity_ids: Vec<impl Into<crate::entity::EntityId>>,
    ) -> impl Iterator<Item = Option<Identifier>> {
        let mut result: Vec<Option<Identifier>> = ::alloc::vec::Vec::new();
        for entity_id in entity_ids {
            result
                .push(
                    self
                        .ctx()
                        .db()
                        .identifier()
                        .entity_id()
                        .find(entity_id.into().value()),
                );
        }
        result.into_iter()
    }
}
impl GetIdentifierRowOptionsByEntityId for spacetimedsl::DSL<'_> {}

pub trait GetIdentifierRowOptionsByValue: spacetimedsl::DSLContext {
    ///Get all Option<Identifier> by it's value's.
    fn get_identifiers_by_value_in<'a>(
        &'a self,
        values: Vec<Box<str>>,
    ) -> impl Iterator<Item = Option<Identifier>> {
        let mut result: Vec<Option<Identifier>> = ::alloc::vec::Vec::new();
        for value in values {
            result.push(self.ctx().db().identifier().value().find(value));
        }
        result.into_iter()
    }
}
impl GetIdentifierRowOptionsByValue for spacetimedsl::DSL<'_> {}

pub trait GetPositionRowOptionsById: spacetimedsl::DSLContext {
    ///Get all Option<Position> by it's id's.
    fn get_positions_by_id_in<'a>(
        &'a self,
        ids: Vec<impl Into<PositionId>>,
    ) -> impl Iterator<Item = Option<Position>> {
        let mut result: Vec<Option<Position>> = ::alloc::vec::Vec::new();
        for id in ids {
            result.push(self.ctx().db().position().id().find(id.into().value()));
        }
        result.into_iter()
    }
}
impl GetPositionRowOptionsById for spacetimedsl::DSL<'_> {}

pub trait GetPositionRowOptionsByEntityId: spacetimedsl::DSLContext {
    ///Get all Option<Position> by it's entity_id's.
    fn get_positions_by_entity_id_in<'a>(
        &'a self,
        entity_ids: Vec<impl Into<crate::entity::EntityId>>,
    ) -> impl Iterator<Item = Option<Position>> {
        let mut result: Vec<Option<Position>> = ::alloc::vec::Vec::new();
        for entity_id in entity_ids {
            result
                .push(
                    self
                        .ctx()
                        .db()
                        .position()
                        .entity_id()
                        .find(entity_id.into().value()),
                );
        }
        result.into_iter()
    }
}
impl GetPositionRowOptionsByEntityId for spacetimedsl::DSL<'_> {}
```

#### Get Rows By Column

For any `#[index]` (field) on a table-struct, a `get_rows_by_column` DSL extension method is created, which performs a `row().column().filter(column)`.
If the field is `#[wrap]`, developers must provide a `Wrapper Type` or `Row` instead of an `Column`.

```rust
pub trait GetPositionRowsByXYZ: spacetimedsl::DSLContext {
    #[doc=#comment]
    fn get_positions_by_y<'a>(
        &'a self,
        y: &'a u128,
    ) -> impl Iterator<Item = Position> {
        return self
            .ctx()
            .db()
            .position()
            .y()
            .filter(y);
    }
}

impl GetPositionRowsByXYZ for spacetimedsl::DSL<'_> {}
```

#### Get all Rows

For any table-struct, a `get_all_rows` DSL extension method is created, which performs a `row().iter()`.

```rust
pub trait GetAllEntityRows: spacetimedsl::DSLContext {
    ///Get all Entity.
    fn get_all_entities<'a>(&'a self) -> impl Iterator<Item = Entity> {
        return self.ctx().db().entity().iter();
    }
}
impl GetAllEntityRows for spacetimedsl::DSL<'_> {}

pub trait GetAllIdentifierRows: spacetimedsl::DSLContext {
    ///Get all Identifier.
    fn get_all_identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> {
        return self.ctx().db().identifier().iter();
    }
}
impl GetAllIdentifierRows for spacetimedsl::DSL<'_> {}

pub trait GetAllPositionRows: spacetimedsl::DSLContext {
    ///Get all Position.
    fn get_all_positions<'a>(&'a self) -> impl Iterator<Item = Position> {
        return self.ctx().db().position().iter();
    }
}
impl GetAllPositionRows for spacetimedsl::DSL<'_> {}
```

#### Get count of Rows

For any table-struct, a `get_count_of_rows` DSL extension method is created, which performs a `row().count()`.

```rust
pub trait GetCountOfEntityRows: spacetimedsl::DSLContext {
    ///Get the count of all Entity.
    fn get_count_of_entities<'a>(&'a self) -> u64 {
        return self.ctx().db().entity().count();
    }
}
impl GetCountOfEntityRows for spacetimedsl::DSL<'_> {}

pub trait GetCountOfIdentifierRows: spacetimedsl::DSLContext {
    ///Get the count of all Identifier.
    fn get_count_of_identifiers<'a>(&'a self) -> u64 {
        return self.ctx().db().identifier().count();
    }
}
impl GetCountOfIdentifierRows for spacetimedsl::DSL<'_> {}

pub trait GetCountOfPositionRows: spacetimedsl::DSLContext {
    ///Get the count of all Position.
    fn get_count_of_positions<'a>(&'a self) -> u64 {
        return self.ctx().db().position().count();
    }
}
impl GetCountOfPositionRows for spacetimedsl::DSL<'_> {}
```

#### Update Row by Column

For any `#[primary_key]` and `#[unique]` field on a table-struct, a `update_row_by_column` DSL extension method is created, which performs a `row().column().update(row)`.

If the table has a column with name `modified_at`, it's value is set to the current timestamp before updating.

If a table has only private (non-public) fields, no update method is generated. You can see that because for the `Entity` table, no one was generated, because its only field is a private field which is a primary key and auto inc and shouldn't change.

```rust

pub trait UpdateIdentifierRowById: spacetimedsl::DSLContext {
    ///Update a Identifier row by it's id.
    fn update_identifier_by_id(&self, mut identifier: Identifier) -> Identifier {
        identifier.modified_at = self.ctx().timestamp;
        return self.ctx().db().identifier().id().update(identifier);
    }
}
impl UpdateIdentifierRowById for spacetimedsl::DSL<'_> {}

pub trait UpdateIdentifierRowByEntityId: spacetimedsl::DSLContext {
    ///Update a Identifier row by it's entity_id.
    fn update_identifier_by_entity_id(
        &self,
        mut identifier: Identifier,
    ) -> Identifier {
        identifier.modified_at = self.ctx().timestamp;
        return self.ctx().db().identifier().entity_id().update(identifier);
    }
}
impl UpdateIdentifierRowByEntityId for spacetimedsl::DSL<'_> {}

pub trait UpdateIdentifierRowByValue: spacetimedsl::DSLContext {
    ///Update a Identifier row by it's value.
    fn update_identifier_by_value(&self, mut identifier: Identifier) -> Identifier {
        identifier.modified_at = self.ctx().timestamp;
        return self.ctx().db().identifier().value().update(identifier);
    }
}
impl UpdateIdentifierRowByValue for spacetimedsl::DSL<'_> {}

pub trait UpdatePositionRowById: spacetimedsl::DSLContext {
    ///Update a Position row by it's id.
    fn update_position_by_id(&self, mut position: Position) -> Position {
        position.modified_at = self.ctx().timestamp;
        return self.ctx().db().position().id().update(position);
    }
}
impl UpdatePositionRowById for spacetimedsl::DSL<'_> {}

pub trait UpdatePositionRowByEntityId: spacetimedsl::DSLContext {
    ///Update a Position row by it's entity_id.
    fn update_position_by_entity_id(&self, mut position: Position) -> Position {
        position.modified_at = self.ctx().timestamp;
        return self.ctx().db().position().entity_id().update(position);
    }
}
impl UpdatePositionRowByEntityId for spacetimedsl::DSL<'_> {}
```

#### Delete Row by Column

For any `#[primary_key]` and `#[unique]` field on a table-struct, a `delete_row_by_column` DSL extension method is created, which performs a `row().column().delete(column)` (deletes one row).
If the field is `#[wrap]`, developers must provide a `Wrapper Type` or `Row` instead of an `Column`.

```rust
pub trait DeleteEntityRowById: spacetimedsl::DSLContext {
    ///Delete a Entity row by it's id (or None).
    fn delete_entity_by_id(&self, id: impl Into<EntityId>) -> bool {
        return self.ctx().db().entity().id().delete(id.into().value());
    }
}
impl DeleteEntityRowById for spacetimedsl::DSL<'_> {}

pub trait DeleteIdentifierRowById: spacetimedsl::DSLContext {
    ///Delete a Identifier row by it's id (or None).
    fn delete_identifier_by_id(&self, id: impl Into<IdentifierId>) -> bool {
        return self.ctx().db().identifier().id().delete(id.into().value());
    }
}
impl DeleteIdentifierRowById for spacetimedsl::DSL<'_> {}

pub trait DeleteIdentifierRowByEntityId: spacetimedsl::DSLContext {
    ///Delete a Identifier row by it's entity_id (or None).
    fn delete_identifier_by_entity_id(
        &self,
        entity_id: impl Into<crate::entity::EntityId>,
    ) -> bool {
        return self
            .ctx()
            .db()
            .identifier()
            .entity_id()
            .delete(entity_id.into().value());
    }
}
impl DeleteIdentifierRowByEntityId for spacetimedsl::DSL<'_> {}

pub trait DeleteIdentifierRowByValue: spacetimedsl::DSLContext {
    ///Delete a Identifier row by it's value (or None).
    fn delete_identifier_by_value(&self, value: String) -> bool {
        return self.ctx().db().identifier().value().delete(value);
    }
}
impl DeleteIdentifierRowByValue for spacetimedsl::DSL<'_> {}

pub trait DeletePositionRowById: spacetimedsl::DSLContext {
    ///Delete a Position row by it's id (or None).
    fn delete_position_by_id(&self, id: impl Into<PositionId>) -> bool {
        return self.ctx().db().position().id().delete(id.into().value());
    }
}
impl DeletePositionRowById for spacetimedsl::DSL<'_> {}

pub trait DeletePositionRowByEntityId: spacetimedsl::DSLContext {
    ///Delete a Position row by it's entity_id (or None).
    fn delete_position_by_entity_id(
        &self,
        entity_id: impl Into<crate::entity::EntityId>,
    ) -> bool {
        return self
            .ctx()
            .db()
            .position()
            .entity_id()
            .delete(entity_id.into().value());
    }
}
impl DeletePositionRowByEntityId for spacetimedsl::DSL<'_> {}

```

#### Delete Rows by Column

For any `#[index]` (field) on a table-struct, a `delete_rows_by_columns` DSL extension method is created, which performs a `row().column().delete(column)` (deletes many rows).
If the field is `#[wrap]`, developers must provide a `Wrapper Type` or `Row` instead of an `Column`.

```rust
pub trait DeletePositionRowsByY: spacetimedsl::DSLContext {
    ///Delete a Position row by it's entity_id (or None).
    fn delete_positions_by_y(
        &self,
        y: &'a u128,
    ) -> u64 {
        return self
            .ctx()
            .db()
            .position()
            .y()
            .delete(y);
    }
}
impl DeletePositionRowsByY for spacetimedsl::DSL<'_> {}
```

The implementation is the same as the `Delete Row By Column` method, except that it returns a `u64` instead of a `bool`.

## Licensing

SpacetimeDSL is dual-licensed under both the [MIT License](https://choosealicense.com/licenses/mit/) and the [Apache License (Version 2.0)](https://choosealicense.com/licenses/apache-2.0/) and therefore Open Source.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
