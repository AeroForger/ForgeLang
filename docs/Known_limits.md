Known Limits (Alpha 2)

    Code Chernobyl (CC) — The 7.1M Incident

    Classification: Severity 1 (Desktop Near-Miss)Date: Alpha 0.2 developmentCause: 7,101,004-line stress test fed to the compiler as a singlefunction. Frontend completed; the LLVM backend consumed 5.6GB+ RAM andwas still climbing when manually terminated seconds before swap-thrashfroze the system.Casualties: None. All processes killed cleanly. Zero failed units.Lesson: Single functions do not scale. Functions do.Permanent remedies: per-function compilation units (C++ port),memory-capped builds for stress tests, and While loops so that noForgeLang program ever needs 7.1 million statements again.

    "The frontend scaled linearly. The backend did not."  — CC Incident Report
