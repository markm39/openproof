import Mathlib
/-!
OpenProof scratch session: Fibonacci Gcd Theorem Over `ℕ`
Session id: session_c617393e-cc6e-4522-bb07-b9e78292884c
Mode: research
Updated: 
-/
-- branch: Fibonacci Gcd Theorem Over `ℕ` planner [proving]
-- branch id: branch_96e1704c-c493-4645-81c3-0cf58bd065e9
-- node: Fibonacci Gcd Theorem Over `ℕ` [proving]
-- kind: theorem
-- statement: `theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n)`

-- artifact: Fibonacci Gcd Theorem Over `ℕ` planner [pending]
-- artifact id: artifact_branch_96e1704c-c493-4645-81c3-0cf58bd065e9
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_96e1704c-c493-4645-81c3-0cf58bd065e9
-- label: Fibonacci Gcd Theorem Over `ℕ` planner
-- metadata: {"hidden":true}

import Mathlib.Data.Nat.Fib.Basic

#check Nat.fib_gcd

theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n) := by
  exact (Nat.fib_gcd m n).symm