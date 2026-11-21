---
name: drill
description: Quiz the learner with fresh "does it compile / what does it print?" Rust questions on a weak topic, each verified by actually compiling it. Use for "/drill <topic>", "quiz me on lifetimes", "more practice with borrowing".
---

# Drill a topic

**Topic:** the argument, e.g. `/drill two-phase borrows`. If there's none,
find the weakest spot in `work/progress.toml` (quiz `missed` questions,
exercises with many `failed_checks` or `hints`), and say which topic you
picked and why.

1. **Write five snippets** that probe the topic from different angles: a
   `fn main`, standard library only, at most 15 lines each. Mix "does it
   compile?" with "what does it print?", and include at least one that
   surprises people.
2. **Verify every snippet before using it.** Save it in the scratchpad and
   run `rustc --edition 2024 -C debug-assertions=yes -A warnings main.rs`,
   then run the binary. Never state an answer you haven't seen the compiler
   produce.
3. **Ask one question at a time:** the snippet plus 2–4 choices labelled
   a–d. Then wait for their answer.
4. **After each answer:** right or wrong, what really happens (the first
   error line or the output), and the rule behind it in two or three
   sentences.
5. **At the end:** the score and the one rule to remember. If they missed
   several, point at the matching exercise: `cargo rusty goto <exercise>`.
