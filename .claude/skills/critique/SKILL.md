---
name: critique
description: Review the learner's solution to a solved helpmeiamrusty exercise for idiom and style, comparing it with the reference solution. Use for "/critique", "review my solution", "is this idiomatic?", or right after an exercise passes.
---

# Critique a solved exercise

1. **Find the exercise.** Use the argument if one was given (an id like
   `01-ownership/04_split_borrows`, a name, or a fragment). Otherwise use
   `current` from `work/progress.toml`, or the exercise solved most recently
   (highest `solved_at`).
2. **Confirm it passes:** `cargo rusty check <exercise>`. If it doesn't,
   stop. Say it isn't solved yet and offer the next hint instead (the tutor
   rules in CLAUDE.md apply: no solution).
3. **Read both files:** theirs at `work/<module>/<name>.rs`, and the
   reference at `modules/<module>/solutions/<name>.rs`, including the notes
   in its `//!` header.
4. **Reply in this shape, in under ~25 lines:**
   - **Good:** one or two *specific* things they did well.
   - **The one change worth making:** the single most valuable improvement,
     as a short before/after taken from *their* code. Prefer idiom and API
     knowledge (the entry API, an iterator adaptor, a more general borrowed
     type, `mem::take`) over formatting nits. If their code is already as
     good as the reference, say so and skip this.
   - **Also possible:** at most two other approaches, one line each, when
     the reference differs meaningfully.
   - **Remember:** one sentence worth internalizing.
5. Don't rewrite their file or edit anything unless they ask.
