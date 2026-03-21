import Mathlib
/-!
OpenProof scratch session: GCD of Fibonacci Numbers
Session id: session_fbb2ab2a-e41c-4858-93fd-8cc340ad9445
Mode: research
Updated: 
-/
-- branch: GCD of Fibonacci Numbers repair [proving]
-- branch id: branch_cf9e226d-e7c9-440d-9e57-7a5caab5b95b
-- node: GCD of Fibonacci Numbers [verifying]
-- kind: theorem
-- statement: `theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n)`

-- artifact: GCD of Fibonacci Numbers repair [pending]
-- artifact id: artifact_branch_cf9e226d-e7c9-440d-9e57-7a5caab5b95b
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_cf9e226d-e7c9-440d-9e57-7a5caab5b95b
-- label: GCD of Fibonacci Numbers repair
-- metadata: {"hidden":true}

theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n) := by
  simpa using (Nat.fib_gcd m n).symm