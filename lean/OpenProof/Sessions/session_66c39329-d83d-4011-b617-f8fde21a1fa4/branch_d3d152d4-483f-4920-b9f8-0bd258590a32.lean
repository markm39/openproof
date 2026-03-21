import Mathlib
/-!
OpenProof scratch session: Fibonacci Gcd Identity
Session id: session_66c39329-d83d-4011-b617-f8fde21a1fa4
Mode: research
Updated: 
-/
-- branch: Fibonacci Gcd Identity repair [proving]
-- branch id: branch_d3d152d4-483f-4920-b9f8-0bd258590a32
-- node: Fibonacci Gcd Identity [verifying]
-- kind: theorem
-- statement: theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n)

-- artifact: Fibonacci Gcd Identity repair [pending]
-- artifact id: artifact_branch_d3d152d4-483f-4920-b9f8-0bd258590a32
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_d3d152d4-483f-4920-b9f8-0bd258590a32
-- label: Fibonacci Gcd Identity repair
-- metadata: {"hidden":true}

#check Nat.fib
#check Nat.gcd
#check Nat.gcd_fib

example (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n) := by
  simpa using Nat.gcd_fib m n