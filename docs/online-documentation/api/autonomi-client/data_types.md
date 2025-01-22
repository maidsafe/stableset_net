# Data Types Reference

This page provides detailed information about the core data types used in the Autonomi Client API.

## Address

A unique identifier for content in the network.

=== "Node.js"
    ```typescript
    interface Address {
        readonly bytes: Buffer;
        toString(): string;
        equals(other: Address): boolean;
    }
    ```

=== "Python"
    ```python
    class Address:
        @property
        def bytes(self) -> bytes: ...
        def __str__(self) -> str: ...
        def __eq__(self, other: Address) -> bool: ...
    ```

=== "Rust"
    ```rust
    pub struct Address([u8; 32]);

    impl Address {
        pub fn as_bytes(&self) -> &[u8];
        pub fn to_string(&self) -> String;
    }
    ```

## Chunk

An immutable data block with quantum-secure encryption.

=== "Node.js"
    ```typescript
    interface Chunk {
        readonly address: Address;
        readonly data: Buffer;
        readonly size: number;
        readonly type: ChunkType;
    }

    enum ChunkType {
        Data,
        Metadata,
        Index
    }
    ```

=== "Python"
    ```python
    class Chunk:
        @property
        def address(self) -> Address: ...
        @property
        def data(self) -> bytes: ...
        @property
        def size(self) -> int: ...
        @property
        def type(self) -> ChunkType: ...

    class ChunkType(Enum):
        DATA = 1
        METADATA = 2
        INDEX = 3
    ```

=== "Rust"
    ```rust
    pub struct Chunk {
        pub address: Address,
        pub data: Vec<u8>,
        pub size: usize,
        pub type_: ChunkType,
    }

    pub enum ChunkType {
        Data,
        Metadata,
        Index,
    }
    ```

## Pointer

A mutable reference to data in the network.

=== "Node.js"
    ```typescript
    interface Pointer {
        readonly address: Address;
        readonly target: Address;
        readonly version: number;
        setTarget(target: Address): void;
    }
    ```

=== "Python"
    ```python
    class Pointer:
        @property
        def address(self) -> Address: ...
        @property
        def target(self) -> Address: ...
        @property
        def version(self) -> int: ...
        def set_target(self, target: Address) -> None: ...
    ```

=== "Rust"
    ```rust
    pub struct Pointer {
        pub address: Address,
        pub target: Address,
        pub version: u64,
    }

    impl Pointer {
        pub fn set_target(&mut self, target: Address);
    }
    ```

## GraphEntry

A decentralized Graph structure for linked data.

=== "Node.js"
    ```typescript
    interface GraphEntry<T> {
        readonly address: Address;
        readonly length: number;
        append(item: T): void;
        get(index: number): T;
        toArray(): T[];
    }
    ```

=== "Python"
    ```python
    class GraphEntry(Generic[T]):
        @property
        def address(self) -> Address: ...
        @property
        def length(self) -> int: ...
        def append(self, item: T) -> None: ...
        def __getitem__(self, index: int) -> T: ...
        def to_list(self) -> List[T]: ...
    ```

=== "Rust"
    ```rust
    pub struct GraphEntry<T> {
        pub address: Address,
        pub length: usize,
    }

    impl<T> GraphEntry<T> {
        pub fn append(&mut self, item: T);
        pub fn get(&self, index: usize) -> Option<&T>;
        pub fn to_vec(&self) -> Vec<T>;
    }
    ```

## ScratchPad

Unstructured data with CRDT properties.

=== "Node.js"
    ```typescript
    interface ScratchPad {
        readonly address: Address;
        readonly type: ContentType;
        readonly updateCounter: number;
        update(data: Buffer): void;
        getData(): Buffer;
    }
    ```

=== "Python"
    ```python
    class ScratchPad:
        @property
        def address(self) -> Address: ...
        @property
        def type(self) -> ContentType: ...
        @property
        def update_counter(self) -> int: ...
        def update(self, data: bytes) -> None: ...
        def get_data(self) -> bytes: ...
    ```

=== "Rust"
    ```rust
    pub struct ScratchPad {
        pub address: Address,
        pub type_: ContentType,
        pub update_counter: u64,
    }

    impl ScratchPad {
        pub fn update(&mut self, data: Vec<u8>);
        pub fn get_data(&self) -> Vec<u8>;
    }
    ``` 
