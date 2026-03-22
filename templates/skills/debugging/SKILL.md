---
name: debugging
description: Systematic debugging process — reproduce, isolate, fix, verify. Use when tracking down bugs, unexpected behavior, or test failures.
---

# Debugging

## When to Use

When the user reports a bug, error, unexpected behavior, or says "this doesn't work", "why is this failing?", or "help me debug".

## Instructions

1. **Reproduce:** Get the exact steps, inputs, and error messages. Confirm you can trigger the issue.
2. **Read the error:** Parse the full stack trace or error message. The root cause is often in the first or last frame, not the middle.
3. **Isolate:** Narrow the scope:
   - Binary search through code (comment out halves)
   - Check recent changes (`git log`, `git diff`)
   - Add targeted logging or print statements
   - Test with minimal input
4. **Hypothesize and test:** Form a theory, then test it. Don't change multiple things at once.
5. **Fix:** Apply the minimal change that resolves the issue.
6. **Verify:**
   - Confirm the original error is gone
   - Run the full test suite to check for regressions
   - Test edge cases near the fix
7. **Document:** Add a test that would have caught this bug. Add a comment if the fix is non-obvious.

## Examples

**User:** "My tests pass locally but fail in CI"
**Agent:** Checks for environment differences: OS, dependency versions, file paths, timezone, locale settings, race conditions in parallel tests. Compares CI logs with local output to identify the divergence point.

**User:** "This function returns wrong results for some inputs"
**Agent:** Asks for the specific failing inputs, adds assertions at intermediate steps, finds where actual diverges from expected, traces back to the root cause.
