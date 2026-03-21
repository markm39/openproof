import Mathlib
/-!
OpenProof scratch session: Fibonacci Gcd Theorem Over `ℕ`
Session id: session_c617393e-cc6e-4522-bb07-b9e78292884c
Mode: research
Updated: 
-/
-- branch: Fibonacci Gcd Theorem Over `ℕ` repair [proving]
-- branch id: branch_23cdfcc7-2822-4ab9-895b-b15833eb8949
-- node: Fibonacci Gcd Theorem Over `ℕ` [proving]
-- kind: theorem
-- statement: `theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n)`

-- artifact: Fibonacci Gcd Theorem Over `ℕ` repair [pending]
-- artifact id: artifact_branch_23cdfcc7-2822-4ab9-895b-b15833eb8949
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_23cdfcc7-2822-4ab9-895b-b15833eb8949
-- label: Fibonacci Gcd Theorem Over `ℕ` repair
-- metadata: {"hidden":true}

theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n) := by
  exact (Nat.fib_gcd m n).symm