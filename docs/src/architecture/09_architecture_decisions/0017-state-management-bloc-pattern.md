---
    title: 0017 State Management with BLoC Pattern
    adr:
        author: Catalyst Engineering Team
        created: 15-Jan-2024
        status: accepted
    tags:
        - flutter
        - state-management
        - bloc
        - frontend
---

!!! note "Application-Specific ADR"
    This ADR describes the state management architecture of the Catalyst Voices application, not the catalyst-libs libraries.
    It is kept here for reference as Catalyst Voices is a major consumer of catalyst-libs.
    For library-specific architecture decisions, see other ADRs.
    
    This ADR provides detailed implementation decisions for ADR 0005, which established the high-level decision to use BLoC pattern.

## Context

The Catalyst Voices application requires a state management solution that:
- Handles complex asynchronous operations
- Supports reactive UI updates
- Enables testability
- Provides clear separation of business logic from UI
- Supports cross-feature communication

ADR 0005 established the high-level decision to use BLoC pattern. This ADR provides detailed implementation decisions.

## Decision

We use **flutter_bloc** package with the following patterns:

1. **BLoC for Complex State**: Use `Bloc` class for state machines with multiple events
2. **Cubit for Simple State**: Use `Cubit` class for simpler state management with direct method calls
3. **Signals for Cross-BLoC Communication**: Custom signal mechanism for BLoC-to-BLoC communication
4. **Repository Pattern**: BLoCs interact only with repositories, never directly with APIs or databases

## Implementation Patterns

### BLoC Structure
```dart
class ProposalBloc extends Bloc<ProposalEvent, ProposalState> {
  final ProposalRepository repository;
  
  ProposalBloc(this.repository) : super(ProposalInitial()) {
    on<LoadProposal>(_onLoadProposal);
    on<UpdateProposal>(_onUpdateProposal);
  }
  
  Future<void> _onLoadProposal(
    LoadProposal event,
    Emitter<ProposalState> emit,
  ) async {
    emit(ProposalLoading());
    try {
      final proposal = await repository.getProposal(event.id);
      emit(ProposalLoaded(proposal));
    } catch (e) {
      emit(ProposalError(e));
    }
  }
}
```

### Cubit Structure
```dart
class SessionCubit extends Cubit<SessionState> {
  final SessionRepository repository;
  
  SessionCubit(this.repository) : super(SessionInitial());
  
  Future<void> login(String mnemonic) async {
    emit(SessionLoading());
    try {
      final session = await repository.authenticate(mnemonic);
      emit(SessionAuthenticated(session));
    } catch (e) {
      emit(SessionError(e));
    }
  }
}
```

### Signal Pattern
```dart
// Emit signal from one BLoC
emitSignal(AccountUpdatedSignal(userId));

// Listen to signal in another BLoC
onSignal<AccountUpdatedSignal>((signal) {
  // Handle cross-BLoC communication
});
```

## Alternatives Considered

### Provider
- **Pros**: Simple, built into Flutter
- **Cons**: Less structured, harder to test complex flows
- **Rejected**: Not suitable for complex state machines

### Riverpod
- **Pros**: Compile-time safety, excellent testing
- **Cons**: Different paradigm, migration cost
- **Rejected**: BLoC already established, migration not justified

### Redux
- **Pros**: Predictable state management
- **Cons**: Too much boilerplate, not idiomatic for Flutter
- **Rejected**: BLoC provides better Flutter integration

## Risks

* Learning curve for developers new to reactive programming
* Some boilerplate for simple state
* Requires understanding of Streams and async patterns

## Consequences

### Positive
- Clear separation of business logic and UI
- Excellent testability with bloc_test package
- Reactive updates via Streams
- Predictable state transitions
- Good tooling support (BlocObserver for debugging)

### Negative
- Learning curve for developers new to reactive programming
- Some boilerplate for simple state
- Requires understanding of Streams and async patterns

### Best Practices Established
- One BLoC per feature domain
- Repository pattern for data access
- ViewModels for UI-specific transformations
- Signal pattern for cross-BLoC communication
- Error handling via error states

## More Information

* [BLoC Library Documentation](https://bloclibrary.dev/)
* [ADR 0005: Flutter App Architecture](0005-flutter-app.md)
* [Flutter BLoC Examples](https://github.com/felangel/bloc/tree/master/examples)
