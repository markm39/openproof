import Mathlib
/-!
OpenProof scratch session: Fibonacci Gcd Identity
Session id: session_66c39329-d83d-4011-b617-f8fde21a1fa4
Mode: research
Updated: 
-/
-- branch: Fibonacci Gcd Identity prover [proving]
-- branch id: branch_1b7ca0ea-4cad-459a-a325-7e794b0125c8
-- node: Fibonacci Gcd Identity [proving]
-- kind: theorem
-- statement: theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n)

-- artifact: Fibonacci Gcd Identity prover [pending]
-- artifact id: artifact_branch_1b7ca0ea-4cad-459a-a325-7e794b0125c8
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_1b7ca0ea-4cad-459a-a325-7e794b0125c8
-- label: Fibonacci Gcd Identity prover
-- metadata: {"hidden":false,"foreground":true}

#check Nat.fib
#check Nat.gcd

example (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n) := by
  simp