# Essential Rust Mastery Topics

[1. **Ownership, Move Semantics, and Borrowing Rules**](docs/MainTopics/01_Ownership_Move_Semantics_and_Borrowing_Rules.md)<br>
Understanding ownership transfer, move vs copy semantics, and the fundamental borrowing rules (mutable and immutable references).

[2. **Lifetimes and Lifetime Annotations**](docs/MainTopics/02_Lifetimes_and_Lifetime_Annotations.md)<br>
Lifetime parameters, elision rules, bounds, higher-ranked trait bounds (HRTBs), and non-lexical lifetimes (NLL).

[3. **Interior Mutability Patterns**](docs/MainTopics/03_Interior_Mutability_Patterns.md)<br>
Cell, RefCell, Mutex, RwLock, and understanding when to use each for safe mutation of shared data.

[4. **Reference Counting and Smart Pointers**](docs/MainTopics/04_Reference_Counting_and_Smart_Pointers.md)<br>
Rc, Arc, Weak, Box, Cow, custom smart pointers, and Deref coercion.

[5. **Structs, Enums, and Pattern Matching**](docs/MainTopics/05_Structs_Enums_and_Pattern_Matching.md)<br>
Struct types, enum variants, exhaustive pattern matching, destructuring, and match guards.

[6. **Generics and Monomorphization**](docs/MainTopics/06_Generics_and_Monomorphization.md)<br>
Generic types, functions, and understanding how Rust compiles generics into concrete types.

[7. **Traits: Definition and Implementation**](docs/MainTopics/07_Traits_Definition_and_Implementation.md)<br>
Defining traits, implementing traits for types, default implementations, and associated functions.

[8. **Trait Bounds and Where Clauses**](docs/MainTopics/08_Trait_Bounds_and_Where_Clauses.md)<br>
Constraining generic types with trait bounds, complex where clauses, and multiple bounds.

[9. **Associated Types vs Generic Parameters**](docs/MainTopics/09_Associated_Types_vs_Generic_Parameters.md)<br>
Understanding when to use associated types versus generic type parameters in trait design.

[10. **Trait Objects and Dynamic Dispatch**](docs/MainTopics/10_Trait_Objects_and_Dynamic_Dispatch.md)<br>
Using dyn Trait, object safety rules, vtables, and performance implications of dynamic dispatch.

[11. **Marker Traits: Send, Sync, Copy, Sized**](docs/MainTopics/11_Marker_Traits_Send_Sync_Copy_Sized.md)<br>
Understanding auto traits, their implications for concurrency, and memory semantics.

[12. **Advanced Trait Features**](docs/MainTopics/12_Advanced_Trait_Features.md)<br>
Supertraits, blanket implementations, orphan rules, coherence, and sealed trait pattern.

[13. **Type System Deep Dive**](docs/MainTopics/13_Type_System_Deep_Dive.md)<br>
Zero-sized types (ZSTs), phantom data, unsized types (DSTs), type aliases, and newtype pattern.

[14. **Const Generics and Generic Associated Types**](docs/MainTopics/14_Const_Generics_and_Generic_Associated_Types.md)<br>
Using const parameters in generics and understanding GATs for advanced abstractions.

[15. **Return and Argument Position impl Trait**](docs/MainTopics/15_Return_and_Argument_Position_impl_Trait.md)<br>
RPIT, APIT, abstract return types, and their use cases in API design.

[16. **Iterator Trait and Lazy Evaluation**](docs/MainTopics/16_Iterator_Trait_and_Lazy_Evaluation.md)<br>
Iterator protocol, custom iterators, iterator combinators, and zero-cost abstractions.

[17. **Closure Traits and Capture Semantics**](docs/MainTopics/17_Closure_Traits_and_Capture_Semantics.md)<br>
Fn, FnMut, FnOnce, closure capture modes, and move closures.

[18. **Result, Option, and Error Handling**](docs/MainTopics/18_Result_Option_and_Error_Handling.md)<br>
Using Result/Option types, the ? operator, and error propagation patterns.

[19. **Custom Error Types and Error Trait**](docs/MainTopics/19_Custom_Error_Types_and_Error_Trait.md)<br>
Implementing the Error trait, using anyhow/thiserror, error context, and backtraces.

[20. **Panic Handling and Unwinding**](docs/MainTopics/20_Panic_Handling_and_Unwinding.md)<br>
Understanding panic vs recoverable errors, catch_unwind, UnwindSafe, and abort behavior.

[21. **Unsafe Rust Fundamentals**](docs/MainTopics/21_Unsafe_Rust_Fundamentals.md)<br>
Unsafe blocks, functions, raw pointers, and the five unsafe superpowers.

[22. **Understanding Undefined Behavior**](docs/MainTopics/22_Understanding_Undefined_Behavior.md)<br>
What constitutes UB in Rust, common pitfalls, and how to avoid it.

[23. **Memory Layout and Representations**](docs/MainTopics/23_Memory_Layout_and_Representations.md)<br>
Stack vs heap, alignment, padding, repr(C/transparent/packed), and memory layout guarantees.

[24. **FFI and C Interoperability**](docs/MainTopics/24_FFI_and_C_Interoperability.md)<br>
C ABI compatibility, cbindgen, bindgen, calling conventions, and cross-language safety.

[25. **Thread Safety and Concurrency Primitives**](docs/MainTopics/25_Thread_Safety_and_Concurrency_Primitives.md)<br>
Send/Sync deep dive, data races, Mutex, RwLock, atomic operations, and lock-free structures.

[26. **Memory Ordering and Atomics**](docs/MainTopics/26_Memory_Ordering_and_Atomics.md)<br>
Atomic types, memory ordering (Relaxed, Acquire, Release, SeqCst), and synchronization.

[27. **Channels and Message Passing**](docs/MainTopics/27_Channels_and_Message_Passing.md)<br>
mpsc channels, crossbeam channels, bounded/unbounded queues, and CSP patterns.

[28. **Async/Await and Futures**](docs/MainTopics/28_Async_Await_and_Futures.md)<br>
Async syntax, the Future trait, poll-based execution, and async function transformations.

[29. **Pinning and the Pin Type**](docs/MainTopics/29_Pinning_and_the_Pin_Type.md)<br>
Understanding Pin<T>, Unpin trait, self-referential types, and why pinning is necessary.

[30. **Async Runtimes and Executors**](docs/MainTopics/30_Async_Runtimes_and_Executors.md)<br>
Tokio, async-std, custom executors, single vs multi-threaded runtimes, and work stealing.

[31. **Streams and Async Iteration**](docs/MainTopics/31_Streams_and_Async_Iteration.md)<br>
The Stream trait, async iteration patterns, and differences from sync iterators.

[32. **Async Patterns and Best Practices**](docs/MainTopics/32_Async_Patterns_and_Best_Practices.md)<br>
Select/join operations, cancellation, timeouts, backpressure, and structured concurrency.

[33. **Declarative Macros macro_rules**](docs/MainTopics/33_Declarative_Macros.md)<br>
Pattern matching syntax, fragment specifiers, repetition, hygiene, and debugging macros.

[34. **Procedural Macros**](docs/MainTopics/34_Procedural_Macros.md)<br>
Derive macros, attribute macros, function-like macros, TokenStream manipulation, syn/quote.

[35. **Common Design Patterns**](docs/MainTopics/35_Common_Design_Patterns.md)<br>
Builder, type state, RAII, visitor, strategy, extension traits, and the newtype pattern.

[36. **Zero-Cost Abstractions and Performance**](docs/MainTopics/36_Zero_Cost_Abstractions_and_Performance.md)<br>
Verifying zero-cost abstractions, inline hints, monomorphization costs, and optimization strategies.

[37. **SIMD and Vectorization**](docs/MainTopics/37_SIMD_and_Vectorization.md)<br>
Using SIMD instructions, portable SIMD, and writing cache-friendly code.

[38. **Profiling and Benchmarking**](docs/MainTopics/38_Profiling_and_Benchmarking.md)<br>
Using perf, flamegraph, criterion, memory profilers, and interpreting performance data.

[39. **Optimization Techniques**](docs/MainTopics/39_Optimization_Techniques.md)<br>
Avoiding allocations, LTO, PGO, compiler optimization levels, and target-cpu flags.

[40. **Testing Strategies**](docs/MainTopics/40_Testing_Strategies.md)<br>
Unit tests, integration tests, doc tests, property-based testing (proptest), and mocking.

[41. **Documentation and Rustdoc**](docs/MainTopics/41_Documentation_and_Rustdoc.md)<br>
Writing effective documentation, doc comments, code examples, and documentation tests.

[42. **Cargo and Workspace Management**](docs/MainTopics/42_Cargo_and_Workspace_Management.md)<br>
Workspaces, feature flags, build scripts (build.rs), profiles, and dependency management.

[43. **Development Tooling**](docs/MainTopics/43_Development_Tooling.md)<br>
Clippy configuration, rustfmt, rust-analyzer, cargo-expand, miri, and cargo-audit.

[44. **No-std and Embedded Development**](docs/MainTopics/44_No_std_and_Embedded_Development.md)<br>
Working without the standard library, custom allocators, panic handlers, and embedded constraints.

[45. **WebAssembly (WASM) Compilation**](docs/MainTopics/45_WebAssembly_Compilation.md)<br>
Compiling to WASM, wasm-bindgen, interacting with JavaScript, and optimizing WASM binaries.

[46. **Type-Level Programming**](docs/MainTopics/46_Type_Level_Programming.md)<br>
Encoding state machines in types, compile-time computation, const evaluation, and type-level proofs.

[47. **Compiler Internals Understanding**](docs/MainTopics/47_Compiler_Internals_Understanding.md)<br>
MIR, borrow checker algorithm, trait resolution, reading compiler errors, and HIR/MIR stages.

[48. **Cross-Compilation and Target Specifications**](docs/MainTopics/48_Cross_Compilation_and_Target_Specifications.md)<br>
Target triples, cross-compiling, linker configuration, and platform-specific code.

[49. **Database and Web Frameworks**](docs/MainTopics/49_Database_and_Web_Frameworks.md)<br>
sqlx, diesel, connection pooling, Axum, Actix-web, Rocket, and async web patterns.

[50. **Community Standards and Best Practices**](docs/MainTopics/50_Community_Standards_and_Best_Practices.md)<br>
RFC process, API guidelines, semantic versioning, crate evaluation, and contributing to the ecosystem.

## Other

[100. **Modifying Static Mutable Variables**](docs/Other/100_Modifying_Static_Mutable_Variables.md)<br>
[101. **Pin Short Explanation**](docs/Other/101_Pin_Explained.md)<br>
[102. **Spin Crate Examples**](docs/Other/102_Spin_Crate_Examples.md)<br>

## Summary

[1000. **Summary**](docs/Summary/1000_Summary.md)<br>