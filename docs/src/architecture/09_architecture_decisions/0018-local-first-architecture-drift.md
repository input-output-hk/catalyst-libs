---
    title: 0018 Local-First Architecture with Drift
    adr:
        author: Catalyst Engineering Team
        created: 15-Jan-2024
        status: accepted
    tags:
        - flutter
        - database
        - offline
        - frontend
---

!!! note "Application-Specific ADR"
    This ADR describes the local storage architecture of the Catalyst Voices application, not the catalyst-libs libraries.
    It is kept here for reference as Catalyst Voices is a major consumer of catalyst-libs.
    For library-specific architecture decisions, see other ADRs.

## Context

The Catalyst Voices application must:
- Work offline with full functionality
- Provide fast, responsive UI
- Cache data locally for performance
- Support complex queries on JSON documents
- Work across Web, iOS, and Android platforms

We need a local database solution that supports:
- SQLite with JSONB support (3.45.0+)
- Type-safe queries
- Reactive streams for UI updates
- Schema migrations
- Cross-platform compatibility

## Decision

We use **Drift** (formerly Moor) as our local database solution with:

1. **SQLite Backend**: Native SQLite on mobile, WASM SQLite on Web
2. **JSONB Support**: Leverage SQLite 3.45.0+ JSONB functions for document queries
3. **Reactive Streams**: Drift's watch queries for automatic UI updates
4. **Type Safety**: Generated code from Dart table definitions
5. **Migrations**: Versioned schema migrations with Drift

## Implementation Details

### Database Schema
```dart
@DataClassName('DocumentRow')
class Documents extends Table {
  IntColumn get id => integer().autoIncrement()();
  TextColumn get documentId => text()();
  TextColumn get content => text()(); // JSONB column
  DateTimeColumn get createdAt => dateTime()();
  
  @override
  Set<Column> get primaryKey => {documentId};
}
```

### Repository Pattern
```dart
class DocumentRepository {
  final CatalystDatabase _db;
  
  Stream<List<Document>> watchDocuments() {
    return _db.select(_db.documents).watch();
  }
  
  Future<void> insertDocument(Document doc) async {
    await _db.into(_db.documents).insert(
      DocumentsCompanion.insert(
        documentId: doc.id,
        content: doc.toJson(),
      ),
    );
  }
}
```

### Web Platform Considerations
- Requires `sqlite3.v1.wasm` and `driftWorker.js`
- Needs CORS headers: `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`
- WASM files must be served with `Content-Type: application/wasm`

## Alternatives Considered

### Hive
- **Pros**: NoSQL, simple API
- **Cons**: No SQL queries, limited query capabilities
- **Rejected**: Need SQL queries for complex document filtering

### Isar
- **Pros**: Fast, good query capabilities
- **Cons**: Less mature, smaller community
- **Rejected**: Drift has better cross-platform support

### sqflite (direct SQLite)
- **Pros**: Direct SQLite access
- **Cons**: No type safety, manual migrations, no reactive streams
- **Rejected**: Drift provides better developer experience

## Risks

* SQLite version requirement (3.45.0+)
* Web platform requires WASM setup
* Migration complexity for schema changes
* Initial database setup overhead

## Consequences

### Positive
- Full offline functionality
- Fast local queries with indexing
- Reactive UI updates via streams
- Type-safe database operations
- Cross-platform compatibility

### Negative
- SQLite version requirement (3.45.0+)
- Web platform requires WASM setup
- Migration complexity for schema changes
- Initial database setup overhead

### Follow-up Work
- Document migration procedures
- Establish JSONB query patterns
- Create database testing utilities

## More Information

* [Drift Documentation](https://drift.simonbinder.eu/)
* [SQLite JSONB Functions](https://www.sqlite.org/jsonb.html)
* [Drift Web Support](https://drift.simonbinder.eu/web/)
