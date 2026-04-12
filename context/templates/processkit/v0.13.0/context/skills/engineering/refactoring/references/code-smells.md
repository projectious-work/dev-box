# Code Smells Catalog

A comprehensive catalog of code smells organized by category, with detection heuristics and recommended refactorings. Based on Martin Fowler's "Refactoring" and the refactoring.guru catalog.

## Bloaters

Code that has grown too large to work with effectively.

### Long Method
- **Detection:** Function exceeds ~20 lines, or you need to scroll to read it, or it does more than one thing, or you feel the need to write a comment explaining a block.
- **Refactorings:** Extract Function, Replace Temp with Query, Introduce Parameter Object, Decompose Conditional, Replace Method with Method Object.

### Large Class
- **Detection:** Class has more than ~5 instance variables, more than ~300 lines, or its name includes "And" or "Manager" or "Utility".
- **Refactorings:** Extract Class, Extract Subclass, Extract Interface.

### Primitive Obsession
- **Detection:** Using strings for emails, URLs, currency amounts; using integers for IDs or status codes; parallel arrays instead of objects.
- **Refactorings:** Replace Data Value with Object, Introduce Parameter Object, Replace Type Code with Class/Subclass.

### Long Parameter List
- **Detection:** Function takes more than 3 parameters, especially if several are of the same type or frequently passed together.
- **Refactorings:** Introduce Parameter Object, Preserve Whole Object, Replace Parameter with Method Call.

### Data Clumps
- **Detection:** The same 3+ variables appear together in multiple places (function signatures, class fields, data structures).
- **Refactorings:** Extract Class, Introduce Parameter Object.

## Object-Orientation Abusers

Incomplete or incorrect use of OOP principles.

### Switch Statements
- **Detection:** Switch/match on type codes that appear in multiple places; adding a new type requires changes in many locations.
- **Refactorings:** Replace Conditional with Polymorphism, Replace Type Code with Subclasses, Introduce Strategy pattern.

### Temporary Field
- **Detection:** An object field is only set and used in certain code paths; it is null/None/empty most of the time.
- **Refactorings:** Extract Class (move field and its related code into a separate object), Introduce Null Object.

### Refused Bequest
- **Detection:** A subclass inherits methods or data it does not use or overrides parent methods to do nothing.
- **Refactorings:** Replace Inheritance with Delegation, Push Down Method/Field, Extract Subclass.

### Alternative Classes with Different Interfaces
- **Detection:** Two classes do similar things but have different method names or signatures.
- **Refactorings:** Rename Method, Move Method, Extract Superclass/Interface.

## Change Preventers

Smells that make changes unnecessarily expensive.

### Divergent Change
- **Detection:** A single class is modified for multiple unrelated reasons (e.g., changes to database schema AND changes to business rules both touch the same class).
- **Refactorings:** Extract Class (one class per reason for change).

### Shotgun Surgery
- **Detection:** A single logical change requires small edits across many different classes or files.
- **Refactorings:** Move Method, Move Field, Inline Class (consolidate scattered logic).

### Parallel Inheritance Hierarchies
- **Detection:** Every time you add a subclass to one hierarchy, you must add a corresponding subclass to another.
- **Refactorings:** Move Method/Field to eliminate one hierarchy, use delegation.

## Dispensables

Code that contributes nothing and should be removed.

### Duplicate Code
- **Detection:** Identical or near-identical blocks in two or more places. Includes structural duplication (same algorithm with different data).
- **Refactorings:** Extract Function, Extract Class, Pull Up Method, Form Template Method.

### Dead Code
- **Detection:** Code that is never called, variables that are assigned but never read, imports never used, conditions that can never be true.
- **Refactorings:** Delete it. Use your VCS if you need it back.

### Lazy Class
- **Detection:** A class that does too little to justify its existence (single method, trivial delegation).
- **Refactorings:** Inline Class, Collapse Hierarchy.

### Speculative Generality
- **Detection:** Abstractions, interfaces, or parameters that exist "just in case" but have only one implementation or caller.
- **Refactorings:** Collapse Hierarchy, Inline Class, Remove Parameter, Rename Method.

### Data Class
- **Detection:** A class with only fields and getters/setters but no behavior.
- **Refactorings:** Move related behavior into the class (Encapsulate Field, Move Method). Note: data-only types are valid in some contexts (DTOs, value objects).

### Comments (as smell)
- **Detection:** Long comments explaining what the code does (rather than why). The code should be self-documenting.
- **Refactorings:** Extract Function (name replaces comment), Rename Variable/Method, Introduce Assertion.

## Couplers

Smells that create excessive dependencies between components.

### Feature Envy
- **Detection:** A method accesses data from another object more than its own. It "envies" the other object's data.
- **Refactorings:** Move Method (to the envied class), Extract Method then Move Method.

### Inappropriate Intimacy
- **Detection:** Two classes access each other's private/internal members, or one class has deep knowledge of another's implementation.
- **Refactorings:** Move Method/Field, Extract Class, Replace Inheritance with Delegation, Hide Delegate.

### Message Chains
- **Detection:** A client calls `a.getB().getC().getD()` -- a long chain of object navigations.
- **Refactorings:** Hide Delegate, Extract Method, Move Method.

### Middle Man
- **Detection:** A class where most methods just delegate to another class without adding value.
- **Refactorings:** Remove Middle Man (access the delegate directly), Inline Method, Replace Delegation with Inheritance (rare).

---

## References

- Martin Fowler, "Refactoring: Improving the Design of Existing Code" (2018, 2nd ed.)
- https://refactoring.guru/refactoring/smells
- Joshua Kerievsky, "Refactoring to Patterns" (2004)
