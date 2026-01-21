---
    title: 0016 Routing Architecture with GoRouter
    adr:
        author: Catalyst Engineering Team
        created: 15-Jan-2024
        status: accepted
    tags:
        - flutter
        - routing
        - frontend
---

!!! note "Application-Specific ADR"
    This ADR describes the routing architecture of the Catalyst Voices application, not the catalyst-libs libraries.
    It is kept here for reference as Catalyst Voices is a major consumer of catalyst-libs.
    For library-specific architecture decisions, see other ADRs.

## Context

The Catalyst Voices frontend application requires a robust routing solution that supports:
- Type-safe navigation
- Deep linking
- Route guards and authentication checks
- Nested routing for complex UI flows
- Web URL handling
- Route restoration for state persistence

We evaluated several routing solutions including:
- Navigator 2.0 (Flutter's built-in solution)
- AutoRoute
- GoRouter

## Decision

We chose **GoRouter** as our routing solution for the following reasons:

1. **Type Safety**: Provides compile-time route safety with code generation
2. **Declarative API**: Clean, declarative route definitions
3. **Deep Linking**: Built-in support for web URLs and deep links
4. **Route Guards**: Easy implementation of authentication and authorization checks
5. **State Restoration**: Built-in support for Flutter's restoration API
6. **Active Maintenance**: Well-maintained package with active community
7. **Code Generation**: `go_router_builder` provides type-safe navigation

## Alternatives Considered

### Navigator 2.0
- **Pros**: Built into Flutter, no external dependencies
- **Cons**: Verbose API, requires significant boilerplate, complex nested routing
- **Rejected**: Too much boilerplate for our use case

### AutoRoute
- **Pros**: Code generation, type-safe routes
- **Cons**: Less flexible than GoRouter, smaller community
- **Rejected**: GoRouter provides better deep linking support

## Implementation Details

Routes are defined in `lib/routes/routing/` with:
- Route definitions using GoRouter's declarative API
- Route guards for authentication
- Nested routes for complex flows
- Type-safe navigation via generated code

Example route structure:
```dart
GoRoute(
  path: '/proposal/:id',
  builder: (context, state) {
    final id = state.pathParameters['id']!;
    return ProposalPage(proposalId: id);
  },
)
```

## Risks

* Additional dependency (go_router package)
* Learning curve for team members unfamiliar with GoRouter
* Code generation step required in build process

## Consequences

### Positive
- Type-safe navigation reduces runtime errors
- Deep linking enables better web and mobile UX
- Route guards simplify authentication flow
- Code generation ensures consistency

### Negative
- Additional dependency (go_router package)
- Learning curve for team members unfamiliar with GoRouter
- Code generation step required in build process

### Follow-up Work
- Document route naming conventions
- Create route guard patterns documentation
- Establish deep linking URL structure standards

## More Information

* [GoRouter Documentation](https://pub.dev/packages/go_router)
* [GoRouter Builder](https://pub.dev/packages/go_router_builder)
* [Flutter Navigation and Routing](https://docs.flutter.dev/development/ui/navigation)
