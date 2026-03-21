import Mathlib
/-!
OpenProof scratch session: Fibonacci Gcd Theorem Over `ℕ`
Session id: session_c617393e-cc6e-4522-bb07-b9e78292884c
Mode: research
Updated: 
-/
-- branch: Fibonacci Gcd Theorem Over `ℕ` prover [proving]
-- branch id: branch_fda70be3-6f51-41ee-b4e7-217d9e2bc6c7
-- node: Fibonacci Gcd Theorem Over `ℕ` [proving]
-- kind: theorem
-- statement: `theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n)`

-- artifact: Fibonacci Gcd Theorem Over `ℕ` prover [pending]
-- artifact id: artifact_branch_fda70be3-6f51-41ee-b4e7-217d9e2bc6c7
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_fda70be3-6f51-41ee-b4e7-217d9e2bc6c7
-- label: Fibonacci Gcd Theorem Over `ℕ` prover
-- metadata: {"hidden":false,"foreground":true}

#check Nat.fib
#check Nat.gcd

example (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n) := by
  -- proof search next
  fail_if_success exact rfl