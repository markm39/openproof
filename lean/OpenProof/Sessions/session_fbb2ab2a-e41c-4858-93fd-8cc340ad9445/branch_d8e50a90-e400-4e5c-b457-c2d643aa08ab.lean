import Mathlib
/-!
OpenProof scratch session: GCD of Fibonacci Numbers
Session id: session_fbb2ab2a-e41c-4858-93fd-8cc340ad9445
Mode: research
Updated: 
-/
-- branch: GCD of Fibonacci Numbers prover [proving]
-- branch id: branch_d8e50a90-e400-4e5c-b457-c2d643aa08ab
-- node: GCD of Fibonacci Numbers [proving]
-- kind: theorem
-- statement: `theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n)`

-- artifact: GCD of Fibonacci Numbers prover [pending]
-- artifact id: artifact_branch_d8e50a90-e400-4e5c-b457-c2d643aa08ab
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_d8e50a90-e400-4e5c-b457-c2d643aa08ab
-- label: GCD of Fibonacci Numbers prover
-- metadata: {"hidden":false,"foreground":true}

theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n) := by
  simpa using Nat.gcd_fib m n